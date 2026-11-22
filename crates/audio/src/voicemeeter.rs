use super::{AudioResult, Level, VolumeProvider, bounded_delta};
use libloading::Library;
use std::{
    ffi::{CString, c_char},
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
};
use windows::{Win32::System::Registry::*, core::w};

type Login = unsafe extern "system" fn() -> i32;
type Logout = unsafe extern "system" fn() -> i32;
type GetType = unsafe extern "system" fn(*mut i32) -> i32;
type GetFloat = unsafe extern "system" fn(*const c_char, *mut f32) -> i32;
type SetFloat = unsafe extern "system" fn(*const c_char, f32) -> i32;
type Dirty = unsafe extern "system" fn() -> i32;
type GetString = unsafe extern "system" fn(*const c_char, *mut u16) -> i32;

#[derive(Clone, Copy, Debug)]
pub enum Target {
    Strip(u8),
    Bus(u8),
}

impl Target {
    fn parameter(self, suffix: &str) -> AudioResult<CString> {
        let name = match self {
            Self::Strip(i) if i <= 7 => format!("Strip[{i}].{suffix}"),
            Self::Bus(i) if i <= 4 => format!("Bus[{i}].{suffix}"),
            _ => return Err("Índice Potato fora dos limites: strips 0..7, buses 0..4".into()),
        };
        CString::new(name).map_err(|e| e.to_string())
    }
}

/// Discover an installed copy through its uninstall registry key, never DLL search on PATH/CWD.
pub fn installed_dll() -> AudioResult<PathBuf> {
    for view in [RRF_SUBKEY_WOW6432KEY, RRF_SUBKEY_WOW6464KEY] {
        let mut buffer = [0u16; 4096];
        let mut bytes = (buffer.len() * 2) as u32;
        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                w!(
                    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\VB:Voicemeeter {17359A74-1236-5467}"
                ),
                w!("UninstallString"),
                RRF_RT_REG_SZ | view,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut bytes),
            )
        };
        if status.is_ok() {
            let value = String::from_utf16_lossy(
                &buffer[..buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len())],
            );
            let executable = if let Some(rest) = value.strip_prefix('"') {
                rest.split('"').next().unwrap_or("").to_string()
            } else if let Some(end) = value.to_ascii_lowercase().find(".exe") {
                value[..end + 4].to_string()
            } else {
                continue;
            };
            if let Some(parent) = Path::new(&executable).parent() {
                let dll = parent.join("VoicemeeterRemote64.dll");
                if dll.is_absolute() && dll.is_file() {
                    return Ok(dll);
                }
            }
        }
    }
    Err("VoicemeeterRemote64.dll não encontrada na instalação registrada".into())
}

pub struct Voicemeeter {
    _library: Library,
    logout: Logout,
    get: GetFloat,
    set: SetFloat,
    dirty: Dirty,
    get_string: GetString,
    target: Target,
    _thread: PhantomData<Rc<()>>,
}

fn check(code: i32, operation: &str) -> AudioResult<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(format!(
            "Voicemeeter {operation}: código {code} (indisponível ou parâmetro inválido)"
        ))
    }
}

