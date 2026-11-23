use orbit_core::model::IconFilters;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    path: String,
    filters: IconFilters,
}
fn apply_filters(pixels: &mut [u8], filters: IconFilters) {
    let grayscale = filters.grayscale as f32 / 100.0;
    let contrast = filters.contrast as f32 / 100.0;
    let brightness = filters.brightness as f32 * 255.0 / 100.0;
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let luminance =
            0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
        for channel in &mut pixel[..3] {
            let value = *channel as f32 * (1.0 - grayscale) + luminance * grayscale;
            let value = ((value - 127.5) * contrast + 127.5 + brightness).clamp(0.0, 255.0);
            *channel = if filters.invert { 255.0 - value } else { value }.round() as u8;
        }
    }
}
type Pixels = Result<(u32, u32, Vec<u8>), String>;
pub struct Icons {
    loaded: HashMap<Key, slint::Image>,
    pending: HashSet<Key>,
    tx: Option<SyncSender<Key>>,
    rx: Receiver<(Key, Pixels)>,
    thread: Option<JoinHandle<()>>,
}
impl Icons {
    pub fn new(wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Key>(32);
        let (result, results) = mpsc::channel();
        let thread = thread::spawn(move || {
            while let Ok(key) = rx.recv() {
                let path = &key.path;
                let pixels = (|| {
                    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024
                    {
                        return Err("Ícone excede 16 MB".into());
                    }
                    let mut reader = image::ImageReader::open(path).map_err(|e| e.to_string())?;
                    let mut limits = image::Limits::default();
                    limits.max_image_width = Some(4096);
                    limits.max_image_height = Some(4096);
                    limits.max_alloc = Some(64 * 1024 * 1024);
                    reader.limits(limits);
                    let mut pixels = reader
                        .decode()
                        .map_err(|e| e.to_string())?
                        .thumbnail(128, 128)
                        .to_rgba8();
                    apply_filters(pixels.as_mut(), key.filters);
                    Ok((pixels.width(), pixels.height(), pixels.into_raw()))
                })();
                if result.send((key, pixels)).is_err() {
                    break;
                }
                wake();
            }
        });
        Self {
            loaded: HashMap::new(),
            pending: HashSet::new(),
            tx: Some(tx),
            rx: results,
            thread: Some(thread),
        }
    }
    pub fn get(&mut self, path: &str, filters: IconFilters) -> slint::Image {
        let key = Key {
            path: path.into(),
            filters,
        };
        if let Some(image) = self.loaded.get(&key) {
            return image.clone();
        }
        if self.pending.insert(key.clone())
            && !self
                .tx
                .as_ref()
                .is_some_and(|tx| tx.try_send(key.clone()).is_ok())
        {
            self.pending.remove(&key);
        }
        slint::Image::default()
    }
    pub fn poll(&mut self) -> bool {
        let mut updated = false;
        while let Ok((path, result)) = self.rx.try_recv() {
            self.pending.remove(&path);
            let img = match result {
                Ok((w, h, pixels)) => slint::Image::from_rgba8(
                    slint::SharedPixelBuffer::clone_from_slice(&pixels, w, h),
                ),
                Err(_) => slint::Image::default(),
            };
            if self.loaded.len() >= 256 {
                self.loaded.clear();
            }
            self.loaded.insert(path, img);
            updated = true;
        }
        updated
    }
}
impl Drop for Icons {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filters_preserve_alpha_and_original_colors_and_clamp_channels() {
        let original = [255, 0, 0, 128, 20, 80, 150, 0];
        let mut pixels = original;
        apply_filters(&mut pixels, IconFilters::default());
        assert_eq!(pixels, original);
        apply_filters(
            &mut pixels,
            IconFilters {
                grayscale: 100,
                ..Default::default()
            },
        );
        assert_eq!(&pixels[..4], &[54, 54, 54, 128]);
        assert_eq!(pixels[7], 0);
        let mut inverted = original;
        apply_filters(
            &mut inverted,
            IconFilters {
                invert: true,
                ..Default::default()
            },
        );
        assert_eq!(&inverted[..4], &[0, 255, 255, 128]);
        let mut gray = original;
        apply_filters(
            &mut gray,
            IconFilters {
                contrast: 0,
                ..Default::default()
            },
        );
        assert_eq!(&gray[..4], &[128, 128, 128, 128]);
        for (brightness, expected) in [(-100, 0), (100, 255)] {
            let mut result = original;
            apply_filters(
                &mut result,
                IconFilters {
                    brightness,
                    ..Default::default()
                },
            );
            assert_eq!(&result[..4], &[expected, expected, expected, 128]);
        }
        assert_ne!(
            Key {
                path: "same.png".into(),
                filters: IconFilters::default()
            },
            Key {
                path: "same.png".into(),
                filters: IconFilters {
                    invert: true,
                    ..Default::default()
                }
            }
        );
    }
}
