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

        let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", db_path)).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_queue() -> TaskQueue {
        // Use in-memory database for tests (faster and no permission issues)
        TaskQueue::new(":memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_task_status_conversions() {
        assert_eq!(TaskStatus::Queued.as_str(), "queued");
        assert_eq!(TaskStatus::Running.as_str(), "running");
        assert_eq!(TaskStatus::Completed.as_str(), "completed");
        assert_eq!(TaskStatus::Failed.as_str(), "failed");
        assert_eq!(TaskStatus::Cancelled.as_str(), "cancelled");

        assert_eq!(TaskStatus::from_str("queued"), Some(TaskStatus::Queued));
        assert_eq!(TaskStatus::from_str("running"), Some(TaskStatus::Running));
        assert_eq!(TaskStatus::from_str("invalid"), None);
    }

    #[tokio::test]
    async fn test_queue_creation() {
        let queue = create_test_queue().await;
        let tasks = queue.list().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_enqueue_and_dequeue() {
        let queue = create_test_queue().await;

        // Enqueue task
        let id = queue.enqueue("Run tests", Some("sonnet".to_string())).await.unwrap();
        assert_eq!(id.len(), 8);

        // Dequeue task
        let task = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(task.command, "Run tests");
        assert_eq!(task.status, TaskStatus::Queued);
        assert_eq!(task.model, Some("sonnet".to_string()));
    }

    #[tokio::test]
    async fn test_dequeue_empty_queue() {
        let queue = create_test_queue().await;
        let task = queue.dequeue().await.unwrap();
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_dequeue_returns_oldest_queued_task() {
        let queue = create_test_queue().await;

        // Enqueue tasks
        let id1 = queue.enqueue("Task 1", None).await.unwrap();
        queue.enqueue("Task 2", None).await.unwrap();
        queue.enqueue("Task 3", None).await.unwrap();

        // Dequeue returns oldest queued task (doesn't remove it)
        let task = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Queued);

        // Dequeue again - returns same task since status wasn't updated
        let task2 = queue.dequeue().await.unwrap().unwrap();
        assert_eq!(task.id, task2.id);

        // Mark first task as running
        queue.update_status(&id1, TaskStatus::Running).await.unwrap();

        // Now dequeue returns the second task
        let task3 = queue.dequeue().await.unwrap().unwrap();
        assert_ne!(task3.id, id1);
        assert_eq!(task3.command, "Task 2");
    }

    #[tokio::test]
    async fn test_update_status_to_running() {
        let queue = create_test_queue().await;
        let id = queue.enqueue("Test task", None).await.unwrap();

        queue.update_status(&id, TaskStatus::Running).await.unwrap();

        let task = queue.get(&id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Running);
        assert!(task.started_at.is_some());
        assert!(task.completed_at.is_none());
    }

    #[tokio::test]
    async fn test_update_status_to_completed() {
        let queue = create_test_queue().await;
        let id = queue.enqueue("Test task", None).await.unwrap();

        queue.update_status(&id, TaskStatus::Running).await.unwrap();
        queue.update_status(&id, TaskStatus::Completed).await.unwrap();

        let task = queue.get(&id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.started_at.is_some());
        assert!(task.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_fail_task() {
        let queue = create_test_queue().await;
        let id = queue.enqueue("Test task", None).await.unwrap();

        queue.fail(&id, "Something went wrong").await.unwrap();

        let task = queue.get(&id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error, Some("Something went wrong".to_string()));
        assert!(task.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let queue = create_test_queue().await;

        queue.enqueue("Task 1", None).await.unwrap();
        queue.enqueue("Task 2", Some("opus".to_string())).await.unwrap();

        let tasks = queue.list().await.unwrap();
        assert_eq!(tasks.len(), 2);

        // Ordered by created_at DESC
        assert_eq!(tasks[0].command, "Task 2");
        assert_eq!(tasks[1].command, "Task 1");
    }

    #[tokio::test]
    async fn test_get_task_by_id() {
        let queue = create_test_queue().await;
        let id = queue.enqueue("Find me", Some("haiku".to_string())).await.unwrap();

        let task = queue.get(&id).await.unwrap().unwrap();
        assert_eq!(task.id, id);
        assert_eq!(task.command, "Find me");
        assert_eq!(task.model, Some("haiku".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_task() {
        let queue = create_test_queue().await;
        let task = queue.get("nonexistent").await.unwrap();
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_clear_completed() {
        let queue = create_test_queue().await;

        let id1 = queue.enqueue("Task 1", None).await.unwrap();
        let id2 = queue.enqueue("Task 2", None).await.unwrap();
        let id3 = queue.enqueue("Task 3", None).await.unwrap();

        queue.update_status(&id1, TaskStatus::Completed).await.unwrap();
        queue.update_status(&id2, TaskStatus::Cancelled).await.unwrap();
        // id3 stays queued

        let cleared = queue.clear_completed().await.unwrap();
        assert_eq!(cleared, 2);

        let tasks = queue.list().await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, id3);
    }

    #[tokio::test]
    async fn test_task_serialization() {
        let task = Task {
            id: "test123".to_string(),
            command: "Run tests".to_string(),
            status: TaskStatus::Running,
            model: Some("sonnet".to_string()),
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: None,
            error: None,
        };

        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("test123"));
        assert!(json.contains("Run tests"));
        assert!(json.contains("running"));

        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, task.id);
        assert_eq!(deserialized.command, task.command);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let queue = create_test_queue().await;

        // Spawn multiple enqueue operations concurrently
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let queue = queue.clone();
                tokio::spawn(async move {
                    queue.enqueue(&format!("Task {}", i), None).await.unwrap()
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let tasks = queue.list().await.unwrap();
        assert_eq!(tasks.len(), 10);
    }
}

impl Clone for TaskQueue {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
