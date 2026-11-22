//! Windows-only probes. Hooks are opt-in and never execute actions in callbacks.
#![cfg(windows)]
pub mod services;

use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Instant,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::Result,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn cursor() -> Result<Point> {
    let mut p = POINT::default();
    unsafe {
        GetCursorPos(&mut p)?;
    }
    Ok(Point { x: p.x, y: p.y })
}

pub fn foreground() -> usize {
    unsafe { GetForegroundWindow().0 as usize }
}

pub fn scale_at(point: Point) -> Result<f64> {
    unsafe {
        let monitor = MonitorFromPoint(
            POINT {
                x: point.x,
                y: point.y,
            },
            MONITOR_DEFAULTTONEAREST,
        );
        let (mut x, mut y) = (96, 96);
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y)?;
        Ok(x as f64 / 96.0)
    }
}

pub fn enable_per_monitor_dpi() {
    // A manifest takes precedence; an already initialized process may reject this.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

pub fn prepare_overlay(hwnd: HWND) -> Result<()> {
    unsafe {
        windows::Win32::UI::Shell::SetWindowSubclass(hwnd, Some(overlay_proc), 0x4f524249, 0)
            .ok()?;
        SetLastError(WIN32_ERROR(0));
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let previous = SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            style | (WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0) as isize,
        );
        if previous == 0 && GetLastError().0 != 0 {
            return Err(windows::core::Error::from_thread());
        }
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED,
        )?;
    }
    Ok(())
}

// Winit rewrites extended styles when visibility/DPI changes. Preserve our own-window contract.
unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    message: u32,
    w: WPARAM,
    l: LPARAM,
    id: usize,
    _: usize,
) -> LRESULT {
    use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    if message == WM_STYLECHANGING && w.0 as isize == GWL_EXSTYLE.0 as isize {
        let styles = unsafe { &mut *(l.0 as *mut STYLESTRUCT) };
        styles.styleNew =
            (styles.styleNew | WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0) & !WS_EX_APPWINDOW.0;
    }
    if message == WM_MOUSEACTIVATE {
        return LRESULT(MA_NOACTIVATE as isize);
    }
    if message == WM_NCDESTROY {
        unsafe {
            let _ = RemoveWindowSubclass(hwnd, Some(overlay_proc), id);
        }
    }
    unsafe { DefSubclassProc(hwnd, message, w, l) }
}

pub fn show_overlay(hwnd: HWND, center: Point, diameter: i32) -> Result<()> {
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            center.x - diameter / 2,
            center.y - diameter / 2,
            diameter,
            diameter,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
    }
}

pub fn hide_overlay(hwnd: HWND) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

pub fn acrylic_probe(hwnd: HWND) -> Result<()> {
    let kind = DWMSBT_TRANSIENTWINDOW.0;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            (&kind as *const i32).cast(),
            size_of::<i32>() as u32,
        )
    }
}

#[derive(Clone, Debug)]
pub enum Input {
    Press {
        point: Point,
        foreground: usize,
        at: Instant,
    },
    Release,
    Learned(u32),
    ForegroundChanged,
    ImeUnavailable,
    Select(Point),
    Outside,
    Wheel {
        point: Point,
        delta: i16,
    },
    Escape,
    Character(char),
    Text(String),
    Backspace,
    Navigate(i32),
    Enter,
}

pub struct Shared {
    pub ready: AtomicBool,
    pub active: AtomicBool,
    pub search: AtomicBool,
    pub button: AtomicU32,
    pub origin: AtomicUsize,
    pub center_x: AtomicI32,
    pub center_y: AtomicI32,
    pub radius: AtomicI32,
    pub allowed_origin: AtomicUsize,
    learning_until: AtomicU64,
    heartbeat: AtomicU64,
    clock: Instant,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            ready: AtomicBool::new(false),
            active: AtomicBool::new(false),
            search: AtomicBool::new(false),
            button: AtomicU32::new(1),
            origin: AtomicUsize::new(0),
            center_x: AtomicI32::new(0),
            center_y: AtomicI32::new(0),
            radius: AtomicI32::new(180),
            allowed_origin: AtomicUsize::new(usize::MAX),
            learning_until: AtomicU64::new(0),
            heartbeat: AtomicU64::new(0),
            clock: Instant::now(),
        }
    }
}

