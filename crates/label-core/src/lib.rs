//! Core domain types, 6-level location taxonomy, label geometry, and voice
//! command intents for Label-Agent.

pub mod error;
pub mod intent;
pub mod label_spec;
pub mod taxonomy;

pub use crate::error::CoreError;
pub use crate::intent::{LabelFormat, PrintMode, VoiceCommandIntent};
pub use crate::label_spec::{AssetLabelSpec, BoxLabelSpec, PageGeometry};
pub use crate::taxonomy::{LocationNode, LocationPath, TaxonomyLevel};
