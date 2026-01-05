//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct AudioConfig {
    #[serde(default = "default_input_device")]
    pub input_device: String,

    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    #[serde(default = "default_stt_provider")]
    pub provider: String,

    #[serde(default = "default_language")]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    #[serde(default = "default_tts_provider")]
    pub provider: String,

    #[serde(default = "default_voice")]
    pub voice: String,

    #[serde(default = "default_rate")]
    pub rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentConfig {
    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,

    #[serde(default = "default_ensemble_size")]
    pub ensemble_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationConfig {
    #[serde(default = "default_mode")]
    pub mode: String,

    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeConfig {
    #[serde(default = "default_local_db")]
    pub local_db: PathBuf,

    pub sync_endpoint: Option<String>,

    #[serde(default = "default_sync_mode")]
    pub sync_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    #[serde(default = "default_cli_path")]
    pub cli_path: String,

    #[serde(default = "default_claude_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
