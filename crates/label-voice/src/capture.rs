//! Audio PCM configuration and utterance stream capture abstractions.
//!
//! Provides hardware PCM audio parameters tuned for Le Potato ALSA capture
//! and deterministic utterance queues for voice agent testing and offline
//! processing.

use std::collections::VecDeque;

use crate::error::VoiceError;

/// Linear PCM audio capture format configuration.
///
/// Encapsulates sample rate, channel count, and frame duration for low-latency
/// audio stream buffers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmConfig {
    sample_rate_hz: u32,
    channels: u8,
    frame_ms: u16,
}

impl PcmConfig {
    /// Creates a new PCM configuration with custom capture parameters.
    #[must_use]
    pub const fn new(sample_rate_hz: u32, channels: u8, frame_ms: u16) -> Self {
        Self {
            sample_rate_hz,
            channels,
            frame_ms,
        }
    }

    /// Default PCM configuration tuned for Libre Computer AML-S905X-CC ALSA.
    ///
    /// Configured for 16 kHz mono audio with 20 ms frames (320 samples per frame),
    /// optimal for speech recognition engines.
    #[must_use]
    pub const fn lepotato_alsa_default() -> Self {
        Self {
            sample_rate_hz: 16_000,
            channels: 1,
            frame_ms: 20,
        }
    }

    /// Returns the capture sampling rate in Hertz.
    #[must_use]
    pub const fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    /// Returns the number of audio channels (e.g. 1 for mono, 2 for stereo).
    #[must_use]
    pub const fn channels(&self) -> u8 {
        self.channels
    }

    /// Returns the audio frame duration in milliseconds.
    #[must_use]
    pub const fn frame_ms(&self) -> u16 {
        self.frame_ms
    }

    /// Computes the number of audio samples per frame per channel.
    #[must_use]
    pub const fn samples_per_frame(&self) -> usize {
        (self.sample_rate_hz as usize * self.frame_ms as usize) / 1000
    }
}

impl Default for PcmConfig {
    fn default() -> Self {
        Self::lepotato_alsa_default()
    }
}

/// Abstract streaming source yielding spoken natural language utterances.
pub trait UtteranceSource {
    /// Yields the next available spoken utterance string, or `None` if exhausted.
    ///
    /// # Errors
    ///
    /// Returns a [`VoiceError`] if audio stream acquisition or transcription fails.
    fn next_utterance(&mut self) -> Result<Option<String>, VoiceError>;
}

/// Deterministic in-memory utterance queue for testing and dry-run execution.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct StaticUtteranceQueue {
    utterances: VecDeque<String>,
}

impl StaticUtteranceQueue {
    /// Constructs a new queue initialized with a list of utterance strings.
    #[must_use]
    pub fn new(utterances: Vec<String>) -> Self {
        Self {
            utterances: VecDeque::from(utterances),
        }
    }

    /// Returns the number of remaining utterances in the queue.
    #[must_use]
    pub fn len(&self) -> usize {
        self.utterances.len()
    }

    /// Returns `true` if the utterance queue contains no remaining items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.utterances.is_empty()
    }

    /// Appends an utterance to the back of the queue.
    pub fn push_back(&mut self, utterance: impl Into<String>) {
        self.utterances.push_back(utterance.into());
    }
}

impl UtteranceSource for StaticUtteranceQueue {
    fn next_utterance(&mut self) -> Result<Option<String>, VoiceError> {
        Ok(self.utterances.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcm_config_defaults() {
        let config = PcmConfig::default();
        assert_eq!(config.sample_rate_hz(), 16_000);
        assert_eq!(config.channels(), 1);
        assert_eq!(config.frame_ms(), 20);
        assert_eq!(config.samples_per_frame(), 320);

        let lepotato = PcmConfig::lepotato_alsa_default();
        assert_eq!(config, lepotato);
    }

    #[test]
    fn test_pcm_config_custom() {
        let custom = PcmConfig::new(48_000, 2, 10);
        assert_eq!(custom.sample_rate_hz(), 48_000);
        assert_eq!(custom.channels(), 2);
        assert_eq!(custom.frame_ms(), 10);
        assert_eq!(custom.samples_per_frame(), 480);
    }

    #[test]
    fn test_static_utterance_queue_lifecycle() {
        let mut queue = StaticUtteranceQueue::new(vec![
            "print box label for BOX-042".to_string(),
            "query box BOX-042".to_string(),
        ]);

        assert_eq!(queue.len(), 2);
        assert!(!queue.is_empty());

        let first = queue
            .next_utterance()
            .expect("queue extraction should succeed");
        assert_eq!(first, Some("print box label for BOX-042".to_string()));
        assert_eq!(queue.len(), 1);

        queue.push_back("move box BOX-042 to S2");
        assert_eq!(queue.len(), 2);

        let second = queue
            .next_utterance()
            .expect("queue extraction should succeed");
        assert_eq!(second, Some("query box BOX-042".to_string()));

        let third = queue
            .next_utterance()
            .expect("queue extraction should succeed");
        assert_eq!(third, Some("move box BOX-042 to S2".to_string()));

        let empty = queue
            .next_utterance()
            .expect("exhausted queue returns None without error");
        assert_eq!(empty, None);
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }
}
