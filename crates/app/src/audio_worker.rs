use orbit_audio::{
    Level, VolumeProvider,
    voicemeeter::{Target, Voicemeeter},
    windows_audio::WindowsVolume,
};
use std::{
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioKey {
    pub potato: bool,
    pub target: u8,
    pub bus: bool,
    pub endpoint: Option<String>,
}
pub enum Request {
    Devices,
    Labels,
    MacroMute(AudioKey, SyncSender<Result<(), String>>),
    Read(AudioKey),
    Adjust(AudioKey, f32),
    Mute(AudioKey),
}
pub struct Reply {
    pub key: AudioKey,
    pub result: Result<Level, String>,
}
type ChannelLabels = Result<(Vec<String>, Vec<String>), String>;
pub struct AudioWorker {
    tx: Option<SyncSender<Request>>,
    pub replies: Receiver<Reply>,
    pub devices: Receiver<Result<Vec<(String, String)>, String>>,
    pub labels: Receiver<ChannelLabels>,
    thread: Option<JoinHandle<()>>,
}
impl AudioWorker {
    pub fn start(wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<Request>(64);
        let (reply_tx, replies) = mpsc::channel();
        let (device_tx, devices) = mpsc::channel();
        let (label_tx, labels) = mpsc::channel();
        let thread = thread::spawn(move || {
            let mut current = None;
            let mut controller: Option<Box<dyn VolumeProvider>> = None;
            let mut pending = None;
            while let Some(request) = pending.take().or_else(|| rx.recv().ok()) {
                let (key, mut delta, mute, ack) = match request {
                    Request::Labels => {
                        controller = None;
                        current = None;
                        let result =
                            Voicemeeter::connect(Target::Strip(0)).and_then(|v| v.labels());
                        let _ = label_tx.send(result);
                        wake();
                        continue;
                    }
                    Request::Devices => {
                        let _ = device_tx.send(orbit_audio::windows_audio::list_endpoints());
                        wake();
                        continue;
                    }
                    Request::Read(k) => (k, None, false, None),
                    Request::Adjust(k, d) => (k, Some(d), false, None),
                    Request::Mute(k) => (k, None, true, None),
                    Request::MacroMute(k, ack) => (k, None, true, Some(ack)),
                };
                if let Some(total) = &mut delta {
                    let end = Instant::now() + Duration::from_millis(12);
                    while Instant::now() < end {
                        match rx.recv_timeout(end.saturating_duration_since(Instant::now())) {
                            Ok(Request::Adjust(k, d)) if k == key => *total += d,
                            Ok(other) => {
                                pending = Some(other);
                                break;
                            }
                            Err(_) => break,
                        }
                    }
                }
                if current.as_ref() != Some(&key) || !key.potato {
                    controller = None;
                    current = Some(key.clone());
                }
                let result = (|| {
                    if controller.is_none() {
                        controller = Some(if key.potato {
                            Box::new(Voicemeeter::connect(if key.bus {
                                Target::Bus(key.target)
                            } else {
                                Target::Strip(key.target)
                            })?) as Box<dyn VolumeProvider>
                        } else {
                            Box::new(WindowsVolume::connect_endpoint(key.endpoint.as_deref())?)
                                as Box<dyn VolumeProvider>
                        });
                    }
                    let c = controller.as_ref().unwrap();
                    if mute {
                        c.toggle_mute()
                    } else if let Some(d) = delta {
                        c.adjust_level(if key.potato { d } else { d / 2.0 })
                    } else {
                        c.read_level()
                    }
                })();
                if result.is_err() {
                    controller = None;
                }
                if let Some(ack) = ack {
                    let _ = ack.send(result.as_ref().map(|_| ()).map_err(Clone::clone));
                }
                if reply_tx.send(Reply { key, result }).is_err() {
                    break;
                }
                wake();
            }
        });
        Self {
            tx: Some(tx),
            replies,
            devices,
            labels,
            thread: Some(thread),
        }
    }
    pub fn client(&self) -> SyncSender<Request> {
        self.tx.as_ref().expect("audio worker running").clone()
    }
    pub fn request(&self, r: Request) -> bool {
        self.tx.as_ref().is_some_and(|tx| tx.try_send(r).is_ok())
    }
}
impl Drop for AudioWorker {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
