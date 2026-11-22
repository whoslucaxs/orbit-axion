use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};
type Pixels = Result<(u32, u32, Vec<u8>), String>;
pub struct Icons {
    loaded: HashMap<String, slint::Image>,
    pending: HashSet<String>,
    tx: Option<SyncSender<String>>,
    rx: Receiver<(String, Pixels)>,
    thread: Option<JoinHandle<()>>,
}
impl Icons {
    pub fn new(wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<String>(32);
        let (result, results) = mpsc::channel();
        let thread = thread::spawn(move || {
            while let Ok(path) = rx.recv() {
                let pixels = (|| {
                    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024
                    {
                        return Err("Ícone excede 16 MB".into());
                    }
                    let mut reader = image::ImageReader::open(&path).map_err(|e| e.to_string())?;
                    let mut limits = image::Limits::default();
                    limits.max_image_width = Some(4096);
                    limits.max_image_height = Some(4096);
                    limits.max_alloc = Some(64 * 1024 * 1024);
                    reader.limits(limits);
                    let pixels = reader
                        .decode()
                        .map_err(|e| e.to_string())?
                        .thumbnail(128, 128)
                        .to_rgba8();
                    Ok((pixels.width(), pixels.height(), pixels.into_raw()))
                })();
                if result.send((path, pixels)).is_err() {
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
    pub fn get(&mut self, path: &str) -> slint::Image {
        if let Some(image) = self.loaded.get(path) {
            return image.clone();
        }
        if self.pending.insert(path.to_string())
            && !self
                .tx
                .as_ref()
                .is_some_and(|tx| tx.try_send(path.to_string()).is_ok())
        {
            self.pending.remove(path);
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
