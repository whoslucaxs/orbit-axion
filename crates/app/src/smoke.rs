use crate::{Ring, hwnd, monitor_scale};
use orbit_windows::{self as native, Point};
use slint::ComponentHandle;
use std::time::{Duration, Instant};
use windows::Win32::{Foundation::RECT, UI::WindowsAndMessaging::*};

/// Exercises real Slint/Win32 windows without hooks, injected input or audio writes.
pub struct Smoke {
    started: Instant,
    opened: Option<(Instant, usize, Point, i32)>,
}
impl Smoke {
    pub fn new() -> Self {
        Self {
            started: Instant::now(),
            opened: None,
        }
    }
    pub fn tick(&mut self, raw: Option<isize>, ring: &Ring) -> Option<Result<(), String>> {
        if self.started.elapsed() > Duration::from_secs(10) {
            return Some(Err("Timeout preparando overlay Slint".into()));
        }
        let raw = raw?;
        if let Some((opened, origin, center, size)) = self.opened {
            if opened.elapsed() < Duration::from_millis(250) {
                return None;
            }
            let mut rect = RECT::default();
            let result = (|| {
                unsafe {
                    GetWindowRect(hwnd(raw), &mut rect).map_err(|e| e.to_string())?;
                }
                let ex = unsafe { GetWindowLongPtrW(hwnd(raw), GWL_EXSTYLE) } as u32;
                if !unsafe { IsWindowVisible(hwnd(raw)) }.as_bool() {
                    return Err("Overlay não ficou visível".into());
                }
                if ex & WS_EX_NOACTIVATE.0 == 0 || ex & WS_EX_TOPMOST.0 == 0 {
                    return Err(format!("Estilos inválidos: 0x{ex:x}"));
                }
                if native::foreground() != origin {
                    return Err("Overlay alterou a janela em primeiro plano".into());
                }
                if rect.left != center.x - size / 2
                    || rect.top != center.y - size / 2
                    || rect.right - rect.left != size
                    || rect.bottom - rect.top != size
                {
                    return Err(format!(
                        "Geometria inesperada: {rect:?}; centro {center:?}, tamanho {size}"
                    ));
                }
                println!(
                    "SMOKE OK: overlay visível, NOACTIVATE, TOPMOST, centro exato, foco preservado; {size}px"
                );
                Ok(())
            })();
            let _ = ring.hide();
            return Some(result);
        }
        let center = match native::cursor() {
            Ok(p) => p,
            Err(e) => return Some(Err(e.to_string())),
        };
        let origin = native::foreground();
        let size = (360.0 * monitor_scale(center)).round() as i32;
        if let Err(e) = ring.show() {
            return Some(Err(e.to_string()));
        }
        if let Err(e) = native::show_overlay(hwnd(raw), center, size) {
            return Some(Err(e.to_string()));
        }
        self.opened = Some((Instant::now(), origin, center, size));
        None
    }
}
