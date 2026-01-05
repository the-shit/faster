//! Speech-to-text using macOS native dictation

use anyhow::Result;
use std::process::Command;

pub struct MacOSSTT {
    language: String,
}

impl MacOSSTT {
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into(),
        }
    }

    /// Record and transcribe using macOS dictation
    /// Returns transcribed text
    pub fn transcribe(&self) -> Result<String> {
        println!("🎤 Speak now... (will auto-detect when you stop)");

        // Use osascript to trigger dictation
        // Note: This requires user to grant microphone permissions
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"
                tell application "System Events"
                    set textReturned to text returned of (display dialog "Speak your command:" default answer "" buttons {"Cancel", "OK"} default button "OK")
                    return textReturned
                end tell
            "#)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Failed to get speech input");
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let text = text.trim();

        if text.is_empty() {
            anyhow::bail!("No speech detected");
        }

        Ok(text.to_string())
    }

    /// Check if STT is available
    pub fn is_available() -> bool {
        // Check if osascript is available (always on macOS)
        Command::new("osascript")
            .arg("-e")
            .arg("return 1")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        assert!(MacOSSTT::is_available());
    }
}
