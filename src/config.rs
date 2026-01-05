//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub audio: AudioConfig,
    pub stt: SttConfig,
    pub tts: TtsConfig,
    pub intent: IntentConfig,
    pub confirmation: ConfirmationConfig,
    pub knowledge: KnowledgeConfig,
    pub claude: ClaudeConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    pub input_device: String,
    pub sample_rate: u32,
    pub vad_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SttConfig {
    pub provider: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TtsConfig {
    pub provider: String,
    pub voice: String,
    pub rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IntentConfig {
    pub model: String,
    pub confidence_threshold: f32,
    pub ensemble_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfirmationConfig {
    pub mode: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KnowledgeConfig {
    pub local_db: PathBuf,
    pub sync_endpoint: Option<String>,
    pub sync_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaudeConfig {
    pub cli_path: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    pub conduit_endpoint: Option<String>,
}

// Defaults
fn default_input_device() -> String {
    "default".to_string()
}

fn default_sample_rate() -> u32 {
    16000
}

fn default_vad_threshold() -> f32 {
    0.5
}

fn default_stt_provider() -> String {
    "macos-native".to_string()
}

fn default_language() -> String {
    "en-US".to_string()
}

fn default_tts_provider() -> String {
    "macos-native".to_string()
}

fn default_voice() -> String {
    "Samantha".to_string()
}

fn default_rate() -> u32 {
    200
}

fn default_model() -> String {
    "llama-3.2-3b-instruct".to_string()
}

fn default_confidence_threshold() -> f32 {
    0.80
}

fn default_ensemble_size() -> usize {
    3
}

fn default_mode() -> String {
    "smart".to_string()
}

fn default_timeout_ms() -> u64 {
    1000
}

fn default_local_db() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".faster")
        .join("knowledge.db")
}

fn default_cli_path() -> String {
    "claude".to_string()
}

fn default_claude_model() -> String {
    "sonnet".to_string()
}

fn default_sync_mode() -> String {
    "non-sensitive".to_string()
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device: default_input_device(),
            sample_rate: default_sample_rate(),
            vad_threshold: default_vad_threshold(),
        }
    }
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            provider: default_stt_provider(),
            language: default_language(),
        }
    }
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            provider: default_tts_provider(),
            voice: default_voice(),
            rate: default_rate(),
        }
    }
}

impl Default for IntentConfig {
    fn default() -> Self {
        Self {
            model: default_model(),
            confidence_threshold: default_confidence_threshold(),
            ensemble_size: default_ensemble_size(),
        }
    }
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            timeout_ms: default_timeout_ms(),
        }
    }
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            local_db: default_local_db(),
            sync_endpoint: None,
            sync_mode: default_sync_mode(),
        }
    }
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            cli_path: default_cli_path(),
            model: default_claude_model(),
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            conduit_endpoint: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio: AudioConfig {
                input_device: default_input_device(),
                sample_rate: default_sample_rate(),
                vad_threshold: default_vad_threshold(),
            },
            stt: SttConfig {
                provider: default_stt_provider(),
                language: default_language(),
            },
            tts: TtsConfig {
                provider: default_tts_provider(),
                voice: default_voice(),
                rate: default_rate(),
            },
            intent: IntentConfig {
                model: default_model(),
                confidence_threshold: default_confidence_threshold(),
                ensemble_size: default_ensemble_size(),
            },
            confirmation: ConfirmationConfig {
                mode: default_mode(),
                timeout_ms: default_timeout_ms(),
            },
            knowledge: KnowledgeConfig {
                local_db: default_local_db(),
                sync_endpoint: None,
                sync_mode: default_sync_mode(),
            },
            claude: ClaudeConfig {
                cli_path: default_cli_path(),
                model: default_claude_model(),
            },
            observability: ObservabilityConfig {
                conduit_endpoint: None,
            },
        }
    }
}

impl Config {
    /// Load from TOML file
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save to TOML file
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Get config path
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".faster")
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();

        // Audio defaults
        assert_eq!(config.audio.input_device, "default");
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.audio.vad_threshold, 0.5);

        // STT defaults
        assert_eq!(config.stt.provider, "macos-native");
        assert_eq!(config.stt.language, "en-US");

        // TTS defaults
        assert_eq!(config.tts.provider, "macos-native");
        assert_eq!(config.tts.voice, "Samantha");
        assert_eq!(config.tts.rate, 200);

        // Intent defaults
        assert_eq!(config.intent.model, "llama-3.2-3b-instruct");
        assert_eq!(config.intent.confidence_threshold, 0.80);
        assert_eq!(config.intent.ensemble_size, 3);

        // Confirmation defaults
        assert_eq!(config.confirmation.mode, "smart");
        assert_eq!(config.confirmation.timeout_ms, 1000);

        // Knowledge defaults
        assert!(config.knowledge.local_db.to_string_lossy().contains(".faster"));
        assert_eq!(config.knowledge.sync_endpoint, None);
        assert_eq!(config.knowledge.sync_mode, "non-sensitive");

        // Claude defaults
        assert_eq!(config.claude.cli_path, "claude");
        assert_eq!(config.claude.model, "sonnet");

        // Observability defaults
        assert_eq!(config.observability.conduit_endpoint, None);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        // Create and save config
        let mut config = Config::default();
        config.tts.voice = "Alex".to_string();
        config.intent.confidence_threshold = 0.85;

        config.save(&config_path).unwrap();

        // Load and verify
        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.tts.voice, "Alex");
        assert_eq!(loaded.intent.confidence_threshold, 0.85);
        assert_eq!(loaded.claude.cli_path, "claude");
    }

    #[test]
    fn test_config_load_invalid_path() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        let result = Config::load(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();

        assert!(toml_str.contains("input_device"));
        assert!(toml_str.contains("Samantha"));
        assert!(toml_str.contains("sonnet"));
    }

    #[test]
    fn test_config_deserialization_partial() {
        let toml_str = r#"
            [audio]
            sample_rate = 44100

            [claude]
            model = "opus"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();

        // Custom values
        assert_eq!(config.audio.sample_rate, 44100);
        assert_eq!(config.claude.model, "opus");

        // Defaults for missing fields
        assert_eq!(config.audio.input_device, "default");
        assert_eq!(config.tts.voice, "Samantha");
    }

    #[test]
    fn test_config_path() {
        let path = Config::path();
        assert!(path.to_string_lossy().contains(".faster"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_save_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let nested_path = dir.path().join("nested").join("deep").join("config.toml");

        let config = Config::default();
        config.save(&nested_path).unwrap();

        assert!(nested_path.exists());
    }
}
