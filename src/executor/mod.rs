//! Execution layer
//!
//! Sends commands to Claude Code CLI (inherits folder context)

pub mod claude;

pub use claude::ClaudeExecutor;
