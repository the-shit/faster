//! Text-to-speech using macOS say command

use anyhow::Result;
use std::process::Command;

pub struct MacOSTTS {
    voice: String,
    rate: u32,
}

impl MacOSTTS {
    pub fn new(voice: impl Into<String>, rate: u32) -> Self {
        Self {
            voice: voice.into(),
            rate,
        }
    }

    /// Speak text
    pub fn speak(&self, text: &str) -> Result<()> {
        Command::new("say")
            .arg("-v")
            .arg(&self.voice)
            .arg("-r")
            .arg(self.rate.to_string())
            .arg(text)
            .status()?;

        Ok(())
    }

    /// Speak text asynchronously (non-blocking)
    pub fn speak_async(&self, text: &str) -> Result<()> {
        Command::new("say")
            .arg("-v")
            .arg(&self.voice)
            .arg("-r")
            .arg(self.rate.to_string())
            .arg(text)
            .spawn()?;

        Ok(())
    }

    /// Check if TTS is available
    pub fn is_available() -> bool {
        Command::new("say")
            .arg("-v")
            .arg("?")
            .stdout(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// List available voices
    pub fn list_voices() -> Result<Vec<String>> {
        let output = Command::new("say")
            .arg("-v")
            .arg("?")
            .output()?;

        let voices: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                line.split_whitespace()
                    .next()
                    .map(|s| s.to_string())
            })
            .collect();

        Ok(voices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        assert!(MacOSTTS::is_available());
    }

    #[test]
    fn test_list_voices() {
        let voices = MacOSTTS::list_voices().unwrap();
        assert!(!voices.is_empty());
        println!("Available voices: {}", voices.len());
    }
}
