//! Claude Code CLI integration

use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use colored::*;

/// Claude Code executor
pub struct ClaudeExecutor {
    cli_path: String,
    model: Option<String>,
}

impl ClaudeExecutor {
    /// Create new executor
    pub fn new(cli_path: impl Into<String>) -> Self {
        Self {
            cli_path: cli_path.into(),
            model: None,
        }
    }

    /// Set Claude model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Execute prompt in current directory context
    /// Claude Code automatically picks up folder context
    pub fn execute(&self, prompt: &str) -> Result<()> {
        let mut cmd = Command::new(&self.cli_path);

        // Add prompt as single argument
        cmd.arg(prompt);

        // Add model if specified
        if let Some(model) = &self.model {
            cmd.arg("--model").arg(model);
        }

        // Inherit stdio so output streams directly to terminal
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        // Execute in current directory (Claude picks up context)
        let status = cmd.status()
            .context("Failed to execute Claude CLI")?;

        if !status.success() {
            anyhow::bail!("Claude CLI exited with non-zero status");
        }

        Ok(())
    }

    /// Check if Claude CLI is available
    pub fn is_available() -> bool {
        Command::new("claude")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = ClaudeExecutor::new("claude");
        assert_eq!(executor.cli_path, "claude");
        assert_eq!(executor.model, None);
    }

    #[test]
    fn test_executor_with_model() {
        let executor = ClaudeExecutor::new("claude")
            .with_model("opus");

        assert_eq!(executor.model, Some("opus".to_string()));
    }

    #[test]
    fn test_is_available() {
        let available = ClaudeExecutor::is_available();
        println!("Claude CLI available: {}", available);
    }
}
