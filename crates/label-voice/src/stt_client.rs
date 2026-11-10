//! Remote API-key Speech-to-Text (`STT`) client for thin ARM64 edge nodes.
//!
//! Encapsulates the [`SpeechTranscriber`] trait and [`ApiKeySttClient`],
//! allowing the Libre Computer AML-S905X-CC (`sbc-lepotato`) to offload WAV
//! audio transcription over HTTPS (`ureq` + `rustls`) using an API key.

use serde_json::json;
use serde_json::Value;

use crate::error::VoiceError;

/// Default Gemini `generateContent` endpoint for multimodal WAV transcription.
pub const DEFAULT_STT_API_URL: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

const B64_ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes a byte slice into standard RFC 4648 Base64 without external crates.
#[must_use]
pub fn encode_base64(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = if chunk.len() > 1 {
            u32::from(chunk[1])
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            u32::from(chunk[2])
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(char::from(B64_ALPHABET[((triple >> 18) & 0x3F) as usize]));
        out.push(char::from(B64_ALPHABET[((triple >> 12) & 0x3F) as usize]));
        if chunk.len() > 1 {
            out.push(char::from(B64_ALPHABET[((triple >> 6) & 0x3F) as usize]));
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(char::from(B64_ALPHABET[(triple & 0x3F) as usize]));
        } else {
            out.push('=');
        }
    }
    out
}

/// Trait for speech-to-text backends that transcribe 16 kHz mono WAV audio.
pub trait SpeechTranscriber {
    /// Transcribes a RIFF/WAVE audio buffer into a plain-text utterance.
    ///
    /// # Errors
    ///
    /// Returns [`VoiceError`] if the audio payload is empty or remote
    /// transcription fails.
    fn transcribe_wav(&self, wav_bytes: &[u8]) -> Result<String, VoiceError>;
}

/// Remote HTTPS Speech-to-Text client authenticated via an API key.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeySttClient {
    api_url: String,
    api_key: String,
}

impl ApiKeySttClient {
    /// Constructs a new [`ApiKeySttClient`] with the given endpoint and key.
    ///
    /// # Errors
    ///
    /// Returns [`VoiceError::AudioCapture`] if `api_url` or `api_key` is empty.
    pub fn new(api_url: impl Into<String>, api_key: impl Into<String>) -> Result<Self, VoiceError> {
        let api_url = api_url.into();
        let api_key = api_key.into();
        if api_url.trim().is_empty() {
            return Err(VoiceError::AudioCapture {
                message: "STT API URL must not be empty".to_owned(),
            });
        }
        if api_key.trim().is_empty() {
            return Err(VoiceError::AudioCapture {
                message: "STT API key must not be empty".to_owned(),
            });
        }
        Ok(Self { api_url, api_key })
    }

    /// Returns the configured STT API endpoint URL.
    #[must_use]
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Builds the JSON request body for Gemini multimodal WAV transcription.
    #[must_use]
    pub fn build_request_payload(wav_bytes: &[u8]) -> Value {
        let b64_audio = encode_base64(wav_bytes);
        json!({
            "contents": [{
                "parts": [
                    {
                        "text": concat!(
                            "Transcribe this spoken inventory command verbatim ",
                            "as plain text without punctuation."
                        )
                    },
                    {
                        "inline_data": {
                            "mime_type": "audio/wav",
                            "data": b64_audio
                        }
                    }
                ]
            }]
        })
    }

    /// Extracts the transcribed text from a Gemini `generateContent` JSON
    /// response or an OpenAI-style `{"text": "..."}` response.
    ///
    /// # Errors
    ///
    /// Returns [`VoiceError::AudioCapture`] if no transcript text field is
    /// present.
    pub fn extract_transcript_from_json(resp: &Value) -> Result<String, VoiceError> {
        if let Some(text) = resp.get("text").and_then(Value::as_str) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_owned());
            }
        }
        if let Some(text) = resp
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(Value::as_str)
        {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_owned());
            }
        }
        Err(VoiceError::AudioCapture {
            message: "STT response JSON did not contain a transcript".to_owned(),
        })
    }
}

impl SpeechTranscriber for ApiKeySttClient {
    fn transcribe_wav(&self, wav_bytes: &[u8]) -> Result<String, VoiceError> {
        if wav_bytes.len() <= 44 {
            return Err(VoiceError::AudioCapture {
                message: "WAV audio buffer is empty or missing PCM frames".to_owned(),
            });
        }
        let payload = Self::build_request_payload(wav_bytes);
        let response = ureq::post(&self.api_url)
            .set("Content-Type", "application/json")
            .set("x-goog-api-key", &self.api_key)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_string(&payload.to_string())
            .map_err(|err| VoiceError::AudioCapture {
                message: format!("STT HTTP request failed: {err}"),
            })?;

        let body_str = response
            .into_string()
            .map_err(|err| VoiceError::AudioCapture {
                message: format!("Failed to read STT response body: {err}"),
            })?;
        let json_val: Value =
            serde_json::from_str(&body_str).map_err(|err| VoiceError::AudioCapture {
                message: format!("Invalid STT JSON response: {err}"),
            })?;
        Self::extract_transcript_from_json(&json_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_base64_rfc4648_vectors() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(encode_base64(b"f"), "Zg==");
        assert_eq!(encode_base64(b"fo"), "Zm8=");
        assert_eq!(encode_base64(b"foo"), "Zm9v");
        assert_eq!(encode_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_api_key_stt_client_validation() {
        assert!(ApiKeySttClient::new("", "secret-key").is_err());
        assert!(ApiKeySttClient::new(DEFAULT_STT_API_URL, "   ").is_err());
        let client =
            ApiKeySttClient::new(DEFAULT_STT_API_URL, "valid-key").expect("valid client config");
        assert_eq!(client.api_url(), DEFAULT_STT_API_URL);
    }

    #[test]
    fn test_build_and_extract_transcript_payload() {
        let wav = vec![0_u8; 64];
        let payload = ApiKeySttClient::build_request_payload(&wav);
        assert_eq!(
            payload
                .pointer("/contents/0/parts/1/inline_data/mime_type")
                .and_then(Value::as_str),
            Some("audio/wav")
        );

        let gemini_resp = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "print box label for BOX-042\n"}]
                }
            }]
        });
        assert_eq!(
            ApiKeySttClient::extract_transcript_from_json(&gemini_resp).expect("extract gemini"),
            "print box label for BOX-042"
        );

        let whisper_resp = json!({"text": "what is inside box BOX-042"});
        assert_eq!(
            ApiKeySttClient::extract_transcript_from_json(&whisper_resp).expect("extract whisper"),
            "what is inside box BOX-042"
        );
    }
}
