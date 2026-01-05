//! Intent processing layer
//!
//! Extracts structured data from messy speech and forces into deterministic schema

pub mod schema;
pub mod processor;

pub use schema::{Command, Intent, IntentExtractionResult, AmbiguityResolution};
pub use processor::IntentProcessor;

// TODO: Implement ensemble module
// pub mod ensemble;
// pub use ensemble::EnsembleValidator;
