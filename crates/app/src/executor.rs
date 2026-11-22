use orbit_core::model::{Action, Slot};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::*,
        System::Power::SetSuspendState,
        UI::{Input::KeyboardAndMouse::*, Shell::ShellExecuteW, WindowsAndMessaging::*},
    },
    core::{PCWSTR, w},
};

pub struct Executor {
    tx: Option<SyncSender<(Slot, usize)>>,
    pub replies: Receiver<Result<(), String>>,
    pub cancel: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    busy: Arc<AtomicBool>,
}
impl Executor {
    pub fn start(audio: SyncSender<crate::audio_worker::Request>) -> Self {
        let (tx, rx) = mpsc::sync_channel::<(Slot, usize)>(1);
        let (reply_tx, replies) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let busy = Arc::new(AtomicBool::new(false));
        let worker_busy = busy.clone();
        let thread = thread::spawn(move || {
            while let Ok((slot, origin)) = rx.recv() {
                let result = (|| {
                    if slot.confirm {
                        let result = rfd::MessageDialog::new()
                            .set_title("Orbit Axion")
                            .set_description(format!("Executar “{}”?", slot.name))
                            .set_buttons(rfd::MessageButtons::YesNo)
                            .show();
                        if result != rfd::MessageDialogResult::Yes {
                            return Err("Ação cancelada".into());
                        }
                    }
                    execute(&slot.action, origin, &flag, Instant::now(), 0, &audio)
                })();
                worker_busy.store(false, Ordering::Release);
                if reply_tx.send(result).is_err() {
                    break;
                }
            }
        });
        Self {
            tx: Some(tx),
            replies,
            cancel,
            thread: Some(thread),
            busy,
        }
    }
    pub fn run(&self, slot: Slot, origin: usize) -> bool {
        if self.busy.swap(true, Ordering::AcqRel) {
            return false;
        }
        self.cancel.store(false, Ordering::Release);
        let queued = self
            .tx
            .as_ref()
            .is_some_and(|tx| tx.try_send((slot, origin)).is_ok());
        if !queued {
            self.busy.store(false, Ordering::Release);
        }
        queued
    }
}
impl Drop for Executor {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn shell(target: &str, args: &str, elevated: bool) -> Result<(), String> {
    if target.contains('\0') || args.contains('\0') {
        return Err("NUL em caminho/argumento".into());
    }
    let verb = if elevated { w!("runas") } else { w!("open") };
    let code = unsafe {
        ShellExecuteW(
            None,
            verb,
            PCWSTR(wide(target).as_ptr()),
            PCWSTR(wide(args).as_ptr()),
            None,
            SW_SHOWNORMAL,
        )
    };
    if code.0 as isize <= 32 {
        Err(format!(
            "Windows não abriu o destino (código {})",
            code.0 as isize
        ))
    } else {
        Ok(())
    }
}
pub fn quote_arg(s: &str) -> String {
    let mut out = String::from("\"");
    let mut slash = 0;
    for c in s.chars() {
        if c == '\\' {
            slash += 1;
        } else {
            if c == '"' {
                out.push_str(&"\\".repeat(slash * 2 + 1));
            } else {
                out.push_str(&"\\".repeat(slash));
            }
            slash = 0;
            out.push(c);
        }
    }
    out.push_str(&"\\".repeat(slash * 2));
    out.push('"');
    out
}
fn keyboard(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                dwExtraInfo: orbit_windows::INJECTED_INPUT_MARKER,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                ..Default::default()
            },
        },
    }
}
fn send(inputs: &[INPUT]) -> Result<(), String> {
    if unsafe { SendInput(inputs, size_of::<INPUT>() as i32) } == inputs.len() as u32 {
        Ok(())
    } else {
        Err("O Windows bloqueou a entrada (possível diferença de integridade/UIPI)".into())
    }
}
pub fn shortcut_keys(keys: &str) -> Result<Vec<u16>, String> {
    keys.split('+')
        .map(|s| {
            let key = s.trim().to_uppercase();
            match key.as_str() {
                "CTRL" | "CONTROL" => Ok(VK_CONTROL.0),
                "SHIFT" => Ok(VK_SHIFT.0),
                "ALT" => Ok(VK_MENU.0),
                "WIN" => Ok(VK_LWIN.0),
                "ENTER" => Ok(VK_RETURN.0),
                "TAB" => Ok(VK_TAB.0),
                "ESC" => Ok(VK_ESCAPE.0),
                "SPACE" => Ok(VK_SPACE.0),
                "DELETE" => Ok(VK_DELETE.0),
                "BACKSPACE" => Ok(VK_BACK.0),
                "LEFT" => Ok(VK_LEFT.0),
                "RIGHT" => Ok(VK_RIGHT.0),
                "UP" => Ok(VK_UP.0),
                "DOWN" => Ok(VK_DOWN.0),
                _ if key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() => {
                    Ok(key.as_bytes()[0] as u16)
                }
                _ => {
                    if let Some(f) = key
                        .strip_prefix('F')
                        .and_then(|s| s.parse::<u16>().ok())
                        .filter(|f| (1..=24).contains(f))
                    {
                        Ok(VK_F1.0 + f - 1)
                    } else {
                        Err(format!("Tecla não suportada: {key}"))
                    }
                }
            }
        })
        .collect()
}
fn require_origin(origin: usize) -> Result<HWND, String> {
    if origin == 0 || orbit_windows::foreground() != origin {
        return Err("Janela original mudou; ação de entrada cancelada".into());
    }
    Ok(HWND(origin as *mut _))
}
fn execute(
    action: &Action,
    origin: usize,
    cancel: &AtomicBool,
    start: Instant,
    depth: usize,
    audio: &SyncSender<crate::audio_worker::Request>,
) -> Result<(), String> {
    if cancel.load(Ordering::Acquire) || start.elapsed() > Duration::from_secs(600) {
        return Err("Execução cancelada ou tempo limite atingido".into());
    }
    if depth > 16 {
        return Err("Macro excedeu limite de aninhamento".into());
    }
    match action {
        Action::Open { target } => shell(target, "", false),
        Action::Launch {
            program,
            arguments,
            elevated,
        } => {
            let args = arguments
                .iter()
                .map(|a| quote_arg(a))
                .collect::<Vec<_>>()
                .join(" ");
            if std::path::Path::new(program)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("ps1"))
            {
                shell(
                    "powershell.exe",
                    &format!("-NoProfile -File {} {args}", quote_arg(program)),
                    *elevated,
                )
            } else {
                shell(program, &args, *elevated)
            }
        }
        Action::Shortcut { keys } => {
            require_origin(origin)?;
            let keys = shortcut_keys(keys)?;
            if keys.is_empty() {
                return Err("Atalho vazio".into());
            }
            let mut inputs: Vec<_> = keys.iter().map(|&k| keyboard(k, false)).collect();
            inputs.extend(keys.iter().rev().map(|&k| keyboard(k, true)));
            send(&inputs)
        }
        Action::Text { text } => {
            require_origin(origin)?;
            for chunk in text.encode_utf16().collect::<Vec<_>>().chunks(128) {
                if cancel.load(Ordering::Acquire) {
                    return Err("Cancelado".into());
                }
                require_origin(origin)?;
                let inputs: Vec<_> = chunk
                    .iter()
                    .flat_map(|&u| {
                        [
                            INPUT {
                                r#type: INPUT_KEYBOARD,
                                Anonymous: INPUT_0 {
                                    ki: KEYBDINPUT {
                                        wScan: u,
                                        dwExtraInfo: orbit_windows::INJECTED_INPUT_MARKER,
                                        dwFlags: KEYEVENTF_UNICODE,
                                        ..Default::default()
                                    },
                                },
                            },
                            INPUT {
                                r#type: INPUT_KEYBOARD,
                                Anonymous: INPUT_0 {
                                    ki: KEYBDINPUT {
                                        wScan: u,
                                        dwExtraInfo: orbit_windows::INJECTED_INPUT_MARKER,
                                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                                        ..Default::default()
                                    },
                                },
                            },
                        ]
                    })
                    .collect();
                send(&inputs)?;
            }
            Ok(())
        }
        Action::Media { command } => {
            let key = match command.as_str() {
                "play_pause" => VK_MEDIA_PLAY_PAUSE,
                "next" => VK_MEDIA_NEXT_TRACK,
                "previous" => VK_MEDIA_PREV_TRACK,
                "stop" => VK_MEDIA_STOP,
                _ => return Err("Mídia inválida".into()),
            };
            send(&[keyboard(key.0, false), keyboard(key.0, true)])
        }
        Action::Window { command } => {
            let window = require_origin(origin)?;
            unsafe {
                match command.as_str() {
                    "maximize_restore" => {
                        let _ = ShowWindow(
                            window,
                            if IsZoomed(window).as_bool() {
                                SW_RESTORE
                            } else {
                                SW_MAXIMIZE
                            },
                        );
                    }
                    "minimize" => {
                        let _ = ShowWindow(window, SW_MINIMIZE);
                    }
                    "close" => {
                        PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0))
                            .map_err(|e| e.to_string())?;
                    }
                    _ => return Err("Janela inválida".into()),
                }
            }
            Ok(())
        }
        Action::System { command } => match command.as_str() {
            "lock" => unsafe {
                windows::Win32::System::Shutdown::LockWorkStation().map_err(|e| e.to_string())
            },
            "sleep" => unsafe {
                if SetSuspendState(false, false, false) {
                    Ok(())
                } else {
                    Err("O Windows recusou a suspensão".into())
                }
            },
            "shutdown" => shell("shutdown.exe", "/s /t 0", false),
            "restart" => shell("shutdown.exe", "/r /t 0", false),
            _ => Err("Sistema inválido".into()),
        },
        Action::Macro { steps } => {
            for step in steps {
                let until = Instant::now() + Duration::from_millis(step.delay_ms);
                while Instant::now() < until {
                    if cancel.load(Ordering::Acquire) {
                        return Err("Macro cancelada".into());
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                execute(&step.action, origin, cancel, start, depth + 1, audio)?;
            }
            Ok(())
        }
        Action::Volume {
            provider,
            target,
            bus,
            endpoint,
            ..
        } => {
            let key = crate::audio_worker::AudioKey {
                potato: provider == "voicemeeter_potato",
                target: *target,
                bus: *bus,
                endpoint: endpoint.clone(),
            };
            let (tx, rx) = mpsc::sync_channel(1);
            audio
                .try_send(crate::audio_worker::Request::MacroMute(key, tx))
                .map_err(|_| "Áudio ocupado")?;
            rx.recv_timeout(Duration::from_secs(2))
                .map_err(|_| "Tempo limite do áudio")?
        }
        _ => Err("Ação exige controlador da interface".into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_windows_arguments_and_validates_shortcuts() {
        assert_eq!(quote_arg("a b"), "\"a b\"");
        assert_eq!(quote_arg("x\\"), "\"x\\\\\"");
        assert_eq!(shortcut_keys("CTRL+SHIFT+F12").unwrap().len(), 3);
        assert!(shortcut_keys("CTRL+UNKNOWN").is_err());
    }
}
