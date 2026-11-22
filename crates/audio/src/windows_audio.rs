use super::{AudioResult, Level, VolumeProvider, bounded_delta};
use std::{marker::PhantomData, mem::ManuallyDrop, rc::Rc};
use windows::Win32::{
    Media::Audio::{Endpoints::IAudioEndpointVolume, *},
    System::Com::*,
};

pub struct WindowsVolume {
    endpoint: ManuallyDrop<IAudioEndpointVolume>,
    _thread: PhantomData<Rc<()>>,
}

impl WindowsVolume {
    pub fn connect() -> AudioResult<Self> {
        Self::connect_endpoint(None)
    }
    pub fn connect_endpoint(endpoint: Option<&str>) -> AudioResult<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|e| e.to_string())?;
        }
        let result = (|| unsafe {
            let devices: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device = if let Some(id) = endpoint {
                let wide: Vec<_> = id.encode_utf16().chain(Some(0)).collect();
                devices.GetDevice(windows::core::PCWSTR(wide.as_ptr()))?
            } else {
                devices.GetDefaultAudioEndpoint(eRender, eMultimedia)?
            };
            let endpoint = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
            Ok::<_, windows::core::Error>(Self {
                endpoint: ManuallyDrop::new(endpoint),
                _thread: PhantomData,
            })
        })();
        match result {
            Ok(volume) => Ok(volume),
            Err(e) => {
                unsafe {
                    CoUninitialize();
                }
                Err(e.to_string())
            }
        }
    }
}

impl VolumeProvider for WindowsVolume {
    fn toggle_mute(&self) -> AudioResult<Level> {
        let muted = self.read_level()?.muted;
        unsafe {
            self.endpoint
                .SetMute(!muted, std::ptr::null())
                .map_err(|e| e.to_string())?;
        }
        self.read_level()
    }
    fn read_level(&self) -> AudioResult<Level> {
        unsafe {
            Ok(Level {
                value: self
                    .endpoint
                    .GetMasterVolumeLevelScalar()
                    .map_err(|e| e.to_string())?
                    * 100.0,
                muted: self
                    .endpoint
                    .GetMute()
                    .map_err(|e| e.to_string())?
                    .as_bool(),
                unit: "%",
            })
        }
    }
    fn adjust_level(&self, steps: f32) -> AudioResult<Level> {
        let current = self.read_level()?;
        let next = bounded_delta(current.value, steps * 2.0, 0.0, 100.0)?;
        unsafe {
            self.endpoint
                .SetMasterVolumeLevelScalar(next / 100.0, std::ptr::null())
                .map_err(|e| e.to_string())?;
        }
        self.read_level()
    }
}

impl Drop for WindowsVolume {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.endpoint);
            CoUninitialize();
        }
    }
}

pub fn list_endpoints() -> AudioResult<Vec<(String, String)>> {
    use windows::Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        System::Com::StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
    };
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
    }
    let result = (|| unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let devices = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
        let mut result = vec![];
        for i in 0..devices.GetCount()? {
            let device = devices.Item(i)?;
            let raw = device.GetId()?;
            let id = raw.to_string();
            CoTaskMemFree(Some(raw.0.cast()));
            let id = id?;
            let properties = device.OpenPropertyStore(STGM_READ)?;
            let mut value = properties.GetValue(&PKEY_Device_FriendlyName)?;
            let text = PropVariantToStringAlloc(&value);
            let _ = PropVariantClear(&mut value);
            let name = if let Ok(raw) = text {
                let name = raw.to_string();
                CoTaskMemFree(Some(raw.0.cast()));
                name.unwrap_or_else(|_| id.clone())
            } else {
                id.clone()
            };
            result.push((id, name));
        }
        Ok::<_, windows::core::Error>(result)
    })();
    unsafe {
        CoUninitialize();
    }
    result.map_err(|e| e.to_string())
}
