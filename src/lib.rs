//! Faster - Task queue and Claude Code integration
//!
//! Can be used as:
//! - CLI binary (`faster` command)
//! - Zellij plugin (displays queue in terminal)
//! - Rust library (embed in other tools)

pub mod audio;
pub mod bridge;
pub mod config;
pub mod executor;
pub mod intent;
pub mod knowledge;
pub mod queue;

#[cfg(feature = "plugin")]
pub mod plugin;

// Re-exports
pub use config::Config;
pub use queue::{TaskQueue, TaskStatus, Task};
pub use executor::ClaudeExecutor;
