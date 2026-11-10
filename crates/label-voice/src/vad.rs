//! Voice Activity Detection (VAD) energy utilities and WAV audio encoder.
//!
//! Provides integer RMS audio energy calculation for voice activity gating and
//! pure-Rust 16-bit mono PCM to WAV audio container encoding.

/// Computes root-mean-square (RMS) energy level of 16-bit signed PCM audio
/// samples using pure-Rust integer arithmetic.
///
/// Returns `0` if `samples` is empty or all silence.
#[must_use]
pub fn compute_rms_s16le(samples: &[i16]) -> u16 {
    if samples.is_empty() {
        return 0;
    }
    let sum_sq: u128 = samples
        .iter()
        .map(|&s| {
            let val = s as i64;
            (val * val) as u128
        })
        .sum();
    let mean_sq = sum_sq / (samples.len() as u128);
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    {
        (mean_sq as f64).sqrt() as u16
    }
}

/// Encodes mono 16-bit little-endian PCM samples into a RIFF/WAVE container.
///
/// Returns a 44-byte header followed by little-endian 16-bit PCM sample bytes.
#[must_use]
pub fn encode_pcm_s16le_to_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * (u32::from(bits_per_sample) / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_len = (samples.len() * 2) as u32;
    let riff_chunk_size = 36 + data_len;

    let mut wav = Vec::with_capacity(44 + (samples.len() * 2));

    // 0..4: RIFF chunk descriptor
    wav.extend_from_slice(b"RIFF");
    // 4..8: Overall file size minus 8 bytes
    wav.extend_from_slice(&riff_chunk_size.to_le_bytes());
    // 8..12: Format header
    wav.extend_from_slice(b"WAVE");
    // 12..16: Format subchunk header
    wav.extend_from_slice(b"fmt ");
    // 16..20: Subchunk1 size (16 for PCM)
    wav.extend_from_slice(&16u32.to_le_bytes());
    // 20..22: Audio format (1 = PCM)
    wav.extend_from_slice(&1u16.to_le_bytes());
    // 22..24: Number of channels (1 = mono)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    // 24..28: Sample rate in Hz
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    // 28..32: Byte rate
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    // 32..34: Block alignment
    wav.extend_from_slice(&block_align.to_le_bytes());
    // 34..36: Bits per sample
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    // 36..40: Data subchunk header
    wav.extend_from_slice(b"data");
    // 40..44: Subchunk2 size (data payload length)
    wav.extend_from_slice(&data_len.to_le_bytes());

    // PCM samples payload (s16le)
    for &sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_rms_silence_vs_nonzero() {
        assert_eq!(compute_rms_s16le(&[]), 0);
        assert_eq!(compute_rms_s16le(&[0, 0, 0, 0]), 0);

        let non_zero_samples = [1000, -1000, 1000, -1000];
        let rms = compute_rms_s16le(&non_zero_samples);
        assert_eq!(rms, 1000);

        let dynamic_samples = [3000, 4000];
        let dynamic_rms = compute_rms_s16le(&dynamic_samples);
        assert_eq!(dynamic_rms, 3535);
    }

    #[test]
    fn test_encode_pcm_s16le_to_wav_structure() {
        let sample_rate = 16_000;
        let samples: Vec<i16> = vec![0, 500, -500, 1000, -1000];
        let wav = encode_pcm_s16le_to_wav(&samples, sample_rate);

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(wav.len(), 44 + samples.len() * 2);

        // Verify data subchunk header and size
        assert_eq!(&wav[36..40], b"data");
        let payload_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(payload_size, (samples.len() * 2) as u32);
    }
}
