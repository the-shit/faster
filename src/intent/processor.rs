//! Intent processor - translates messy human speech into deterministic Commands

use anyhow::Result;
use super::schema::{Command, Intent};
use std::collections::HashMap;
use chrono::Utc;

pub struct IntentProcessor {
    // TODO: Add Llama model for advanced processing
    confidence_threshold: f32,
}

impl IntentProcessor {
    pub fn new(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold,
        }
    }

    /// Process raw transcript into deterministic Command
    /// Uses pattern matching + keyword detection (MVP)
    /// TODO: Enhance with Llama 3.2 for complex cases
    pub fn process(&self, transcript: &str) -> Result<Command> {
        let transcript_lower = transcript.to_lowercase();

        // Detect intent
        let (intent, confidence) = self.detect_intent(&transcript_lower);

        // Extract entities
        let entities = self.extract_entities(&transcript_lower, &intent);

        // Clean directive
        let directive = self.clean_directive(transcript, &intent);

        // Build context
        let context = self.build_context(&transcript_lower);

        Ok(Command {
            intent,
            directive,
            entities,
            context,
            confidence,
            created_at: Utc::now(),
        })
    }

    /// Detect intent from transcript using pattern matching
    fn detect_intent(&self, text: &str) -> (Intent, f32) {
        // Orchestrate patterns
        if self.contains_any(text, &["run", "execute", "start", "launch", "deploy", "build"]) {
            return (Intent::Orchestrate, 0.85);
        }

        // Research patterns
        if self.contains_any(text, &["find", "search", "look", "where", "what", "show", "list"]) {
            return (Intent::Research, 0.85);
        }

        // Test patterns
        if self.contains_any(text, &["test", "debug", "fix", "check", "verify"]) {
            return (Intent::Test, 0.85);
        }

        // Code patterns
        if self.contains_any(text, &["create", "add", "write", "update", "refactor", "implement", "generate"]) {
            return (Intent::Code, 0.85);
        }

        // Default to Code with lower confidence
        (Intent::Code, 0.60)
    }

    /// Extract key entities from transcript
    fn extract_entities(&self, text: &str, intent: &Intent) -> Vec<String> {
        let mut entities = Vec::new();

        // Extract file patterns
        if text.contains(".rs") || text.contains(".php") || text.contains(".js") {
            let words: Vec<&str> = text.split_whitespace().collect();
            for word in words {
                if word.contains('.') && !word.ends_with('.') {
                    entities.push(word.to_string());
                }
            }
        }

        // Extract test patterns
        if *intent == Intent::Test {
            if text.contains("unit") {
                entities.push("unit".to_string());
            }
            if text.contains("integration") {
                entities.push("integration".to_string());
            }
        }

        // Extract code patterns
        if text.contains("class") || text.contains("function") || text.contains("method") {
            let words: Vec<&str> = text.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if *word == "class" || *word == "function" || *word == "method" {
                    // Look for "class for X" or "function for X" patterns
                    if let Some(&next) = words.get(i + 1) {
                        if next == "for" {
                            if let Some(name) = words.get(i + 2) {
                                entities.push(name.to_string());
                            }
                        } else {
                            entities.push(next.to_string());
                        }
                    }
                }
            }
        }

        entities
    }

    /// Clean directive by removing filler words
    fn clean_directive(&self, text: &str, _intent: &Intent) -> String {
        let filler_words = [
            "um", "uh", "like", "you know", "actually", "basically",
            "just", "please", "can you", "could you", "i want", "i need"
        ];

        let mut cleaned = text.to_string();
        for filler in &filler_words {
            cleaned = cleaned.replace(filler, "");
        }

        // Clean extra whitespace
        cleaned.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Build context map from transcript
    fn build_context(&self, text: &str) -> HashMap<String, String> {
        let mut context = HashMap::new();

        // Detect urgency
        if self.contains_any(text, &["urgent", "asap", "quick", "fast", "now"]) {
            context.insert("urgency".to_string(), "high".to_string());
        }

        // Detect scope
        if self.contains_any(text, &["all", "every", "entire", "whole"]) {
            context.insert("scope".to_string(), "broad".to_string());
        } else if self.contains_any(text, &["this", "that", "single", "one"]) {
            context.insert("scope".to_string(), "narrow".to_string());
        }

        context
    }

    /// Helper: Check if text contains any of the patterns
    fn contains_any(&self, text: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|p| text.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrate_intent() {
        let processor = IntentProcessor::new(0.80);
        let cmd = processor.process("run the tests").unwrap();
        assert_eq!(cmd.intent, Intent::Orchestrate);
        assert!(cmd.confidence >= 0.80);
    }

    #[test]
    fn test_research_intent() {
        let processor = IntentProcessor::new(0.80);
        let cmd = processor.process("find all the auth files").unwrap();
        assert_eq!(cmd.intent, Intent::Research);
    }

    #[test]
    fn test_code_intent() {
        let processor = IntentProcessor::new(0.80);
        let cmd = processor.process("create a new class for users").unwrap();
        assert_eq!(cmd.intent, Intent::Code);
        assert!(cmd.entities.contains(&"users".to_string()));
    }

    #[test]
    fn test_clean_directive() {
        let processor = IntentProcessor::new(0.80);
        let cmd = processor.process("um like can you just run the tests please").unwrap();
        assert_eq!(cmd.directive, "run the tests");
    }
}
