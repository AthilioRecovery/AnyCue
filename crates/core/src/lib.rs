//! Platform-independent mixing, dynamics and conversion utilities.
//! No dependency on Tauri, Windows, I/O or the UI.

/// Linear gain, equivalent to -inf..+6 dB for the range 0..2.
pub fn gain_from_percent(percent: f32) -> f32 {
    percent.clamp(0.0, 200.0) / 100.0
}

/// Transparent below -1 dBFS and smoothly saturated above that threshold.
/// Non-finite samples are converted to silence before they reach an output device.
pub fn soft_limit(sample: f32) -> f32 {
    if !sample.is_finite() {
        return 0.0;
    }
    const THRESHOLD: f32 = 0.891_250_9;
    let magnitude = sample.abs();
    if magnitude <= THRESHOLD {
        return sample;
    }
    let compressed =
        THRESHOLD + (1.0 - THRESHOLD) * ((magnitude - THRESHOLD) / (1.0 - THRESHOLD)).tanh();
    sample.signum() * compressed.min(1.0)
}

/// Add a pair of stereo samples with independently configured gains.
/// Both microphone and sound clips are assumed to be in [-1, 1].
pub fn mix_stereo(
    microphone: [f32; 2],
    sound: [f32; 2],
    mic_gain: f32,
    sound_gain: f32,
) -> [f32; 2] {
    [
        soft_limit(microphone[0] * mic_gain + sound[0] * sound_gain),
        soft_limit(microphone[1] * mic_gain + sound[1] * sound_gain),
    ]
}

/// RMS for visual metering, deliberately outside the realtime callback.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let squares: f64 = samples.iter().map(|s| f64::from(*s) * f64::from(*s)).sum();
    (squares / samples.len() as f64).sqrt() as f32
}

/// Convert signed PCM samples to normalized f32.
pub fn pcm16_to_f32(value: i16) -> f32 {
    value as f32 / 32768.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silent_inputs_produce_silence() {
        assert_eq!(mix_stereo([0.0; 2], [0.0; 2], 1.0, 1.0), [0.0; 2]);
    }
    #[test]
    fn large_mix_is_bounded() {
        for v in [-100.0, -1.0, 0.0, 1.0, 100.0] {
            assert!(mix_stereo([v; 2], [v; 2], 2.0, 2.0)
                .iter()
                .all(|x| x.is_finite() && x.abs() <= 1.0));
        }
    }
    #[test]
    fn limiter_is_transparent_below_threshold() {
        assert_eq!(soft_limit(0.5), 0.5);
    }
    #[test]
    fn limiter_is_odd_and_bounded() {
        for v in [0.01, 0.5, 1.0, 50.0] {
            assert!((soft_limit(v) + soft_limit(-v)).abs() < 1e-6);
            assert!(soft_limit(v).abs() <= 1.0);
        }
    }
    #[test]
    fn nan_cannot_poison_audio() {
        assert_eq!(soft_limit(f32::NAN), 0.0);
    }
    #[test]
    fn convert_and_gain() {
        assert_eq!(gain_from_percent(300.0), 2.0);
        assert_eq!(gain_from_percent(-1.0), 0.0);
        assert!(pcm16_to_f32(32767) < 1.0);
    }
    #[test]
    fn meter() {
        assert_eq!(rms(&[]), 0.0);
        assert!((rms(&[1.0, -1.0]) - 1.0).abs() < 1e-5);
    }
}
