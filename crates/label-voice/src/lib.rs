//! Voice command capture abstractions and natural-language intent parser
//! for Label-Agent.

pub mod capture;
pub mod error;
pub mod parser;

pub use crate::capture::{PcmConfig, StaticUtteranceQueue, UtteranceSource};
pub use crate::error::VoiceError;
pub use crate::parser::parse_spoken_command;
