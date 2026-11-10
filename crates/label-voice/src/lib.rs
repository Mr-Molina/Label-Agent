//! Voice command capture abstractions and natural-language intent parser
//! for Label-Agent.

pub mod capture;
pub mod error;
pub mod parser;
pub mod stt_client;
pub mod vad;

pub use crate::capture::{PcmConfig, StaticUtteranceQueue, UtteranceSource};
pub use crate::error::VoiceError;
pub use crate::parser::parse_spoken_command;
pub use crate::stt_client::{ApiKeySttClient, SpeechTranscriber, DEFAULT_STT_API_URL};
pub use vad::{compute_rms_s16le, encode_pcm_s16le_to_wav};
