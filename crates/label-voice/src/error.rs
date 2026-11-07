//! Error types for voice capture and intent parsing.

use thiserror::Error;

/// Errors that can occur during audio capture or voice intent parsing.
#[non_exhaustive]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VoiceError {
    /// Utterance was empty or contained only whitespace.
    #[error("spoken utterance was empty")]
    EmptyUtterance,

    /// Utterance could not be mapped to a known command intent.
    #[error("unrecognized voice command intent: {utterance}")]
    UnrecognizedIntent {
        /// The raw utterance text that could not be parsed.
        utterance: String,
    },

    /// A required command parameter or identifier is missing.
    #[error("missing required identifier in voice command: {context}")]
    MissingIdentifier {
        /// Description of the missing parameter or context.
        context: &'static str,
    },

    /// Audio hardware or stream capture failure.
    #[error("audio capture failure: {message}")]
    AudioCapture {
        /// Detailed failure message from the audio subsystem.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_formatting() {
        let empty = VoiceError::EmptyUtterance;
        assert_eq!(empty.to_string(), "spoken utterance was empty");

        let unrecognized = VoiceError::UnrecognizedIntent {
            utterance: "do something unknown".to_owned(),
        };
        assert_eq!(
            unrecognized.to_string(),
            "unrecognized voice command intent: do something unknown"
        );

        let missing = VoiceError::MissingIdentifier {
            context: "box identifier expected after 'box'",
        };
        assert_eq!(
            missing.to_string(),
            "missing required identifier in voice command: box identifier expected after 'box'"
        );

        let audio = VoiceError::AudioCapture {
            message: "ALSA device busy".to_owned(),
        };
        assert_eq!(audio.to_string(), "audio capture failure: ALSA device busy");
    }

    #[test]
    fn test_error_traits() {
        let err1 = VoiceError::EmptyUtterance;
        let err2 = err1.clone();
        assert_eq!(err1, err2);
        assert_eq!(format!("{err1:?}"), "EmptyUtterance");
    }
}