impl Voicemeeter {
    pub fn connect(target: Target) -> AudioResult<Self> {
        target.parameter("Gain")?;
        let dll = installed_dll()?;
        unsafe {
            // Absolute installed library; symbols are retained only while the library lives.
            let library: Library = libloading::os::windows::Library::load_with_flags(
                dll,
                libloading::os::windows::LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR
                    | libloading::os::windows::LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
            .map_err(|e| e.to_string())?
            .into();
            let login: Login = *library.get(b"VBVMR_Login\0").map_err(|e| e.to_string())?;
            let logout: Logout = *library.get(b"VBVMR_Logout\0").map_err(|e| e.to_string())?;
            let get_type: GetType = *library
                .get(b"VBVMR_GetVoicemeeterType\0")
                .map_err(|e| e.to_string())?;
            let get: GetFloat = *library
                .get(b"VBVMR_GetParameterFloat\0")
                .map_err(|e| e.to_string())?;
            let set: SetFloat = *library
                .get(b"VBVMR_SetParameterFloat\0")
                .map_err(|e| e.to_string())?;
            let dirty: Dirty = *library
                .get(b"VBVMR_IsParametersDirty\0")
                .map_err(|e| e.to_string())?;
            let get_string: GetString = *library
                .get(b"VBVMR_GetParameterStringW\0")
                .map_err(|e| e.to_string())?;
            let code = login();
            if code != 0 {
                if code == 1 {
                    logout();
                }
                return Err(format!(
                    "Voicemeeter Login: {code}; abra o Potato e tente reconectar"
                ));
            }
            let mut edition = 0;
            let type_code = get_type(&mut edition);
            if type_code != 0 || edition != 3 {
                logout();
                return Err(format!(
                    "Potato necessário: edição {edition}, código {type_code}"
                ));
            }
            Ok(Self {
                _library: library,
                logout,
                get,
                set,
                dirty,
                get_string,
                target,
                _thread: PhantomData,
            })
        }
    }
    pub fn labels(&self) -> AudioResult<(Vec<String>, Vec<String>)> {
        let status = unsafe { (self.dirty)() };
        if status < 0 {
            return Err(format!("Voicemeeter IsParametersDirty: {status}"));
        }
        let read = |target: Target| -> AudioResult<String> {
            let name = target.parameter("Label")?;
            let mut text = [0u16; 512];
            check(
                unsafe { (self.get_string)(name.as_ptr(), text.as_mut_ptr()) },
                "GetParameterStringW",
            )?;
            let length = text.iter().position(|&c| c == 0).unwrap_or(text.len());
            Ok(String::from_utf16_lossy(&text[..length]))
        };
        let strips = (0..8)
            .map(|i| read(Target::Strip(i)).map(|name| format!("Strip {i} · {name}")))
            .collect::<AudioResult<_>>()?;
        let buses = (0..5)
            .map(|i| read(Target::Bus(i)).map(|name| format!("Bus {i} · {name}")))
            .collect::<AudioResult<_>>()?;
        Ok((strips, buses))
    }
    fn read_parameter(&self, suffix: &str) -> AudioResult<f32> {
        let status = unsafe { (self.dirty)() };
        if status < 0 {
            return Err(format!("Voicemeeter IsParametersDirty: {status}"));
        }
        let name = self.target.parameter(suffix)?;
        let mut value = 0.0;
        check(
            unsafe { (self.get)(name.as_ptr(), &mut value) },
            "GetParameterFloat",
        )?;
        if !value.is_finite() {
            return Err("Voicemeeter retornou valor não finito".into());
        }
        Ok(value)
    }
}

impl VolumeProvider for Voicemeeter {
    fn toggle_mute(&self) -> AudioResult<Level> {
        let name = self.target.parameter("Mute")?;
        let next = if self.read_parameter("Mute")? == 0.0 {
            1.0
        } else {
            0.0
        };
        check(
            unsafe { (self.set)(name.as_ptr(), next) },
            "SetParameterFloat Mute",
        )?;
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if (self.read_parameter("Mute")? - next).abs() < 0.1 {
                break;
            }
        }
        self.read_level()
    }
    fn read_level(&self) -> AudioResult<Level> {
        Ok(Level {
            value: self.read_parameter("Gain")?,
            muted: self.read_parameter("Mute")? != 0.0,
            unit: "dB",
        })
    }
    fn adjust_level(&self, steps: f32) -> AudioResult<Level> {
        // SDK Remote API documents Gain as -60..+12 dB. Never reuse a cached level for writes.
        let next = bounded_delta(self.read_parameter("Gain")?, steps, -60.0, 12.0)?;
        let name = self.target.parameter("Gain")?;
        check(
            unsafe { (self.set)(name.as_ptr(), next) },
            "SetParameterFloat",
        )?;
        // Remote setters are asynchronous; serialize updates until the server reflects the write.
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let level = self.read_level()?;
            if (level.value - next).abs() < 0.05 {
                return Ok(level);
            }
        }
        self.read_level()
    }
}

impl Drop for Voicemeeter {
    fn drop(&mut self) {
        unsafe {
            (self.logout)();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_only_potato_targets() {
        assert_eq!(
            Target::Strip(7)
                .parameter("Gain")
                .unwrap()
                .to_str()
                .unwrap(),
            "Strip[7].Gain"
        );
        assert!(Target::Strip(8).parameter("Gain").is_err());
        assert!(Target::Bus(5).parameter("Gain").is_err());
        assert!(Target::Bus(4).parameter("Gain").is_ok());
    }
}
