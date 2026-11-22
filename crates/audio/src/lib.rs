//! Thread-confined COM and Voicemeeter Remote API probes.
#![cfg(windows)]
pub mod voicemeeter;
pub mod windows_audio;

pub type AudioResult<T> = Result<T, String>;

#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub value: f32,
    pub muted: bool,
    pub unit: &'static str,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.1} {}{}",
            self.value,
            self.unit,
            if self.muted { " · MUTE" } else { "" }
        )
    }
}

pub trait VolumeProvider {
    fn read_level(&self) -> AudioResult<Level>;
    fn adjust_level(&self, steps: f32) -> AudioResult<Level>;
    fn toggle_mute(&self) -> AudioResult<Level>;
}

pub fn bounded_delta(current: f32, delta: f32, min: f32, max: f32) -> AudioResult<f32> {
    if !current.is_finite()
        || !delta.is_finite()
        || !min.is_finite()
        || !max.is_finite()
        || min > max
    {
        return Err("Nível, delta ou limites inválidos".into());
    }
    Ok((current + delta).clamp(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clamps_and_rejects_invalid_values() {
        assert_eq!(bounded_delta(99.0, 2.0, 0.0, 100.0), Ok(100.0));
        assert_eq!(bounded_delta(-59.0, -2.0, -60.0, 12.0), Ok(-60.0));
        assert!(bounded_delta(f32::NAN, 1.0, 0.0, 100.0).is_err());
        assert!(bounded_delta(1.0, 1.0, 100.0, 0.0).is_err());
    }
}