impl Shared {
    pub fn learn_button(&self) {
        self.learning_until.store(
            self.clock.elapsed().as_millis() as u64 + 10_000,
            Ordering::Release,
        );
    }
    pub fn beat(&self) {
        self.heartbeat
            .store(self.clock.elapsed().as_millis() as u64, Ordering::Relaxed);
    }
    fn healthy(&self) -> bool {
        (self.clock.elapsed().as_millis() as u64)
            .saturating_sub(self.heartbeat.load(Ordering::Relaxed))
            < 250
    }
    fn menu_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
            && self.healthy()
            && foreground() == self.origin.load(Ordering::Relaxed)
    }
    fn contains(&self, p: Point) -> bool {
        (p.x - self.center_x.load(Ordering::Relaxed)).abs() <= self.radius.load(Ordering::Relaxed)
            && (p.y - self.center_y.load(Ordering::Relaxed)).abs()
                <= self.radius.load(Ordering::Relaxed)
    }
}

struct HookContext {
    shared: Arc<Shared>,
    tx: SyncSender<Input>,
    activation_down: bool,
    left_consumed: bool,
    keys: [bool; 256],
    preheld: [bool; 256],
    wake: Arc<dyn Fn() + Send + Sync>,
    dead: Option<char>,
}
thread_local! { static CONTEXT: RefCell<Option<HookContext>> = const { RefCell::new(None) }; }

impl HookContext {
    fn send(&self, event: Input) -> bool {
        if self.tx.try_send(event).is_ok() {
            (self.wake)();
            true
        } else {
            self.shared.active.store(false, Ordering::Release);
            self.shared.ready.store(false, Ordering::Release);
            false
        }
    }
}

unsafe extern "system" fn mouse_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        // Windows guarantees MSLLHOOKSTRUCT for HC_ACTION in this callback.
        let data = unsafe { &*(l.0 as *const MSLLHOOKSTRUCT) };
        let point = Point {
            x: data.pt.x,
            y: data.pt.y,
        };
        let consumed = CONTEXT.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let Some(c) = borrow.as_mut() else {
                return false;
            };
            let message = w.0 as u32;
            let button = match message {
                WM_MBUTTONDOWN | WM_MBUTTONUP => Some(0),
                WM_XBUTTONDOWN | WM_XBUTTONUP => Some(data.mouseData >> 16),
                _ => None,
            };
            if let Some(button) = button
                && matches!(message, WM_MBUTTONDOWN | WM_XBUTTONDOWN)
                && c.shared.ready.load(Ordering::Acquire)
                && c.shared.healthy()
                && c.shared.learning_until.load(Ordering::Acquire)
                    > c.shared.clock.elapsed().as_millis() as u64
            {
                c.shared.learning_until.store(0, Ordering::Release);
                c.shared.button.store(button, Ordering::Release);
                c.activation_down = c.send(Input::Learned(button));
                return c.activation_down;
            }
            if button == Some(c.shared.button.load(Ordering::Relaxed)) {
                if message == WM_MBUTTONUP || message == WM_XBUTTONUP {
                    if c.activation_down {
                        c.activation_down = false;
                        c.send(Input::Release);
                        return true;
                    }
                } else if c.shared.ready.load(Ordering::Acquire)
                    && c.shared.healthy()
                    && (c.shared.allowed_origin.load(Ordering::Relaxed) == 0
                        || c.shared.allowed_origin.load(Ordering::Relaxed) == foreground())
                {
                    if !c.shared.active.load(Ordering::Acquire) {
                        c.dead = None;
                        for vk in 1..256 {
                            c.preheld[vk] = unsafe { GetAsyncKeyState(vk as i32) } < 0;
                        }
                    }
                    c.activation_down = c.send(Input::Press {
                        point,
                        foreground: foreground(),
                        at: Instant::now(),
                    });
                    return c.activation_down;
                }
            }
            if message == WM_LBUTTONUP && c.left_consumed {
                c.left_consumed = false;
                return true;
            }
            if c.shared.menu_active() {
                if message == WM_MOUSEWHEEL && c.shared.contains(point) {
                    return c.send(Input::Wheel {
                        point,
                        delta: (data.mouseData >> 16) as i16,
                    });
                }
                if message == WM_LBUTTONDOWN {
                    if c.shared.contains(point) {
                        c.left_consumed = c.send(Input::Select(point));
                        return c.left_consumed;
                    }
                    c.send(Input::Outside);
                } else if message == WM_RBUTTONDOWN {
                    c.send(Input::Outside);
                }
            }
            false
        });
        if consumed {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, w, l) }
}

