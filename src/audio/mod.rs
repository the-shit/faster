//! Audio layer
//!
//! Handles speech-to-text and text-to-speech

pub mod stt;
pub mod tts;

pub use stt::MacOSSTT;
pub use tts::MacOSTTS;
