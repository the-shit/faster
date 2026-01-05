//! Deterministic command schema
//!
//! Forces messy speech into rigid structure that Claude can execute

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The four intent categories Claude can handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Intent {
    /// Manage workflows, coordinate agents, spawn tasks
    Orchestrate,

    /// Search code, gather context, read documentation
    Research,

    /// Generate, edit, refactor code
    Code,

    /// Run tests, debug failures, fix issues
    Test,
}

impl Intent {
    /// Get all possible intents
    pub fn all() -> &'static [Intent] {
        &[
            Intent::Orchestrate,
            Intent::Research,
            Intent::Code,
            Intent::Test,
        ]
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Intent::Orchestrate => "Manage workflows, coordinate agents, spawn tasks",
            Intent::Research => "Search code, gather context, read documentation",
            Intent::Code => "Generate, edit, refactor code",
            Intent::Test => "Run tests, debug failures, fix issues",
        }
    }
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intent::Orchestrate => write!(f, "ORCHESTRATE"),
            Intent::Research => write!(f, "RESEARCH"),
            Intent::Code => write!(f, "CODE"),
            Intent::Test => write!(f, "TEST"),
        }
    }
}

/// Deterministic command structure
///
/// This is the rigid schema that local AI forces messy speech into.
/// Claude receives this and executes without ambiguity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Intent category (one of 4 options)
    pub intent: Intent,

    /// Clear, unambiguous directive for Claude
    /// Example: "Run test suite for authentication module"
    pub directive: String,

    /// Extracted entities (files, modules, functions, etc.)
    /// Example: ["authentication", "tests"]
    pub entities: Vec<String>,

    /// User context from knowledge system
    /// Example: {"current_module": "auth", "current_goal": "refactor auth"}
    pub context: HashMap<String, String>,

    /// Confidence score from local AI (0.0 - 1.0)
    pub confidence: f32,

    /// When this command was created
    pub created_at: DateTime<Utc>,
}

impl Command {
    /// Create new command
    pub fn new(
        intent: Intent,
        directive: impl Into<String>,
        entities: Vec<String>,
        confidence: f32,
    ) -> Self {
        Self {
            intent,
            directive: directive.into(),
            entities,
            context: HashMap::new(),
            confidence,
            created_at: Utc::now(),
        }
    }

    /// Add context key-value pair
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Add multiple context entries
    pub fn with_contexts(mut self, contexts: HashMap<String, String>) -> Self {
        self.context.extend(contexts);
        self
    }

    /// Check if confidence meets threshold
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }

    /// Convert to Claude prompt string
    pub fn to_claude_prompt(&self) -> String {
        let mut prompt = self.directive.clone();

        // Add context if available
        if !self.context.is_empty() {
            prompt.push_str("\n\nContext:");
            for (key, value) in &self.context {
                prompt.push_str(&format!("\n- {}: {}", key, value));
            }
        }

        prompt
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Result of intent extraction
#[derive(Debug, Clone)]
pub struct IntentExtractionResult {
    /// The extracted command
    pub command: Command,

    /// Raw transcript that was processed
    pub transcript: String,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Ambiguities that were resolved
    pub ambiguities_resolved: Vec<AmbiguityResolution>,
}

/// Record of an ambiguity that was resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguityResolution {
    /// Original ambiguous phrase
    pub from_phrase: String,

    /// Resolved entity/concept
    pub to_entity: String,

    /// Context that enabled resolution
    pub context: String,

    /// Confidence in resolution
    pub confidence: f32,
}

impl AmbiguityResolution {
    pub fn new(
        from_phrase: impl Into<String>,
        to_entity: impl Into<String>,
        context: impl Into<String>,
        confidence: f32,
    ) -> Self {
        Self {
            from_phrase: from_phrase.into(),
            to_entity: to_entity.into(),
            context: context.into(),
            confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new(
            Intent::Test,
            "Run test suite for authentication module",
            vec!["authentication".to_string(), "tests".to_string()],
            0.95,
        );

        assert_eq!(cmd.intent, Intent::Test);
        assert_eq!(cmd.directive, "Run test suite for authentication module");
        assert_eq!(cmd.entities.len(), 2);
        assert_eq!(cmd.confidence, 0.95);
    }

    #[test]
    fn test_command_with_context() {
        let cmd = Command::new(
            Intent::Code,
            "Implement payment module",
            vec!["payment".to_string()],
            0.90,
        )
        .with_context("current_goal", "build payment system")
        .with_context("current_module", "payment");

        assert_eq!(cmd.context.len(), 2);
        assert_eq!(cmd.context.get("current_goal").unwrap(), "build payment system");
    }

    #[test]
    fn test_confidence_threshold() {
        let cmd = Command::new(
            Intent::Test,
            "Run tests",
            vec![],
            0.85,
        );

        assert!(cmd.is_confident(0.80));
        assert!(!cmd.is_confident(0.90));
    }

    #[test]
    fn test_claude_prompt_generation() {
        let cmd = Command::new(
            Intent::Test,
            "Run test suite",
            vec![],
            0.95,
        )
        .with_context("module", "auth");

        let prompt = cmd.to_claude_prompt();
        assert!(prompt.contains("Run test suite"));
        assert!(prompt.contains("Context:"));
        assert!(prompt.contains("module: auth"));
    }

    #[test]
    fn test_intent_display() {
        assert_eq!(Intent::Orchestrate.to_string(), "ORCHESTRATE");
        assert_eq!(Intent::Code.to_string(), "CODE");
    }

    #[test]
    fn test_json_serialization() {
        let cmd = Command::new(
            Intent::Research,
            "Search for auth implementation",
            vec!["auth".to_string()],
            0.92,
        );

        let json = cmd.to_json().unwrap();
        assert!(json.contains("RESEARCH"));
        assert!(json.contains("Search for auth implementation"));
    }
}