// A consumed key-down owns repeats and key-up even after the ring closes.
fn key_guard(up: bool, active: bool, preheld: bool, captured: &mut bool) -> Option<bool> {
    if up {
        return Some(std::mem::take(captured));
    }
    if !active {
        return Some(*captured);
    }
    if preheld && !*captured {
        return Some(false);
    }
    None
}

unsafe extern "system" fn keyboard_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let data = unsafe { &*(l.0 as *const KBDLLHOOKSTRUCT) };
        let consumed = CONTEXT.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let Some(c) = borrow.as_mut() else {
                return false;
            };
            let key = data.vkCode as usize;
            if key >= 256 {
                return false;
            }
            let up = w.0 as u32 == WM_KEYUP || w.0 as u32 == WM_SYSKEYUP;
            if up {
                c.preheld[key] = false;
            }
            if let Some(consumed) =
                key_guard(up, c.shared.menu_active(), c.preheld[key], &mut c.keys[key])
            {
                return consumed;
            }

            let event = if key == VK_ESCAPE.0 as usize {
                Some(Input::Escape)
            } else if key == VK_UP.0 as usize {
                Some(Input::Navigate(-1))
            } else if key == VK_DOWN.0 as usize {
                Some(Input::Navigate(1))
            } else if key == VK_RETURN.0 as usize {
                Some(Input::Enter)
            } else if c.shared.search.load(Ordering::Relaxed) {
                let altgr = unsafe { GetAsyncKeyState(VK_RMENU.0 as i32) } < 0;
                if !altgr
                    && (unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) } < 0
                        || unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0)
                {
                    return false;
                }
                if key == VK_BACK.0 as usize {
                    c.dead = None;
                    Some(Input::Backspace)
                } else {
                    let mut state = [0u8; 256];
                    for vk in [
                        VK_SHIFT, VK_LSHIFT, VK_RSHIFT, VK_CONTROL, VK_MENU, VK_RMENU,
                    ] {
                        if unsafe { GetAsyncKeyState(vk.0 as i32) } < 0 {
                            state[vk.0 as usize] = 0x80;
                        }
                    }
                    state[VK_CAPITAL.0 as usize] =
                        (unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1) as u8;
                    let tid = unsafe { GetWindowThreadProcessId(GetForegroundWindow(), None) };
                    let layout = unsafe { GetKeyboardLayout(tid) };
                    if unsafe { windows::Win32::UI::Input::Ime::ImmIsIME(layout) }.as_bool()
                        && (key == VK_SPACE.0 as usize
                            || (0x30..=0x5a).contains(&key)
                            || (0x60..=0x6f).contains(&key)
                            || (0xba..=0xe2).contains(&key))
                    {
                        c.send(Input::ImeUnavailable);
                        c.keys[key] = true;
                        return true;
                    }
                    let mut buffer = [0u16; 8];
                    let count = unsafe {
                        ToUnicodeEx(
                            data.vkCode,
                            data.scanCode,
                            &state,
                            &mut buffer,
                            4,
                            Some(layout),
                        )
                    };
                    if count < 0 {
                        c.dead = char::from_u32(buffer[0] as u32);
                        c.keys[key] = true;
                        return true;
                    } else if count > 0 {
                        use unicode_normalization::UnicodeNormalization;
                        let mut text =
                            String::from_utf16_lossy(&buffer[..(count as usize).min(buffer.len())]);
                        if let Some(dead) = c.dead.take() {
                            let combining = match dead {
                                '´' => '\u{301}',
                                '`' => '\u{300}',
                                '^' => '\u{302}',
                                '~' => '\u{303}',
                                '¨' => '\u{308}',
                                _ => dead,
                            };
                            text = format!("{text}{combining}").nfc().collect();
                        }
                        if text.chars().any(|c| !c.is_control()) {
                            Some(Input::Text(text))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            } else {
                None
            };
            if let Some(event) = event {
                c.keys[key] |= c.send(event);
                return c.keys[key];
            }
            false
        });
        if consumed {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, w, l) }
}

