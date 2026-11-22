use windows::{
    Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT},
    core::PCWSTR,
};
pub struct Feedback {
    waves: [Vec<u8>; 5],
}
impl Feedback {
    pub fn new(volume: u32) -> Self {
        Self {
            waves: [520.0, 380.0, 660.0, 880.0, 220.0]
                .map(|frequency| wave(frequency, volume.min(100) as f32 / 100.0)),
        }
    }
    pub fn play(&self, index: usize) {
        unsafe {
            let _ = PlaySoundW(
                PCWSTR(self.waves[index % 5].as_ptr().cast()),
                None,
                SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
            );
        }
    }
}
impl Drop for Feedback {
    fn drop(&mut self) {
        unsafe {
            let _ = PlaySoundW(PCWSTR::null(), None, SND_ASYNC);
        }
    }
}
fn wave(frequency: f32, volume: f32) -> Vec<u8> {
    let rate = 22050u32;
    let samples = 1102u32;
    let mut data = vec![];
    data.extend(b"RIFF");
    data.extend((36 + samples * 2).to_le_bytes());
    data.extend(b"WAVEfmt ");
    data.extend(16u32.to_le_bytes());
    data.extend(1u16.to_le_bytes());
    data.extend(1u16.to_le_bytes());
    data.extend(rate.to_le_bytes());
    data.extend((rate * 2).to_le_bytes());
    data.extend(2u16.to_le_bytes());
    data.extend(16u16.to_le_bytes());
    data.extend(b"data");
    data.extend((samples * 2).to_le_bytes());
    for i in 0..samples {
        let t = i as f32 / samples as f32;
        let envelope = (std::f32::consts::PI * t).sin().powi(2);
        let value = ((i as f32 * std::f32::consts::TAU * frequency / rate as f32).sin()
            * volume
            * 2500.0
            * envelope) as i16;
        data.extend(value.to_le_bytes());
    }
    data
}
