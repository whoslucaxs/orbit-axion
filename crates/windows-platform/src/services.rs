use super::*;
use std::path::Path;
use windows::{
    Win32::{
        Globalization::GetUserDefaultLocaleName,
        System::{Registry::*, Threading::*},
    },
    core::w,
};
pub struct Instance(HANDLE);
impl Instance {
    pub fn acquire() -> Result<Option<Self>> {
        unsafe {
            let h = CreateMutexW(None, false, w!("Local\\OrbitAxion.SingleInstance"))?;
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let _ = CloseHandle(h);
                if let Ok(window) = FindWindowW(None, w!("Orbit Axion")) {
                    let _ = ShowWindow(window, SW_RESTORE);
                    let _ = SetForegroundWindow(window);
                }
                return Ok(None);
            }
            Ok(Some(Self(h)))
        }
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
pub fn executable(window: usize) -> Option<String> {
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(HWND(window as *mut _), Some(&mut pid));
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut text = [0u16; 32768];
        let mut len = text.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(text.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(process);
        result
            .ok()
            .map(|_| String::from_utf16_lossy(&text[..len as usize]))
    }
}
pub fn light_theme() -> bool {
    let mut value = 0u32;
    let mut len = 4;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut len),
        )
        .is_ok()
            && value != 0
    }
}
pub fn language() -> String {
    let mut name = [0u16; 85];
    let len = unsafe { GetUserDefaultLocaleName(&mut name) };
    if len > 0 && String::from_utf16_lossy(&name[..len as usize]).starts_with("pt") {
        "pt-BR".into()
    } else {
        "en".into()
    }
}
pub fn set_startup(enabled: bool, path: &Path) -> std::result::Result<(), String> {
    unsafe {
        let mut key = HKEY::default();
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
        .ok()
        .map_err(|e| e.to_string())?;
        let result = if enabled {
            let text: Vec<u16> = format!("\"{}\" --watchdog --background", path.display())
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let bytes = std::slice::from_raw_parts(text.as_ptr().cast(), text.len() * 2);
            RegSetValueExW(key, w!("OrbitAxion"), None, REG_SZ, Some(bytes))
        } else {
            let r = RegDeleteValueW(key, w!("OrbitAxion"));
            if r == ERROR_FILE_NOT_FOUND {
                WIN32_ERROR(0)
            } else {
                r
            }
        };
        let _ = RegCloseKey(key);
        result.ok().map_err(|e| e.to_string())
    }
}
pub fn memory_mb() -> u64 {
    use windows::Win32::System::ProcessStatus::*;
    unsafe {
        let mut info = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        if GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut info,
            size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
        .is_ok()
        {
            info.WorkingSetSize as u64 / 1024 / 1024
        } else {
            0
        }
    }
}

pub fn extract_icon(path: &std::path::Path) -> std::result::Result<Vec<u8>, String> {
    use windows::{
        Win32::{Graphics::Gdi::*, Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES, UI::Shell::*},
        core::PCWSTR,
    };
    let text: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    unsafe {
        let mut info = SHFILEINFOW::default();
        if SHGetFileInfoW(
            PCWSTR(text.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        ) == 0
        {
            return Err("Não foi possível extrair o ícone".into());
        }
        let screen = GetDC(None);
        let dc = CreateCompatibleDC(Some(screen));
        if dc.0.is_null() {
            let _ = DestroyIcon(info.hIcon);
            let _ = ReleaseDC(None, screen);
            return Err("Falha ao criar superfície do ícone".into());
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 32,
                biHeight: -32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let result = match CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(bitmap) => {
                let old = SelectObject(dc, HGDIOBJ(bitmap.0));
                std::ptr::write_bytes(bits, 0, 32 * 32 * 4);
                let draw = DrawIconEx(dc, 0, 0, info.hIcon, 32, 32, 0, None, DI_NORMAL);
                let result = draw
                    .map(|_| {
                        let mut pixels =
                            std::slice::from_raw_parts(bits.cast::<u8>(), 32 * 32 * 4).to_vec();
                        let has_alpha = pixels.chunks(4).any(|p| p[3] != 0);
                        for p in pixels.chunks_mut(4) {
                            p.swap(0, 2);
                            if !has_alpha {
                                p[3] = if p[0] != 0 || p[1] != 0 || p[2] != 0 {
                                    255
                                } else {
                                    0
                                };
                            }
                        }
                        pixels
                    })
                    .map_err(|e| e.to_string());
                SelectObject(dc, old);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                result
            }
            Err(e) => Err(e.to_string()),
        };
        let _ = DeleteDC(dc);
        let _ = ReleaseDC(None, screen);
        let _ = DestroyIcon(info.hIcon);
        result
    }
}

pub fn cpu_ticks() -> u64 {
    unsafe {
        let (mut create, mut exit, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        if windows::Win32::System::Threading::GetProcessTimes(
            GetCurrentProcess(),
            &mut create,
            &mut exit,
            &mut kernel,
            &mut user,
        )
        .is_ok()
        {
            ((kernel.dwHighDateTime as u64) << 32 | kernel.dwLowDateTime as u64)
                + ((user.dwHighDateTime as u64) << 32 | user.dwLowDateTime as u64)
        } else {
            0
        }
    }
}

pub fn key_name(key: u32) -> String {
    if (0x70..=0x87).contains(&key) {
        return format!("F{}", key - 0x6f);
    }
    let scan = unsafe { MapVirtualKeyW(key, MAPVK_VK_TO_VSC_EX) };
    let flags = ((scan & 0xff) << 16) | if scan & 0xff00 != 0 { 1 << 24 } else { 0 };
    let mut text = [0u16; 128];
    let length = unsafe { GetKeyNameTextW(flags as i32, &mut text) };
    if length > 0 {
        String::from_utf16_lossy(&text[..length as usize])
    } else {
        format!("VK 0x{key:02X}")
    }
}

pub fn keyboard_choices() -> Vec<u32> {
    (8..=254)
        .filter(|&key| {
            orbit_core::model::valid_keyboard_key(key)
                && (unsafe { MapVirtualKeyW(key, MAPVK_VK_TO_VSC_EX) } != 0
                    || (0x70..=0x87).contains(&key)
                    || (0xa6..=0xb7).contains(&key))
        })
        .collect()
}

pub fn choose_color(owner: Option<HWND>, rgb: u32) -> std::result::Result<Option<u32>, String> {
    use windows::Win32::UI::Controls::Dialogs::*;
    static CUSTOM: std::sync::Mutex<[COLORREF; 16]> =
        std::sync::Mutex::new([COLORREF(0x00ffffff); 16]);
    let mut custom = CUSTOM.lock().map_err(|_| "Paleta indisponível")?;
    let mut dialog = CHOOSECOLORW {
        lStructSize: size_of::<CHOOSECOLORW>() as u32,
        hwndOwner: owner.unwrap_or_default(),
        rgbResult: COLORREF(((rgb >> 16) & 0xff) | (rgb & 0xff00) | ((rgb & 0xff) << 16)),
        lpCustColors: custom.as_mut_ptr(),
        Flags: CC_FULLOPEN | CC_RGBINIT,
        ..Default::default()
    };
    if unsafe { ChooseColorW(&mut dialog) }.as_bool() {
        let value = dialog.rgbResult.0;
        Ok(Some(
            ((value & 0xff) << 16) | (value & 0xff00) | ((value >> 16) & 0xff),
        ))
    } else {
        let error = unsafe { CommDlgExtendedError() };
        if error.0 == 0 {
            Ok(None)
        } else {
            Err(format!("Seletor de cores: {}", error.0))
        }
    }
}