unsafe extern "system" fn foreground_hook(
    _: windows::Win32::UI::Accessibility::HWINEVENTHOOK,
    _: u32,
    window: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    CONTEXT.with(|cell| {
        let borrowed = cell.borrow();
        if let Some(c) = borrowed.as_ref()
            && c.shared.active.load(Ordering::Acquire)
            && window.0 as usize != c.shared.origin.load(Ordering::Relaxed)
        {
            c.shared.active.store(false, Ordering::Release);
            c.send(Input::ForegroundChanged);
        }
    });
}

pub struct Hooks {
    thread: Option<JoinHandle<()>>,
    id: u32,
    pub shared: Arc<Shared>,
}

impl Hooks {
    pub fn start(
        button: u32,
        search: bool,
    ) -> std::result::Result<(Self, Receiver<Input>), String> {
        Self::start_with_wake(button, search, Arc::new(|| {}))
    }
    pub fn start_with_wake(
        button: u32,
        search: bool,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> std::result::Result<(Self, Receiver<Input>), String> {
        if button > 2 {
            return Err("Botão inválido: use 0 (meio), 1 (Mouse 4) ou 2 (Mouse 5)".into());
        }
        let shared = Arc::new(Shared::default());
        shared.button.store(button, Ordering::Relaxed);
        shared.search.store(search, Ordering::Relaxed);
        let copy = shared.clone();
        let (tx, rx) = mpsc::sync_channel(128);
        let (startup_tx, startup_rx) = mpsc::sync_channel(1);
        let thread = thread::spawn(move || unsafe {
            let result = (|| -> Result<(HHOOK, HHOOK)> {
                let module = GetModuleHandleW(None)?;
                let mouse =
                    SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), Some(HINSTANCE(module.0)), 0)?;
                match SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(keyboard_hook),
                    Some(HINSTANCE(module.0)),
                    0,
                ) {
                    Ok(keyboard) => Ok((mouse, keyboard)),
                    Err(e) => {
                        let _ = UnhookWindowsHookEx(mouse);
                        Err(e)
                    }
                }
            })();
            let (mouse, keyboard) = match result {
                Ok(handles) => handles,
                Err(e) => {
                    let _ = startup_tx.send(Err(e.to_string()));
                    return;
                }
            };
            let foreground = windows::Win32::UI::Accessibility::SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(foreground_hook),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            if foreground.0.is_null() {
                let _ = UnhookWindowsHookEx(keyboard);
                let _ = UnhookWindowsHookEx(mouse);
                let _ = startup_tx.send(Err("Não foi possível monitorar o foreground".into()));
                return;
            }
            let mut msg = MSG::default();
            let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
            CONTEXT.with(|c| {
                *c.borrow_mut() = Some(HookContext {
                    shared: copy,
                    tx,
                    activation_down: false,
                    left_consumed: false,
                    keys: [false; 256],
                    preheld: [false; 256],
                    wake,
                    dead: None,
                })
            });
            let _ = startup_tx.send(Ok(GetCurrentThreadId()));
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = windows::Win32::UI::Accessibility::UnhookWinEvent(foreground);
            let _ = UnhookWindowsHookEx(keyboard);
            let _ = UnhookWindowsHookEx(mouse);
            CONTEXT.with(|c| *c.borrow_mut() = None);
        });
        let id = startup_rx.recv().map_err(|e| e.to_string())??;
        Ok((
            Self {
                thread: Some(thread),
                id,
                shared,
            },
            rx,
        ))
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        self.shared.ready.store(false, Ordering::Release);
        self.shared.active.store(false, Ordering::Release);
        let _ = unsafe { PostThreadMessageW(self.id, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn captured_key_never_leaks_repeat_or_release_after_closing() {
        let mut captured = true;
        assert_eq!(key_guard(false, false, false, &mut captured), Some(true));
        assert_eq!(key_guard(true, false, false, &mut captured), Some(true));
        assert!(!captured);
        assert_eq!(key_guard(false, false, false, &mut captured), Some(false));
    }
    #[test]
    fn key_pressed_before_opening_keeps_its_original_pair() {
        let mut captured = false;
        assert_eq!(key_guard(false, true, true, &mut captured), Some(false));
        assert_eq!(key_guard(true, true, true, &mut captured), Some(false));
        assert_eq!(key_guard(false, true, false, &mut captured), None);
    }
}
