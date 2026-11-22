#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod audio_worker;
mod editor;
mod executor;
mod feedback;
mod icons;
#[cfg(debug_assertions)]
mod interaction_test;
mod smoke;
mod storage;
mod supervision;
mod updater;
use audio_worker::{AudioKey, AudioWorker, Request};
use orbit_core::{Gesture, direction, icon_at, icon_center, model::*};
use orbit_windows::{self as native, Hooks, Input, Point, services};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::{
    ComponentHandle, ModelRc, VecModel,
    winit_030::{
        WinitWindowAccessor,
        winit::{self, platform::windows::WindowAttributesExtWindows},
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, atomic::Ordering, mpsc::Receiver},
    time::{Duration, Instant},
};
use windows::Win32::Foundation::HWND;
slint::include_modules!();
struct Session {
    hooks: Hooks,
    events: Receiver<Input>,
    press: Instant,
    gesture: Gesture,
    wheel_slot: Option<usize>,
    center: Point,
    scale: f64,
    held: bool,
    selected: Option<usize>,
    last_read: Instant,
    profile: String,
    stack: Vec<String>,
    slots: [Option<Slot>; 8],
    query: String,
    origin: usize,
}
struct State {
    hwnd: Option<isize>,
    session: Option<Session>,
    executor: executor::Executor,
    audio: AudioWorker,
    config: Config,
    store: storage::Store,
    profile: usize,
    menu: usize,
    slot: usize,
    steps: Vec<MacroStep>,
    automatic: bool,
    paused: bool,
    safe: bool,
    active_timer: Rc<slint::Timer>,
    wake: Arc<dyn Fn() + Send + Sync>,
    last_foreground: usize,
    cpu_ticks: u64,
    last_metrics: Instant,
    opened: Instant,
    latency_ms: f64,
    updater: updater::Updater,
    latest: Option<updater::Asset>,
    endpoints: Vec<(String, String)>,
    feedback: feedback::Feedback,
    writer: storage::Writer,
    icons: icons::Icons,
}
fn hwnd(raw: isize) -> HWND {
    HWND(raw as *mut std::ffi::c_void)
}
fn monitor_scale(p: Point) -> f64 {
    native::scale_at(p).unwrap_or(1.0)
}
fn color(value: &str) -> slint::Color {
    u32::from_str_radix(value.trim_start_matches('#'), 16)
        .map(|v| slint::Color::from_rgb_u8((v >> 16) as u8, (v >> 8) as u8, v as u8))
        .unwrap_or_default()
}
fn preview_background(ui: &Editor, ring: &Ring) {
    let value = ui.get_background_hex().trim().to_string();
    if value.is_empty() || valid_color(&value) {
        for ink in [ui.global::<Ink>(), ring.global::<Ink>()] {
            ink.set_custom_background(!value.is_empty());
            if !value.is_empty() {
                ink.set_background_color(color(&value));
            }
        }
    }
    ring.set_glass_opacity(ui.get_glass_strength() / 100.0);
}
fn glyph(name: &str) -> &'static str {
    match name {
        "volume" => "\u{e767}",
        "mic" => "\u{e720}",
        "play" => "\u{e768}",
        "folder" => "\u{e8b7}",
        "copy" => "\u{e8c8}",
        "paste" => "\u{e77f}",
        "window" => "\u{e740}",
        "settings" => "\u{e713}",
        "menu" => "\u{e700}",
        "link" => "\u{e71b}",
        "text" => "\u{e8d2}",
        "keyboard" => "\u{e765}",
        "system" => "\u{e7e8}",
        "macro" => "\u{e945}",
        _ => "\u{e710}",
    }
}
fn items(slots: &[Option<Slot>; 8], icons: &mut icons::Icons) -> ModelRc<RingItem> {
    ModelRc::new(VecModel::from(
        (0..8)
            .map(|i| {
                let (x, y) = icon_center(i);
                let slot = slots[i].as_ref();
                let name = slot.map_or("", |s| s.icon.as_str());
                RingItem {
                    x: x as f32,
                    y: y as f32,
                    glyph: glyph(name).into(),
                    label: slot.map_or("Vazia / Empty", |s| s.name.as_str()).into(),
                    image: if std::path::Path::new(name).is_absolute() {
                        icons.get(name)
                    } else {
                        slint::Image::default()
                    },
                    filled: slot.is_some(),
                    tint: slot
                        .and_then(|s| s.color.as_deref())
                        .map(color)
                        .unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>(),
    ))
}
fn audio_key(action: &Action) -> Option<(AudioKey, f32)> {
    if let Action::Volume {
        provider,
        target,
        bus,
        step,
        endpoint,
    } = action
    {
        Some((
            AudioKey {
                potato: provider == "voicemeeter_potato",
                target: *target,
                bus: *bus,
                endpoint: endpoint.clone(),
            },
            *step,
        ))
    } else {
        None
    }
}
fn close(state: &mut State, ring: &Ring) {
    if state.config.settings.sounds
        && state
            .session
            .as_ref()
            .is_some_and(|s| s.hooks.shared.active.load(Ordering::Acquire))
    {
        state.feedback.play(1);
    }
    if let Some(s) = state.session.as_mut() {
        s.hooks.shared.active.store(false, Ordering::Release);
        s.held = false;
    }
    state.active_timer.stop();
    let _ = ring.hide();
    ring.set_selected(-1);
    ring.set_level("".into());
}
fn point(s: &Session, p: Point) -> (f64, f64) {
    (
        (p.x - s.center.x) as f64 / s.scale,
        (p.y - s.center.y) as f64 / s.scale,
    )
}
fn refresh_ring(state: &mut State, ring: &Ring) {
    if let Some(s) = state.session.as_mut() {
        let Some(profile) = state.config.profiles.iter().find(|p| p.id == s.profile) else {
            return;
        };
        if s.query.is_empty() {
            s.slots = state
                .config
                .effective_slots(profile, s.stack.last().unwrap());
        } else {
            let hits = state.config.search(profile, &s.query);
            s.slots = std::array::from_fn(|i| hits.get(i).cloned());
        }
        ring.set_items(items(&s.slots, &mut state.icons));
        ring.set_profile_name(profile.name.clone().into());
        ring.set_query(s.query.clone().into());
        ring.set_level("".into());
    }
}
fn choose(state: &mut State, ui: &Editor, ring: &Ring, selected: Option<usize>) {
    let Some(s) = state.session.as_ref() else {
        return;
    };
    let Some(index) = selected else {
        if s.stack.len() > 1 {
            let s = state.session.as_mut().unwrap();
            s.stack.pop();
            s.query.clear();
            refresh_ring(state, ring);
        } else {
            close(state, ring);
        }
        return;
    };
    let Some(slot) = s.slots[index].clone() else {
        return;
    };
    let origin = s.origin;
    match &slot.action {
        Action::Submenu { menu } => {
            let s = state.session.as_mut().unwrap();
            s.stack.push(menu.clone());
            s.query.clear();
            s.held = false;
            s.gesture.reset();
            refresh_ring(state, ring);
        }
        Action::Settings => {
            close(state, ring);
            let _ = ui.show();
        }
        Action::Volume { .. } => {
            let (key, _) = audio_key(&slot.action).unwrap();
            if slot.confirm {
                close(state, ring);
                if rfd::MessageDialog::new()
                    .set_title("Orbit Axion")
                    .set_description("Alternar mute deste alvo?")
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show()
                    != rfd::MessageDialogResult::Yes
                {
                    return;
                }
            } else {
                close(state, ring);
            }
            state.audio.request(Request::Mute(key));
        }
        _ => {
            close(state, ring);
            if !state.executor.run(slot, origin) {
                ui.set_status("Executor ocupado. Cancele a macro ou tente novamente.".into());
            }
        }
    }
}
fn search(state: &mut State, ring: &Ring) {
    refresh_ring(state, ring);
    if let Some(s) = state.session.as_mut() {
        s.selected = Some(0);
        ring.set_selected(0);
    }
}
fn process(state: &mut State, ui: &Editor, ring: &Ring, event: Input) {
    let Some(s) = state.session.as_mut() else {
        return;
    };
    if !matches!(event, Input::Press { .. } | Input::Learned(_))
        && !s.hooks.shared.active.load(Ordering::Acquire)
    {
        return;
    }
    match event {
        Input::Press {
            point: p,
            foreground,
            at,
        } => {
            if native::foreground() != foreground {
                return;
            }
            if s.hooks.shared.active.load(Ordering::Acquire) {
                if state.config.settings.hold
                    && !(state.config.settings.mouse_button == 3 && !s.held)
                {
                    s.held = true;
                    s.press = at;
                    s.gesture.reset();
                } else {
                    let (x, y) = point(s, p);
                    choose(state, ui, ring, icon_at(x, y));
                }
                return;
            }
            let Some(raw) = state.hwnd else {
                return;
            };
            let exe = services::executable(foreground).unwrap_or_default();
            let profile = if state.automatic {
                state.config.profile_for(&exe)
            } else {
                &state.config.profiles[state.profile]
            };
            if !profile.enabled {
                return;
            }
            s.profile = profile.id.clone();
            s.stack = vec![profile.menu.clone()];
            s.query.clear();
            s.origin = foreground;
            s.center = p;
            s.scale = monitor_scale(p) * state.config.settings.size as f64 / 360.0;
            ring.set_zoom(state.config.settings.size / 360.0);
            s.press = at;
            s.gesture.reset();
            s.held = true;
            s.selected = None;
            s.hooks.shared.origin.store(foreground, Ordering::Relaxed);
            s.hooks.shared.center_x.store(p.x, Ordering::Relaxed);
            s.hooks.shared.center_y.store(p.y, Ordering::Relaxed);
            s.hooks
                .shared
                .radius
                .store((180.0 * s.scale).round() as i32, Ordering::Relaxed);
            refresh_ring(state, ring);
            ring.set_selected(-1);
            if let Err(e) = ring.show() {
                ui.set_status(e.to_string().into());
                return;
            }
            let s = state.session.as_mut().unwrap();
            if let Err(e) = native::show_overlay(hwnd(raw), p, (360.0 * s.scale).round() as i32) {
                ui.set_status(e.to_string().into());
                close(state, ring);
                return;
            }
            s.hooks.shared.active.store(true, Ordering::Release);
            if state.config.settings.sounds {
                state.feedback.play(0);
            }
            state.latency_ms = s.press.elapsed().as_secs_f64() * 1000.0;
            let weak = ui.as_weak();
            state.active_timer.start(
                slint::TimerMode::Repeated,
                Duration::from_millis(16),
                move || {
                    if let Some(ui) = weak.upgrade() {
                        ui.invoke_pump();
                    }
                },
            );
        }
        Input::Learned(button) => {
            state.config.settings.mouse_button = button;
            ui.set_mouse_button(button as i32);
            let _ = state.writer.save(state.config.clone());
            ui.set_status(
                format!(
                    "Botão configurado: {}",
                    ["Meio", "Mouse 4", "Mouse 5", "Esquerdo", "Direito"][button as usize]
                )
                .into(),
            );
        }
        Input::Release {
            point: released,
            at,
        } => {
            s.held = false;
            if state.config.settings.hold
                && at.saturating_duration_since(s.press)
                    >= Duration::from_millis(state.config.settings.hold_ms)
            {
                if s.gesture.release(Some(0)).is_none() {
                    close(state, ring);
                } else {
                    let selected = if s.query.is_empty() {
                        let (x, y) = point(s, released);
                        direction(x, y, state.config.settings.distance)
                    } else {
                        s.selected
                    };
                    choose(state, ui, ring, selected);
                }
            }
        }
        Input::Select(p) => {
            let (x, y) = point(s, p);
            choose(state, ui, ring, icon_at(x, y));
        }
        Input::Wheel { point: p, delta } => {
            let (x, y) = point(s, p);
            if let Some(i) = icon_at(x, y)
                && let Some(slot) = &s.slots[i]
                && let Some((key, step)) = audio_key(&slot.action)
            {
                if s.wheel_slot != Some(i) {
                    s.gesture.reset_remainder();
                    s.wheel_slot = Some(i);
                }
                let steps = s.gesture.wheel(delta, true);
                if steps != 0
                    && !state
                        .audio
                        .request(Request::Adjust(key, steps as f32 * step))
                {
                    ring.set_level("Fila de áudio ocupada".into());
                }
            } else {
                s.gesture.wheel(delta, false);
                s.wheel_slot = None;
            }
        }
        Input::Escape | Input::Outside => close(state, ring),
        Input::ImeUnavailable => {
            ring.set_query("IME: use o mouse / use the mouse".into());
        }
        Input::Character(ch) => {
            if s.query.chars().count() < 64 {
                s.query.push(ch);
                search(state, ring);
            }
        }
        Input::Text(text) => {
            let room = 64usize.saturating_sub(s.query.chars().count());
            s.query.extend(text.chars().take(room));
            search(state, ring);
        }
        Input::Backspace => {
            s.query.pop();
            search(state, ring);
        }
        Input::Navigate(delta) => {
            let count = s.slots.iter().filter(|s| s.is_some()).count().max(1) as i32;
            s.selected = Some((s.selected.unwrap_or(0) as i32 + delta).rem_euclid(count) as usize);
            ring.set_selected(s.selected.unwrap() as i32);
        }
        Input::Enter => {
            let selected = s.selected;
            choose(state, ui, ring, selected);
        }
    }
}
fn arm(state: &mut State, ui: &Editor, ring: &Ring) {
    close(state, ring);
    state.session = None;
    if state.paused || state.safe || state.hwnd.is_none() {
        ui.set_status(if state.safe {
            "Modo seguro: captura desativada".into()
        } else {
            "Captura pausada".into()
        });
        return;
    }
    match Hooks::start_with_wake(state.config.settings.mouse_button, true, state.wake.clone()) {
        Ok((hooks, events)) => {
            hooks.shared.beat();
            hooks.shared.ready.store(true, Ordering::Release);
            state.session = Some(Session {
                hooks,
                events,
                press: Instant::now(),
                gesture: Gesture::default(),
                wheel_slot: None,
                center: Point::default(),
                scale: 1.0,
                held: false,
                selected: None,
                last_read: Instant::now(),
                profile: "global".into(),
                stack: vec![],
                slots: Default::default(),
                query: String::new(),
                origin: 0,
            });
            ui.set_status("Captura ativa. Abra o anel com o botão do mouse escolhido.".into());
        }
        Err(e) => {
            state.paused = true;
            ui.set_status(e.into());
        }
    }
    ui.set_armed(!state.paused && !state.safe);
}
fn pump(state: &mut State, ui: &Editor, ring: &Ring) {
    while let Ok(result) = state.audio.labels.try_recv() {
        if let Ok((strips, buses)) = result {
            ui.set_strip_labels(ModelRc::new(VecModel::from(
                strips.into_iter().map(Into::into).collect::<Vec<_>>(),
            )));
            ui.set_bus_labels(ModelRc::new(VecModel::from(
                buses.into_iter().map(Into::into).collect::<Vec<_>>(),
            )));
        }
    }
    while let Ok(result) = state.audio.devices.try_recv() {
        match result {
            Ok(devices) => {
                state.endpoints = devices;
                for slot in state
                    .config
                    .menus
                    .iter()
                    .flat_map(|m| m.slots.iter().flatten())
                {
                    if let Action::Volume {
                        provider,
                        endpoint: Some(id),
                        ..
                    } = &slot.action
                        && provider == "windows"
                        && !state.endpoints.iter().any(|e| &e.0 == id)
                    {
                        state
                            .endpoints
                            .push((id.clone(), "Indisponível · saída salva".into()));
                    }
                }

                let mut names = vec!["Padrão / Default".into()];
                names.extend(state.endpoints.iter().map(|d| d.1.clone().into()));
                ui.set_audio_endpoints(ModelRc::new(VecModel::from(names)));
                if let Some(slot) = &state.config.menus[state.menu].slots[state.slot]
                    && let Action::Volume { endpoint, .. } = &slot.action
                {
                    ui.set_audio_endpoint(
                        endpoint
                            .as_ref()
                            .and_then(|id| state.endpoints.iter().position(|e| &e.0 == id))
                            .map_or(0, |i| i as i32 + 1),
                    );
                }
            }
            Err(e) => ui.set_status(e.into()),
        }
    }

    if state.icons.poll() {
        ui.set_items(items(
            &state.config.menus[state.menu].slots,
            &mut state.icons,
        ));
        if let Some(s) = &state.session {
            ring.set_items(items(&s.slots, &mut state.icons));
        }
    }
    while let Ok(reply) = state.writer.replies.try_recv() {
        match reply {
            storage::Reply::Saved(result) => ui.set_status(match result {
                Ok(()) => if ui.get_en() {
                    "Saved locally"
                } else {
                    "Salvo localmente"
                }
                .into(),
                Err(e) => format!("Falha ao salvar: {e}").into(),
            }),
            storage::Reply::Exported(result) => ui.set_status(match result {
                Ok(()) => "Arquivo exportado localmente".into(),
                Err(e) => e.into(),
            }),
            storage::Reply::Logs(text) => ui.set_technical_log(text.into()),
            storage::Reply::Icon(result, menu, slot) => match result {
                Ok(path) => {
                    if state.config.menus[state.menu].id == menu && state.slot == slot {
                        ui.set_icon_name(path.into());
                        let mut slots = state.config.menus[state.menu].slots.clone();
                        if let Some(item) = slots[state.slot].as_mut() {
                            item.icon = ui.get_icon_name().to_string();
                        }
                        ui.set_items(items(&slots, &mut state.icons));
                    }
                }
                Err(e) => ui.set_status(e.into()),
            },
            storage::Reply::Imported(result) => match result {
                Ok(config) => {
                    close(state, ring);
                    state.config = config;
                    state.automatic = state.config.settings.automatic_profiles;
                    state.profile = state
                        .config
                        .profiles
                        .iter()
                        .position(|p| p.id == state.config.settings.manual_profile)
                        .unwrap_or(0);
                    state.menu = state
                        .config
                        .menus
                        .iter()
                        .position(|m| m.id == state.config.profiles[state.profile].menu)
                        .unwrap_or(0);
                    editor::refresh(ui, state);
                    arm(state, ui, ring);
                    ui.set_status("Importação/restauração concluída; configuração anterior preservada em backup.".into());
                }
                Err(e) => ui.set_status(e.into()),
            },
        }
    }

    if state.hwnd.is_none()
        && let Some(result) = ring.window().with_winit_window(|window| {
            let handle = window.window_handle().map_err(|e| e.to_string())?;
            let RawWindowHandle::Win32(h) = handle.as_raw() else {
                return Err::<isize, String>("Handle não Win32".into());
            };
            let raw = h.hwnd.get();
            native::prepare_overlay(hwnd(raw)).map_err(|e| e.to_string())?;
            let _ = native::acrylic_probe(hwnd(raw));
            Ok(raw)
        })
    {
        match result {
            Ok(raw) => {
                state.hwnd = Some(raw);
                arm(state, ui, ring);
            }
            Err(e) => ui.set_status(e.into()),
        }
    }
    let foreground = native::foreground();
    if let Some(s) = state.session.as_ref() {
        s.hooks.shared.beat();
        let allowed = if let Some(exe) = services::executable(foreground) {
            let p = if state.automatic {
                state.config.profile_for(&exe)
            } else {
                &state.config.profiles[state.profile]
            };
            if p.enabled { foreground } else { usize::MAX }
        } else {
            usize::MAX
        };
        s.hooks
            .shared
            .allowed_origin
            .store(allowed, Ordering::Relaxed);
    }
    let events: Vec<_> = state
        .session
        .as_ref()
        .map(|s| s.events.try_iter().take(128).collect())
        .unwrap_or_default();
    for e in events {
        process(state, ui, ring, e);
    }
    // Opening/showing windows can dispatch native events; use a fresh foreground snapshot.
    let foreground = native::foreground();
    let mut should_close = false;
    if let Some(s) = state.session.as_mut() {
        if !s.hooks.shared.ready.load(Ordering::Acquire) {
            should_close = true;
            state.paused = true;
        }
        if s.hooks.shared.active.load(Ordering::Acquire) {
            if foreground != s.origin {
                should_close = true;
            }
            if s.query.is_empty()
                && let Ok(p) = native::cursor()
            {
                let (x, y) = point(s, p);
                if icon_at(x, y) != s.wheel_slot {
                    s.gesture.reset_remainder();
                    s.wheel_slot = None;
                }
                let previous = s.selected;
                s.selected = if state.config.settings.hold && s.held {
                    direction(x, y, state.config.settings.distance)
                } else {
                    icon_at(x, y)
                };
                ring.set_selected(s.selected.map_or(-1, |i| i as i32));
                if previous != s.selected && s.selected.is_some() && state.config.settings.sounds {
                    state.feedback.play(2);
                }
                if let Some(i) = icon_at(x, y)
                    && let Some(slot) = &s.slots[i]
                    && let Some((key, _)) = audio_key(&slot.action)
                {
                    if s.last_read.elapsed() >= Duration::from_millis(150) {
                        s.last_read = Instant::now();
                        state.audio.request(Request::Read(key));
                    }
                } else {
                    ring.set_level("".into());
                }
            }
        }
    }
    if should_close {
        close(state, ring);
    }
    while let Ok(reply) = state.audio.replies.try_recv() {
        if let Some(s) = state.session.as_ref()
            && let Some(index) = s.selected
            && s.slots[index]
                .as_ref()
                .and_then(|s| audio_key(&s.action))
                .is_some_and(|(key, _)| key == reply.key)
        {
            ring.set_level(match reply.result {
                Ok(level) => level.to_string().into(),
                Err(_) => "Indisponível / Unavailable".into(),
            });
        }
    }
    while let Ok(result) = state.executor.replies.try_recv() {
        if state.config.settings.sounds {
            state.feedback.play(if result.is_ok() { 3 } else { 4 });
        }
        ui.set_status(match result {
            Ok(()) => if ui.get_en() {
                "Action completed"
            } else {
                "Ação concluída"
            }
            .into(),
            Err(e) => {
                let _ = state.writer.request(storage::Job::Log("executor_error"));
                e.into()
            }
        });
    }
    while let Ok(reply) = state.updater.replies.try_recv() {
        match reply {
            updater::Reply::Checked(result) => match result {
                Ok(Some(release)) => {
                    state.latest = release
                        .assets
                        .into_iter()
                        .find(|a| a.name.ends_with(".msi"));
                    ui.set_update_status(
                        format!(
                            "Nova versão: {}. Instalação somente após sua aprovação.",
                            release.tag_name
                        )
                        .into(),
                    );
                }
                Ok(None) => ui.set_update_status("Versão atualizada".into()),
                Err(e) => ui.set_update_status(e.into()),
            },
            updater::Reply::Downloaded(result) => match result {
                Ok(path) => match std::process::Command::new("msiexec.exe")
                    .arg("/i")
                    .arg(path)
                    .spawn()
                {
                    Ok(_) => {
                        let _ = slint::quit_event_loop();
                    }
                    Err(e) => ui.set_update_status(e.to_string().into()),
                },
                Err(e) => ui.set_update_status(e.into()),
            },
        }
    }
    if state.last_metrics.elapsed() >= Duration::from_secs(1) {
        if ui.get_page() == 3 {
            let _ = state.writer.request(storage::Job::Logs);
        }
        let interval = state.last_metrics.elapsed().as_secs_f64();
        let ticks = services::cpu_ticks();
        let cpu = (ticks.saturating_sub(state.cpu_ticks) as f64
            / 10_000_000.0
            / interval
            / std::thread::available_parallelism().map_or(1, |n| n.get()) as f64
            * 100.0)
            .clamp(0.0, 100.0);
        state.cpu_ticks = ticks;
        state.last_metrics = Instant::now();
        let light = match state.config.settings.theme.as_str() {
            "light" => true,
            "dark" => false,
            _ => services::light_theme(),
        };
        ui.global::<Ink>().set_light(light);
        preview_background(ui, ring);
        let accent = color(&state.config.settings.accent);
        ui.global::<Ink>().set_accent(accent);
        ring.global::<Ink>().set_accent(accent);
        let _ = i_slint_backend_selector::with_global_context(|ctx| ctx.set_accent_color(accent));
        ring.global::<Ink>().set_light(light);
        ui.set_diagnostic(format!("Orbit Axion {}\nRAM: {} MB · CPU: {:.2}% · Abertura: {:.2} ms\nConfiguração: {}\nCaptura: {} · Sessão: {} s\nSem telemetria e sem histórico de ações.",env!("CARGO_PKG_VERSION"),services::memory_mb(),cpu,state.latency_ms,state.store.directory.display(),if state.paused||state.safe{"pausada"}else{"ativa"},state.opened.elapsed().as_secs()).into());
    }
    state.last_foreground = foreground;
}
fn tray_image() -> slint::Image {
    let mut pixels = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(32, 32);
    for (y, row) in pixels.make_mut_slice().chunks_mut(32).enumerate() {
        for (x, p) in row.iter_mut().enumerate() {
            let d = (x as f32 - 15.5).hypot(y as f32 - 15.5);
            *p = if (9.0..=14.0).contains(&d) {
                slint::Rgba8Pixel::new(0, 172, 172, 255)
            } else {
                slint::Rgba8Pixel::new(0, 0, 0, 0)
            };
        }
    }
    slint::Image::from_rgba8(pixels)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|a| a == "--uninstall-settings") {
        let _ = services::set_startup(false, &std::env::current_exe()?);
        let root = std::path::PathBuf::from(
            std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?,
        );
        let path = root.join("OrbitAxion");
        if path.exists()&&rfd::MessageDialog::new().set_title("Orbit Axion").set_description("Apagar suas configurações, ícones, backups e logs locais? Escolha Não para mantê-los.").set_buttons(rfd::MessageButtons::YesNo).show()==rfd::MessageDialogResult::Yes {
            let resolved=std::fs::canonicalize(&path)?;let parent=std::fs::canonicalize(&root)?;
            if resolved.parent()!=Some(parent.as_path())||resolved.file_name()!=Some(std::ffi::OsStr::new("OrbitAxion")){return Err("Diretório de dados inesperado; remoção bloqueada".into());}
            std::fs::remove_dir_all(resolved)?;
        }
        return Ok(());
    }
    if let Some(code) = supervision::run(&args)? {
        std::process::exit(code);
    }
    let smoke_enabled = args.iter().any(|a| a == "--smoke-test");
    let ui_test = args.iter().any(|a| a == "--ui-test");
    let safe = ui_test || args.iter().any(|a| a == "--safe-mode");
    let _instance = if smoke_enabled || ui_test {
        None
    } else {
        match services::Instance::acquire()? {
            Some(i) => Some(i),
            None => return Ok(()),
        }
    };
    native::enable_per_monitor_dpi();
    let creating_overlay = Rc::new(Cell::new(false));
    let overlay_attributes = creating_overlay.clone();
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .with_winit_window_attributes_hook(move |attrs| {
            // Slint supplies the application name here; component titles arrive later.
            if overlay_attributes.get() {
                attrs
                    .with_active(false)
                    .with_visible(false)
                    .with_skip_taskbar(true)
                    .with_decorations(false)
                    .with_transparent(true)
                    .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
            } else {
                attrs.with_transparent(true)
            }
        })
        .select()?;
    let ui = Editor::new()?;
    // Pinned with Slint: override this process's accent, never the Windows setting.
    i_slint_backend_selector::with_global_context(|ctx| {
        ctx.set_accent_color(slint::Color::from_rgb_u8(0, 172, 172))
    })?;
    creating_overlay.set(true);
    let ring = Ring::new();
    creating_overlay.set(false);
    let ring = ring?;
    let tray = Tray::new()?;
    tray.set_tray_image(tray_image());
    if smoke_enabled || ui_test {
        tray.hide()?;
    }
    let weak = ui.as_weak();
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        let _ = weak.upgrade_in_event_loop(|ui| ui.invoke_pump());
    });
    let store = if smoke_enabled || ui_test {
        storage::Store::at(
            std::env::current_dir()?
                .join("target")
                .join(format!("self-test-{}", std::process::id())),
        )?
    } else {
        storage::Store::open()?
    };
    let (mut config, notice) = match store.load() {
        Ok(value) => value,
        Err(e) if safe => (Config::default(), Some(e)),
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("Orbit Axion")
                .set_description(&e)
                .show();
            return Err(e.into());
        }
    };
    if ui_test {
        config.settings.mouse_button = 2;
        config.settings.sound_volume = 17;
        config.settings.size = 420.0;
        config.settings.startup = true;
        config.settings.background_color = "#464545".into();
        config.settings.opacity = 0.37;
    }
    let automatic = config.settings.automatic_profiles;
    let selected_profile = config
        .profiles
        .iter()
        .position(|p| p.id == config.settings.manual_profile)
        .unwrap_or(0);
    let selected_menu = config
        .menus
        .iter()
        .position(|m| m.id == config.profiles[selected_profile].menu)
        .unwrap_or(0);
    let audio = AudioWorker::start(wake.clone());
    let executor = executor::Executor::start(audio.client());
    let state = Rc::new(RefCell::new(State {
        hwnd: None,
        session: None,
        audio,
        updater: updater::Updater::start(wake.clone()),
        latest: None,
        endpoints: vec![],
        feedback: feedback::Feedback::new(config.settings.sound_volume),
        writer: storage::Writer::start(store.clone(), wake.clone()),
        icons: icons::Icons::new(wake.clone()),
        executor,
        automatic,
        profile: selected_profile,
        config,
        store,
        menu: selected_menu,
        slot: 0,
        steps: vec![],
        paused: smoke_enabled || ui_test,
        safe,
        active_timer: Rc::new(slint::Timer::default()),
        wake,
        last_foreground: 0,
        cpu_ticks: services::cpu_ticks(),
        last_metrics: Instant::now() - Duration::from_secs(2),
        opened: Instant::now(),
        latency_ms: 0.0,
    }));
    editor::connect(&ui, &ring, &tray, state.clone());
    editor::refresh(&ui, &mut state.borrow_mut());
    preview_background(&ui, &ring);
    state.borrow().audio.request(Request::Devices);
    state.borrow().audio.request(Request::Labels);
    if !smoke_enabled && !ui_test && !state.borrow().safe && state.borrow().config.settings.updates
    {
        ui.invoke_check_updates();
    }
    if let Some(notice) = notice {
        ui.set_notice(notice.into());
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        let ring = ring.as_weak();
        ui.on_pump(move || {
            if let (Some(ui), Some(ring)) = (weak.upgrade(), ring.upgrade())
                && let Ok(mut state) = state.try_borrow_mut()
            {
                pump(&mut state, &ui, &ring);
            }
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        let ring = ring.as_weak();
        ui.window().on_close_requested(move || {
            if let Some(ring) = ring.upgrade() {
                close(&mut state.borrow_mut(), &ring);
            }
            if let Some(ui) = weak.upgrade() {
                ui.set_status("Orbit continua na bandeja do sistema.".into());
            }
            slint::CloseRequestResponse::HideWindow
        });
    }
    let idle = slint::Timer::default();
    {
        let weak = ui.as_weak();
        idle.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(100),
            move || {
                if let Some(ui) = weak.upgrade() {
                    ui.invoke_pump();
                }
            },
        );
    }
    let ui_timer = slint::Timer::default();
    if ui_test {
        ui.set_page(2);
        let weak = ui.as_weak();
        #[cfg(debug_assertions)]
        let test_state = state.clone();
        #[cfg(debug_assertions)]
        let test_ring = ring.as_weak();
        ui_timer.start(
            slint::TimerMode::SingleShot,
            Duration::from_millis(250),
            move || {
                if let Some(ui) = weak.upgrade() {
                    assert_eq!(
                        ui.get_mouse_button(),
                        2,
                        "button changed during page creation"
                    );
                    assert_eq!(ui.get_menu_size(), 420);
                    assert_eq!(ui.get_sound_volume(), 17.0);
                    assert!(ui.get_startup());
                    assert_eq!(ui.get_background_hex(), "#464545");
                    assert!((ui.get_glass_strength() - 37.0).abs() < 0.01);
                    assert!(ui.global::<Ink>().get_custom_background());
                    assert_eq!(ui.global::<Ink>().get_background_color(), color("#464545"));
                    println!("UI TEST OK: hidden-page settings preserve their loaded values");
                    #[cfg(debug_assertions)]
                    if let Some(ring) = test_ring.upgrade() {
                        interaction_test::run(ui.as_weak(), ring.as_weak(), test_state.clone(), 0);
                        return;
                    }
                    let _ = slint::quit_event_loop();
                }
            },
        );
    }
    let smoke_result = Rc::new(RefCell::new(None));
    let smoke_timer = slint::Timer::default();
    if smoke_enabled {
        let state = state.clone();
        let ring = ring.as_weak();
        let result = smoke_result.clone();
        let mut smoke = smoke::Smoke::new();
        smoke_timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(20),
            move || {
                if let Some(ring) = ring.upgrade()
                    && let Some(r) = smoke.tick(state.borrow().hwnd, &ring)
                {
                    *result.borrow_mut() = Some(r);
                    let _ = slint::quit_event_loop();
                }
            },
        );
    }
    if !smoke_enabled && (safe || !args.iter().any(|a| a == "--background")) {
        ui.show()?;
    }
    let weak = ring.as_weak();
    slint::spawn_local(async move {
        if let Some(ring) = weak.upgrade()
            && let Err(e) = ring.window().winit_window().await
        {
            eprintln!("{e}");
        }
    })?;
    slint::run_event_loop()?;
    idle.stop();
    smoke_timer.stop();
    state.borrow_mut().active_timer.stop();
    state.borrow_mut().session = None;
    state
        .borrow()
        .executor
        .cancel
        .store(true, Ordering::Release);
    if smoke_enabled {
        smoke_result
            .borrow_mut()
            .take()
            .unwrap_or_else(|| Err("Teste interrompido".into()))?;
    }
    Ok(())
}
