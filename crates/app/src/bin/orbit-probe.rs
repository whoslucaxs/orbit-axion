//! Read-only diagnostics. Installs hooks briefly without arming interception.
use orbit_audio::{
    VolumeProvider,
    voicemeeter::{Target, Voicemeeter, installed_dll},
    windows_audio::WindowsVolume,
};
use orbit_windows::{Hooks, cursor, foreground, scale_at};
fn main() {
    println!("Orbit Axion · diagnóstico de viabilidade (sem alterar áudio)");
    println!("foreground: 0x{:x}", foreground());
    match cursor() {
        Ok(p) => println!("cursor: {p:?}; escala: {:?}", scale_at(p)),
        Err(e) => println!("cursor indisponível: {e}"),
    }
    for button in 0..=4 {
        match Hooks::start(button, false) {
            Ok((hooks, _)) => {
                drop(hooks);
                println!("hook botão {button}: instalado e removido, sem captura");
            }
            Err(e) => println!("hook botão {button}: FALHOU ({e})"),
        }
    }
    match WindowsVolume::connect().and_then(|v| v.read_level()) {
        Ok(level) => println!("Windows: {level}"),
        Err(e) => println!("Windows indisponível: {e}"),
    }
    println!("DLL registrada: {:?}", installed_dll());
    for target in [
        Target::Strip(0),
        Target::Strip(7),
        Target::Bus(0),
        Target::Bus(4),
    ] {
        match Voicemeeter::connect(target).and_then(|v| v.read_level()) {
            Ok(level) => println!("Potato {target:?}: {level}"),
            Err(e) => println!("Potato {target:?} indisponível: {e}"),
        }
    }
}
