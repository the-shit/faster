//! SQLite task queue

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub status: TaskStatus,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Queued => "queued",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "queued" => Some(TaskStatus::Queued),
            "running" => Some(TaskStatus::Running),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }
}

pub struct TaskQueue {
    pool: SqlitePool,
}

impl TaskQueue {
    /// Create new task queue
    pub async fn new(db_path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path)).await?;

        let queue = Self { pool };
        queue.init_schema().await?;

        Ok(queue)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                command TEXT NOT NULL,
                status TEXT NOT NULL,
                model TEXT,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT,
                error TEXT
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        // Create index on status for efficient querying
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Add task to queue
    pub async fn enqueue(&self, command: &str, model: Option<String>) -> Result<String> {
        let id = nanoid::nanoid!(8);
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO tasks (id, command, status, model, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&id)
        .bind(command)
        .bind(TaskStatus::Queued.as_str())
        .bind(model)
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get next queued task
    pub async fn dequeue(&self) -> Result<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT id, command, status, model, created_at, started_at, completed_at, error
            FROM tasks
            WHERE status = ?
            ORDER BY created_at ASC
            LIMIT 1
            "#
        )
        .bind(TaskStatus::Queued.as_str())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Task {
                id: row.get("id"),
                command: row.get("command"),
                status: TaskStatus::from_str(row.get("status")).unwrap(),
                model: row.get("model"),
                created_at: DateTime::parse_from_rfc3339(row.get("created_at"))?.with_timezone(&Utc),
                started_at: row.get::<Option<String>, _>("started_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                completed_at: row.get::<Option<String>, _>("completed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                error: row.get("error"),
            })),
            None => Ok(None),
        }
    }

    /// Update task status
    pub async fn update_status(&self, id: &str, status: TaskStatus) -> Result<()> {
        let mut query = String::from("UPDATE tasks SET status = ?");

        if status == TaskStatus::Running {
            query.push_str(", started_at = ?");
        } else if status == TaskStatus::Completed || status == TaskStatus::Failed {
            query.push_str(", completed_at = ?");
        }

        query.push_str(" WHERE id = ?");

        let now = Utc::now().to_rfc3339();

        if status == TaskStatus::Running {
            sqlx::query(&query)
                .bind(status.as_str())
                .bind(&now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        } else if status == TaskStatus::Completed || status == TaskStatus::Failed {
            sqlx::query(&query)
                .bind(status.as_str())
                .bind(&now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
                .bind(status.as_str())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Mark task as failed with error
    pub async fn fail(&self, id: &str, error: &str) -> Result<()> {
        sqlx::query(
            "UPDATE tasks SET status = ?, completed_at = ?, error = ? WHERE id = ?"
        )
        .bind(TaskStatus::Failed.as_str())
        .bind(Utc::now().to_rfc3339())
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all tasks
    pub async fn list(&self) -> Result<Vec<Task>> {
        let rows = sqlx::query(
            r#"
            SELECT id, command, status, model, created_at, started_at, completed_at, error
            FROM tasks
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| Task {
            id: row.get("id"),
            command: row.get("command"),
            status: TaskStatus::from_str(row.get("status")).unwrap(),
            model: row.get("model"),
            created_at: DateTime::parse_from_rfc3339(row.get("created_at")).unwrap().with_timezone(&Utc),
            started_at: row.get::<Option<String>, _>("started_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            completed_at: row.get::<Option<String>, _>("completed_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            error: row.get("error"),
        }).collect())
    }

    /// Get task by ID
    pub async fn get(&self, id: &str) -> Result<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT id, command, status, model, created_at, started_at, completed_at, error
            FROM tasks
            WHERE id = ?
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Task {
            id: row.get("id"),
            command: row.get("command"),
            status: TaskStatus::from_str(row.get("status")).unwrap(),
            model: row.get("model"),
            created_at: DateTime::parse_from_rfc3339(row.get("created_at")).unwrap().with_timezone(&Utc),
            started_at: row.get::<Option<String>, _>("started_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            completed_at: row.get::<Option<String>, _>("completed_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            error: row.get("error"),
        }))
    }

    /// Clear completed tasks
    pub async fn clear_completed(&self) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM tasks WHERE status IN (?, ?)"
        )
        .bind(TaskStatus::Completed.as_str())
        .bind(TaskStatus::Cancelled.as_str())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
