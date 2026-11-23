use super::*;
pub fn refresh(ui: &Editor, s: &mut State) {
    s.profile = s.profile.min(s.config.profiles.len() - 1);
    s.menu = s.menu.min(s.config.menus.len() - 1);
    ui.set_profiles(ModelRc::new(VecModel::from(
        s.config
            .profiles
            .iter()
            .map(|p| p.name.clone().into())
            .collect::<Vec<_>>(),
    )));
    ui.set_menus(ModelRc::new(VecModel::from(
        s.config
            .menus
            .iter()
            .map(|m| m.name.clone().into())
            .collect::<Vec<_>>(),
    )));
    ui.set_profile_index(s.profile as i32);
    ui.set_menu_index(s.menu as i32);
    let p = &s.config.profiles[s.profile];
    ui.set_profile_name(p.name.clone().into());
    ui.set_executables(p.executables.join("\n").into());
    ui.set_priority(p.priority);
    ui.set_inherit(p.inherit_global);
    ui.set_profile_enabled(p.enabled);
    ui.set_automatic(s.automatic);
    let settings = &s.config.settings;
    ui.set_mouse_button(settings.mouse_button as i32);
    ui.set_trigger_source(i32::from(settings.keyboard_key.is_some()));
    let key = settings.keyboard_key.unwrap_or(0x7c);
    s.keyboard_codes = services::keyboard_choices();
    if !s.keyboard_codes.contains(&key) {
        s.keyboard_codes.push(key);
    }
    ui.set_keyboard_options(ModelRc::new(VecModel::from(
        s.keyboard_codes
            .iter()
            .map(|&key| format!("{} · 0x{key:02X}", services::key_name(key)).into())
            .collect::<Vec<_>>(),
    )));
    ui.set_keyboard_index(
        s.keyboard_codes
            .iter()
            .position(|&value| value == key)
            .unwrap_or(0) as i32,
    );
    ui.set_keyboard_key(key as i32);
    ui.set_keyboard_name(services::key_name(key).into());
    ui.set_hold(settings.hold);
    ui.set_hold_ms(settings.hold_ms as i32);
    ui.set_distance(settings.distance as f32);
    ui.set_glass_strength(settings.opacity * 100.0);
    ui.set_background_hex(settings.background_color.clone().into());
    ui.set_app_liquid_glass(settings.liquid_glass);
    let light = match settings.theme.as_str() {
        "light" => true,
        "dark" => false,
        _ => services::light_theme(),
    };
    let menu_style = settings.resolved_menu_style(light);
    ui.set_menu_background_hex(menu_style.background.into());
    ui.set_menu_surface_hex(menu_style.surface.into());
    ui.set_menu_icons_hex(menu_style.icons.into());
    ui.set_menu_text_hex(menu_style.text.into());
    ui.set_menu_muted_hex(menu_style.muted.into());
    ui.set_menu_border_hex(menu_style.border.into());
    ui.set_menu_title_hex(menu_style.title.into());
    ui.set_menu_selection_background_hex(menu_style.selection_background.into());
    ui.set_menu_selection_icon_hex(menu_style.selection_icon.into());
    ui.set_menu_selection_border_hex(menu_style.selection_border.into());
    ui.set_menu_shadow_hex(menu_style.shadow.into());
    ui.set_menu_opacity(menu_style.opacity * 100.0);
    ui.set_menu_center_background_hex(menu_style.center_background.into());
    ui.set_menu_center_border_hex(menu_style.center_border.into());
    ui.set_menu_glass_highlight_hex(menu_style.glass_highlight.into());
    ui.set_menu_center_radius(menu_style.center_radius);
    ui.set_menu_liquid_glass(menu_style.liquid_glass);
    ui.set_menu_glass_strength(menu_style.glass_strength * 100.0);
    ui.set_theme(match settings.theme.as_str() {
        "dark" => 1,
        "light" => 2,
        _ => 0,
    });
    ui.set_en(settings.language == "en");
    ui.set_startup(settings.startup);
    ui.set_sounds(settings.sounds);
    ui.set_sound_volume(settings.sound_volume as f32);
    ui.set_menu_size(settings.size as i32);
    ui.set_updates(settings.updates);
    ui.set_repository(settings.repository.clone().into());
    ui.set_accent_hex(settings.accent.clone().into());
    load_slot(ui, s);
}
fn load_slot(ui: &Editor, s: &mut State) {
    ui.set_menu_name(s.config.menus[s.menu].name.clone().into());
    ui.set_macro_source_names(ModelRc::new(VecModel::from(
        s.config.menus[s.menu]
            .slots
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                format!(
                    "{} · {}",
                    i + 1,
                    slot.as_ref().map_or("Vazia", |s| s.name.as_str())
                )
                .into()
            })
            .collect::<Vec<_>>(),
    )));
    let slots = &s.config.menus[s.menu].slots;
    ui.set_items(items(slots, &mut s.icons));
    ui.set_slot_index(s.slot as i32);
    ui.set_target("".into());
    ui.set_arguments("".into());
    ui.set_shortcut("".into());
    ui.set_text_value("".into());
    ui.set_command_index(0);
    s.steps.clear();
    ui.set_elevated(false);
    ui.set_confirm(false);
    ui.set_submenu_click_only(false);
    ui.set_icon_name("".into());
    ui.set_icon_is_file(false);
    ui.set_icon_loading(false);
    ui.set_icon_source_target("".into());
    set_icon_filters(ui, IconFilters::default());
    ui.set_icon_color("".into());
    ui.set_button_background("".into());
    ui.set_button_border("".into());
    if let Some(slot) = &slots[s.slot] {
        ui.set_slot_name(slot.name.clone().into());
        ui.set_icon_name(slot.icon.clone().into());
        ui.set_icon_is_file(std::path::Path::new(&slot.icon).is_absolute());
        set_icon_filters(ui, slot.icon_filters);
        ui.set_confirm(slot.confirm);
        ui.set_icon_color(slot.color.clone().unwrap_or_default().into());
        ui.set_button_background(slot.background_color.clone().unwrap_or_default().into());
        ui.set_button_border(slot.border_color.clone().unwrap_or_default().into());
        match &slot.action {
            Action::Open { target } => {
                ui.set_action_type(2);
                ui.set_target(target.clone().into());
            }
            Action::Launch {
                program,
                arguments,
                elevated,
            } => {
                ui.set_action_type(1);
                ui.set_target(program.clone().into());
                ui.set_arguments(arguments.join(" | ").into());
                ui.set_elevated(*elevated);
            }
            Action::Shortcut { keys } => {
                ui.set_action_type(3);
                ui.set_shortcut(keys.clone().into());
            }
            Action::Text { text } => {
                ui.set_action_type(4);
                ui.set_text_value(text.clone().into());
            }
            Action::Media { command } => {
                ui.set_action_type(5);
                ui.set_command_index(
                    ["play_pause", "next", "previous", "stop"]
                        .iter()
                        .position(|c| *c == command)
                        .unwrap_or(0) as i32,
                );
            }
            Action::Window { command } => {
                ui.set_action_type(6);
                ui.set_command_index(
                    ["maximize_restore", "minimize", "close"]
                        .iter()
                        .position(|c| *c == command)
                        .unwrap_or(0) as i32,
                );
            }
            Action::System { command } => {
                ui.set_action_type(7);
                ui.set_command_index(
                    ["lock", "sleep", "shutdown", "restart"]
                        .iter()
                        .position(|c| *c == command)
                        .unwrap_or(0) as i32,
                );
            }
            Action::Volume {
                provider,
                target,
                bus,
                step,
                endpoint,
            } => {
                if let Some(id) = endpoint
                    && !s.endpoints.iter().any(|e| &e.0 == id)
                {
                    s.endpoints
                        .push((id.clone(), "Indisponível · saída salva".into()));
                    let mut names = vec!["Padrão / Default".into()];
                    names.extend(s.endpoints.iter().map(|e| e.1.clone().into()));
                    ui.set_audio_endpoints(ModelRc::new(VecModel::from(names)));
                }
                ui.set_audio_endpoint(
                    endpoint
                        .as_ref()
                        .and_then(|id| s.endpoints.iter().position(|e| &e.0 == id))
                        .map_or(0, |i| i as i32 + 1),
                );
                ui.set_action_type(8);
                ui.set_audio_provider(if provider == "windows" {
                    0
                } else if *bus {
                    2
                } else {
                    1
                });
                ui.set_audio_target(*target as i32);
                ui.set_audio_step(*step);
            }
            Action::Submenu { menu, click_only } => {
                ui.set_action_type(9);
                ui.set_submenu_click_only(*click_only);
                ui.set_submenu_index(
                    s.config
                        .menus
                        .iter()
                        .position(|m| m.id == *menu)
                        .unwrap_or(0) as i32,
                );
            }
            Action::Macro { steps } => {
                ui.set_action_type(10);
                s.steps = steps.clone();
            }
            Action::Settings => ui.set_action_type(11),
        }
    } else {
        ui.set_slot_name("".into());
        ui.set_icon_name("menu".into());
        ui.set_action_type(0);
    }
    color_picker::update_swatches(ui);
    macro_list(ui, s);
}
fn macro_list(ui: &Editor, s: &State) {
    ui.set_macro_steps(ModelRc::new(VecModel::from(
        s.steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                format!(
                    "{} · {} ms · {}",
                    i + 1,
                    step.delay_ms,
                    match step.action {
                        Action::Shortcut { .. } => "Atalho",
                        Action::Text { .. } => "Texto",
                        Action::Open { .. } => "Abrir",
                        _ => "Comando",
                    }
                )
                .into()
            })
            .collect::<Vec<_>>(),
    )));
}
fn persist(ui: &Editor, s: &mut State) -> bool {
    match s
        .config
        .validate()
        .and_then(|_| s.writer.save(s.config.clone()))
    {
        Ok(()) => {
            ui.set_status(
                if ui.get_en() {
                    "Saved locally"
                } else {
                    "Salvo localmente"
                }
                .into(),
            );
            true
        }
        Err(e) => {
            ui.set_status(e.into());
            false
        }
    }
}
fn confirmation(text: &str) -> bool {
    rfd::MessageDialog::new()
        .set_title("Orbit Axion")
        .set_description(text)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        == rfd::MessageDialogResult::Yes
}
fn save_slot(ui: &Editor, s: &mut State) -> Result<(), String> {
    let action = match ui.get_action_type() {
        0 => {
            s.config.menus[s.menu].slots[s.slot] = None;
            return Ok(());
        }
        1 => Action::Launch {
            program: ui.get_target().to_string(),
            arguments: ui
                .get_arguments()
                .split('|')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
            elevated: ui.get_elevated(),
        },
        2 => Action::Open {
            target: ui.get_target().to_string(),
        },
        3 => {
            crate::executor::shortcut_keys(&ui.get_shortcut())?;
            Action::Shortcut {
                keys: ui.get_shortcut().to_string(),
            }
        }
        4 => Action::Text {
            text: ui.get_text_value().to_string(),
        },
        5 => Action::Media {
            command: ["play_pause", "next", "previous", "stop"]
                [ui.get_command_index().clamp(0, 3) as usize]
                .into(),
        },
        6 => Action::Window {
            command: ["maximize_restore", "minimize", "close"]
                [ui.get_command_index().clamp(0, 2) as usize]
                .into(),
        },
        7 => Action::System {
            command: ["lock", "sleep", "shutdown", "restart"]
                [ui.get_command_index().clamp(0, 3) as usize]
                .into(),
        },
        8 => Action::Volume {
            provider: if ui.get_audio_provider() == 0 {
                "windows"
            } else {
                "voicemeeter_potato"
            }
            .into(),
            target: ui.get_audio_target() as u8,
            bus: ui.get_audio_provider() == 2,
            step: ui.get_audio_step(),
            endpoint: if ui.get_audio_endpoint() > 0 {
                s.endpoints
                    .get(ui.get_audio_endpoint() as usize - 1)
                    .map(|e| e.0.clone())
            } else {
                None
            },
        },
        9 => Action::Submenu {
            click_only: ui.get_submenu_click_only(),
            menu: s
                .config
                .menus
                .get(ui.get_submenu_index() as usize)
                .ok_or("Menu ausente")?
                .id
                .clone(),
        },
        10 => Action::Macro {
            steps: s.steps.clone(),
        },
        _ => Action::Settings,
    };
    if ui.get_slot_name().trim().is_empty() {
        return Err("Informe um nome para a ação".into());
    }
    if matches!(action, Action::System { .. })
        && !ui.get_confirm()
        && !confirmation(
            "Esta ação altera o estado do sistema. Salvar sem confirmação de execução?",
        )
    {
        return Err("Salvamento cancelado".into());
    }
    let candidate = Slot {
        name: ui.get_slot_name().to_string(),
        icon: ui.get_icon_name().to_string(),
        icon_filters: icon_filters(ui),
        confirm: ui.get_confirm(),
        background_color: (!ui.get_button_background().is_empty())
            .then(|| ui.get_button_background().to_string()),
        border_color: (!ui.get_button_border().is_empty())
            .then(|| ui.get_button_border().to_string()),
        color: if ui.get_icon_color().is_empty() {
            None
        } else {
            Some(ui.get_icon_color().to_string())
        },
        action,
    };
    let old = s.config.menus[s.menu].slots[s.slot].replace(candidate);
    if let Err(e) = s.config.validate() {
        s.config.menus[s.menu].slots[s.slot] = old;
        return Err(e);
    }
    Ok(())
}
pub fn icon_filters(ui: &Editor) -> IconFilters {
    IconFilters {
        grayscale: ui.get_icon_grayscale().round().clamp(0.0, 100.0) as u8,
        brightness: ui.get_icon_brightness().round().clamp(-100.0, 100.0) as i16,
        contrast: ui.get_icon_contrast().round().clamp(0.0, 200.0) as u16,
        invert: ui.get_icon_invert(),
    }
}
pub fn set_icon_filters(ui: &Editor, filters: IconFilters) {
    ui.set_icon_grayscale(filters.grayscale as f32);
    ui.set_icon_brightness(filters.brightness as f32);
    ui.set_icon_contrast(filters.contrast as f32);
    ui.set_icon_invert(filters.invert);
}
pub fn preview_slot(u: &Editor, s: &mut State) {
    let mut slots = s.config.menus[s.menu].slots.clone();
    if let Some(slot) = slots[s.slot].as_mut() {
        slot.name = u.get_slot_name().to_string();
    }
    if slots[s.slot].is_none() {
        slots[s.slot] = Some(Slot {
            name: u.get_slot_name().to_string(),
            icon: u.get_icon_name().to_string(),
            icon_filters: icon_filters(u),
            confirm: false,
            color: None,
            background_color: None,
            border_color: None,
            action: Action::Settings,
        });
    }
    if let Some(slot) = slots[s.slot].as_mut() {
        slot.icon = u.get_icon_name().to_string();
        slot.icon_filters = icon_filters(u);
        slot.color = (!u.get_icon_color().is_empty()).then(|| u.get_icon_color().to_string());
        slot.background_color =
            (!u.get_button_background().is_empty()).then(|| u.get_button_background().to_string());
        slot.border_color =
            (!u.get_button_border().is_empty()).then(|| u.get_button_border().to_string());
    }
    u.set_icon_is_file(std::path::Path::new(u.get_icon_name().as_str()).is_absolute());
    color_picker::update_swatches(u);
    u.set_items(items(&slots, &mut s.icons));
}
pub fn auto_target_icon(ui: &Editor, state: &mut State) {
    let target = ui.get_target().to_string();
    if ui.get_icon_source_target() == target {
        return;
    }
    ui.set_icon_source_target("".into());
    ui.set_icon_loading(false);
    let path = std::path::Path::new(target.trim());
    if !matches!(ui.get_action_type(), 1 | 2)
        || !path.is_file()
        || !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("lnk"))
    {
        return;
    }
    match state.writer.request(storage::Job::Icon(
        path.to_path_buf(),
        state.config.menus[state.menu].id.clone(),
        state.slot,
        Some(target.clone()),
    )) {
        Ok(()) => {
            ui.set_icon_source_target(target.into());
            ui.set_icon_loading(true);
            ui.set_status("Extraindo o ícone do executável…".into());
        }
        Err(error) => ui.set_status(error.into()),
    }
}
fn import_icon(ui: &Editor, state: &mut State, executable: bool) {
    let dialog = if executable {
        rfd::FileDialog::new().add_filter("Executáveis e atalhos", &["exe", "lnk"])
    } else {
        rfd::FileDialog::new().add_filter("Imagens de ícone", &["png", "jpg", "jpeg", "ico"])
    };
    if let Some(path) = dialog.pick_file() {
        ui.set_icon_source_target("".into());
        ui.set_icon_loading(false);
        let result = state.writer.request(storage::Job::Icon(
            path,
            state.config.menus[state.menu].id.clone(),
            state.slot,
            None,
        ));
        ui.set_status(match result {
            Ok(()) => "Importando ícone…".into(),
            Err(e) => e.into(),
        });
    }
}
pub fn connect(ui: &Editor, ring: &Ring, tray: &Tray, state: Rc<RefCell<State>>) {
    macro_rules! bind {
        ($callback:ident,|$u:ident,$s:ident,$r:ident|$body:block) => {{
            let state = state.clone();
            let weak = ui.as_weak();
            let ring = ring.as_weak();
            ui.$callback(move || {
                if let (Some($u), Some($r)) = (weak.upgrade(), ring.upgrade()) {
                    let Ok(mut borrowed) = state.try_borrow_mut() else {
                        return;
                    };
                    let $s = &mut *borrowed;
                    $body
                }
            });
        }};
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_pick_slot(move |i| {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.slot = i.clamp(0, 7) as usize;
                load_slot(&ui, &mut s);
            }
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_switch_menu(move |i| {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.menu = (i as usize).min(s.config.menus.len() - 1);
                load_slot(&ui, &mut s);
            }
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_switch_profile(move |i| {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.profile = (i as usize).min(s.config.profiles.len() - 1);
                s.menu = s
                    .config
                    .menus
                    .iter()
                    .position(|m| m.id == s.config.profiles[s.profile].menu)
                    .unwrap_or(0);
                refresh(&ui, &mut s);
            }
        });
    }
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_reorder(move |from, x, y| {
            if let Some(to) = icon_at(x as f64, y as f64)
                && to != from as usize
                && let Some(ui) = weak.upgrade()
            {
                let mut s = state.borrow_mut();
                let menu = s.menu;
                s.config.menus[menu].slots.swap(from as usize, to);
                s.slot = to;
                persist(&ui, &mut s);
                load_slot(&ui, &mut s);
            }
        });
    }
    bind!(on_preview_slot, |u, s, _r| {
        preview_slot(&u, s);
    });

    bind!(on_rename_menu, |u, s, _r| {
        if u.get_menu_name().trim().is_empty() {
            u.set_status("Informe o nome do menu".into());
        } else {
            s.config.menus[s.menu].name = u.get_menu_name().to_string();
            persist(&u, s);
            refresh(&u, s);
        }
    });
    bind!(on_delete_menu, |u, s, _r| {
        let menu = &s.config.menus[s.menu].id;
        let used = s.config.profiles.iter().any(|p| &p.menu == menu)
            || s.config.menus.iter().any(|m| {
                m.slots.iter().flatten().any(
                    |slot| matches!(&slot.action,Action::Submenu{menu:target,..}if target==menu),
                )
            });
        if used {
            u.set_status("Menu em uso por perfil ou submenu; remova a referência primeiro.".into());
        } else if confirmation("Remover este menu?") {
            s.config.menus.remove(s.menu);
            s.menu = 0;
            persist(&u, s);
            refresh(&u, s);
        }
    });
    bind!(on_macro_update, |u, s, _r| {
        let index = u.get_macro_index() as usize;
        if index < s.steps.len() {
            let value = u.get_macro_value().to_string();
            let action = match u.get_macro_kind() {
                0 => Action::Shortcut { keys: value },
                1 => Action::Text { text: value },
                2 => Action::Open { target: value },
                _ => Action::Launch {
                    program: value,
                    arguments: vec![],
                    elevated: false,
                },
            };
            s.steps[index] = MacroStep {
                delay_ms: u.get_macro_delay() as u64,
                action,
            };
            macro_list(&u, s);
        }
    });
    bind!(on_save_slot, |u, s, _r| {
        if u.get_icon_loading() {
            u.set_status("Aguarde a extração do ícone…".into());
            return;
        }
        match save_slot(&u, s) {
            Ok(()) => {
                persist(&u, s);
                load_slot(&u, s);
            }
            Err(e) => u.set_status(e.into()),
        }
    });
    bind!(on_refresh_audio, |_u, s, _r| {
        s.audio.request(Request::Devices);
        s.audio.request(Request::Labels);
    });
    bind!(on_auto_target_icon, |u, s, _r| {
        auto_target_icon(&u, s);
    });
    bind!(on_browse_target, |u, s, _r| {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            u.set_target(path.to_string_lossy().into_owned().into());
            auto_target_icon(&u, s);
        }
    });
    bind!(on_import_icon, |u, s, _r| {
        import_icon(&u, s, false);
    });
    bind!(on_import_executable_icon, |u, s, _r| {
        import_icon(&u, s, true);
    });

    bind!(on_new_menu, |u, s, _r| {
        let id = format!("menu-{}", storage::timestamp());
        s.config.menus.push(Menu {
            id,
            name: format!("Submenu {}", s.config.menus.len()),
            slots: Default::default(),
        });
        s.menu = s.config.menus.len() - 1;
        persist(&u, s);
        refresh(&u, s);
    });
    bind!(on_new_profile, |u, s, _r| {
        let id = format!("profile-{}", storage::timestamp());
        s.config.menus.push(Menu {
            id: id.clone(),
            name: "Principal".into(),
            slots: Default::default(),
        });
        s.config.profiles.push(Profile {
            id: id.clone(),
            name: "Novo perfil".into(),
            menu: id,
            executables: vec![],
            priority: 10,
            inherit_global: true,
            enabled: true,
        });
        s.profile = s.config.profiles.len() - 1;
        s.menu = s.config.menus.len() - 1;
        persist(&u, s);
        refresh(&u, s);
    });
    bind!(on_duplicate_profile, |u, s, _r| {
        let mut p = s.config.profiles[s.profile].clone();
        let id = format!("profile-{}", storage::timestamp());
        let mut mapping = std::collections::HashMap::new();
        let mut stack = vec![p.menu.clone()];
        let mut clones = vec![];
        while let Some(mid) = stack.pop() {
            if mapping.contains_key(&mid) {
                continue;
            }
            if let Some(menu) = s.config.menu(&mid) {
                mapping.insert(mid.clone(), format!("{id}-{}", mapping.len()));
                clones.push(menu.clone());
                for slot in menu.slots.iter().flatten() {
                    if let Action::Submenu { menu, .. } = &slot.action {
                        stack.push(menu.clone());
                    }
                }
            }
        }
        for mut m in clones {
            m.id = mapping[&m.id].clone();
            for slot in m.slots.iter_mut().flatten() {
                if let Action::Submenu { menu, .. } = &mut slot.action {
                    *menu = mapping[menu].clone();
                }
            }
            s.config.menus.push(m);
        }
        p.menu = mapping[&p.menu].clone();
        p.id = id;
        p.name.push_str(" · cópia");
        p.executables.clear();
        s.config.profiles.push(p);
        s.profile = s.config.profiles.len() - 1;
        s.menu = s
            .config
            .menus
            .iter()
            .position(|m| m.id == s.config.profiles[s.profile].menu)
            .unwrap();
        persist(&u, s);
        refresh(&u, s);
    });
    bind!(on_delete_profile, |u, s, _r| {
        if s.config.profiles[s.profile].id == "global" {
            u.set_status("O perfil Global é obrigatório".into());
        } else if confirmation("Remover este perfil? Os menus serão preservados.") {
            s.config.profiles.remove(s.profile);
            s.profile = 0;
            s.menu = 0;
            persist(&u, s);
            refresh(&u, s);
        }
    });
    bind!(on_browse_executable, |u, _s, _r| {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Executável", &["exe"])
            .pick_file()
        {
            let mut list = u.get_executables().to_string();
            if !list.is_empty() {
                list.push('\n');
            }
            list.push_str(&path.to_string_lossy());
            u.set_executables(list.into());
        }
    });
    bind!(on_save_profile, |u, s, r| {
        let p = &mut s.config.profiles[s.profile];
        p.name = u.get_profile_name().to_string();
        p.executables = u
            .get_executables()
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        p.priority = u.get_priority();
        p.inherit_global = u.get_inherit();
        p.enabled = if p.id == "global" {
            true
        } else {
            u.get_profile_enabled()
        };
        s.automatic = u.get_automatic();
        s.config.settings.automatic_profiles = s.automatic;
        s.config.settings.manual_profile = s.config.profiles[s.profile].id.clone();
        if persist(&u, s) {
            refresh(&u, s);
            close(s, &r);
        }
    });
    {
        let weak = ui.as_weak();
        ui.on_pick_color(move |target| {
            if let Some(ui) = weak.upgrade() {
                color_picker::choose(&ui, target);
            }
        });
        let weak = ui.as_weak();
        ui.on_color_opacity(move |target, value| {
            if let Some(ui) = weak.upgrade() {
                color_picker::alpha(&ui, target, value);
            }
        });
        let weak = ui.as_weak();
        ui.on_reset_color(move |target| {
            if let Some(ui) = weak.upgrade() {
                color_picker::reset(&ui, target);
            }
        });
        let weak = ui.as_weak();
        ui.on_refresh_icons(move || {
            if let Some(ui) = weak.upgrade() {
                icon_catalog::refresh(&ui);
            }
        });
        let weak = ui.as_weak();
        ui.on_choose_icon(move |id| {
            if let Some(ui) = weak.upgrade() {
                ui.set_icon_source_target("".into());
                ui.set_icon_loading(false);
                ui.set_icon_name(id);
                ui.set_page(0);
                ui.invoke_preview_slot();
            }
        });
    }
    bind!(on_preview_background, |u, _s, r| {
        preview_background(&u, &r);
    });
    bind!(on_save_menu_style, |u, s, r| {
        let style = menu_style_from_ui(&u);
        if let Err(error) = style.validate() {
            u.set_status(error.into());
            return;
        }
        s.config.settings.menu_style = Some(style);
        if persist(&u, s) {
            preview_background(&u, &r);
            u.set_status("Aparência do menu salva separadamente do aplicativo".into());
        }
    });
    bind!(on_save_settings, |u, s, r| {
        let old = s.config.settings.clone();
        {
            let settings = &mut s.config.settings;
            settings.mouse_button = u.get_mouse_button() as u32;
            settings.keyboard_key =
                (u.get_trigger_source() == 1).then_some(u.get_keyboard_key() as u32);
            settings.hold = u.get_hold();
            settings.hold_ms = u.get_hold_ms() as u64;
            settings.distance = u.get_distance() as f64;
            settings.opacity = u.get_glass_strength() / 100.0;
            settings.background_color = u.get_background_hex().trim().to_string();
            settings.liquid_glass = u.get_app_liquid_glass();
            settings.theme = match u.get_theme() {
                1 => "dark",
                2 => "light",
                _ => "system",
            }
            .into();
            settings.language = if u.get_en() { "en" } else { "pt-BR" }.into();
            settings.startup = u.get_startup();
            settings.sounds = u.get_sounds();
            settings.sound_volume = u.get_sound_volume() as u32;
            settings.size = u.get_menu_size() as f32;
            settings.updates = u.get_updates();
            settings.repository = u.get_repository().to_string();
            settings.accent = u.get_accent_hex().to_string();
        }
        if let Err(e) = s.config.validate() {
            s.config.settings = old;
            u.set_status(e.into());
        } else if let Err(e) = std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|path| services::set_startup(s.config.settings.startup, &path))
        {
            s.config.settings = old;
            u.set_status(e.into());
        } else if persist(&u, s) {
            preview_background(&u, &r);
            r.set_zoom(s.config.settings.size / 360.0);
            s.feedback = feedback::Feedback::new(s.config.settings.sound_volume);
            arm(s, &u, &r);
            u.set_status("Preferências salvas".into());
        }
    });
    {
        let state = state.clone();
        let weak = ui.as_weak();
        ui.on_select_keyboard_key(move |index| {
            if let Some(u) = weak.upgrade()
                && let Ok(s) = state.try_borrow()
                && let Some(&key) = s.keyboard_codes.get(index as usize)
            {
                u.set_keyboard_key(key as i32);
                u.set_keyboard_index(index);
                u.set_keyboard_name(services::key_name(key).into());
                u.set_trigger_source(1);
                u.set_status(
                    "Tecla selecionada. Clique em Aplicar preferências para salvar.".into(),
                );
            }
        });
    }
    bind!(on_learn_button, |u, s, r| {
        if s.safe {
            u.set_status("Captura de botão indisponível no modo seguro".into());
        } else {
            if s.paused {
                s.paused = false;
                arm(s, &u, &r);
            }
            if let Some(session) = s.session.as_ref() {
                session.hooks.shared.learn_button();
                u.set_status(
                    "Pressione o botão do mouse ou a tecla desejada nos próximos 10 segundos."
                        .into(),
                );
            }
        }
    });
    bind!(on_toggle_capture, |u, s, r| {
        s.paused = !s.paused;
        arm(s, &u, &r);
    });
    bind!(on_import_config, |u, s, r| {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Configuração / perfil", &["json"])
            .pick_file()
            && confirmation(
                "Importar este pacote? Configurações completas substituem preferências e perfis; pacotes de perfil são adicionados. Ações importadas podem executar programas e scripts; revise antes de usar.",
            )
        {
            close(s, &r);
            match s
                .writer
                .request(storage::Job::Import(path, s.config.clone()))
            {
                Ok(()) => u.set_status("Importando…".into()),
                Err(e) => u.set_status(e.into()),
            }
        }
    });
    bind!(on_export_config, |u, s, _r| {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("orbit-axion.json")
            .save_file()
        {
            match s
                .writer
                .request(storage::Job::Export(s.config.clone(), path, None))
            {
                Ok(()) => u.set_status("Exportando configuração e ícones…".into()),
                Err(e) => u.set_status(e.into()),
            }
        }
    });
    bind!(on_export_profile, |u, s, _r| {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("orbit-profile.json")
            .save_file()
        {
            match s.writer.request(storage::Job::Export(
                s.config.clone(),
                path,
                Some(s.config.profiles[s.profile].id.clone()),
            )) {
                Ok(()) => u.set_status("Exportando perfil e ícones…".into()),
                Err(e) => u.set_status(e.into()),
            }
        }
    });
    bind!(on_restore_backup, |u, s, r| {
        if let Some(path) = rfd::FileDialog::new()
            .set_directory(s.store.directory.join("backups"))
            .add_filter("Backup", &["json"])
            .pick_file()
            && confirmation("Restaurar este backup? A configuração atual será mantida em backup.")
        {
            close(s, &r);
            match s
                .writer
                .request(storage::Job::Import(path, s.config.clone()))
            {
                Ok(()) => u.set_status("Restaurando…".into()),
                Err(e) => u.set_status(e.into()),
            }
        }
    });
    bind!(on_reset_defaults, |u, s, r| {
        if confirmation("Restaurar os padrões? A configuração atual será preservada em backup.")
        {
            close(s, &r);
            s.config = Config::default();
            s.profile = 0;
            s.menu = 0;
            persist(&u, s);
            refresh(&u, s);
            arm(s, &u, &r);
        }
    });
    bind!(on_show_data, |_u, s, _r| {
        let _ = std::process::Command::new("explorer.exe")
            .arg(&s.store.directory)
            .spawn();
    });
    bind!(on_diagnostics_export, |u, s, _r| {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("orbit-diagnostic.txt")
            .save_file()
        {
            match s.writer.request(storage::Job::Diagnostic(
                path,
                u.get_diagnostic().to_string(),
            )) {
                Ok(()) => u.set_status("Exportando diagnóstico…".into()),
                Err(e) => u.set_status(e.into()),
            }
        }
    });
    bind!(on_cancel_macro, |u, s, _r| {
        s.executor.cancel.store(true, Ordering::Release);
        u.set_status("Cancelamento solicitado".into());
    });
    bind!(on_check_updates, |u, s, _r| {
        u.set_update_status("Consultando releases…".into());
        if !s.updater.request(updater::Request::Check(
            s.config.settings.repository.clone(),
        )) {
            u.set_update_status("Verificação já em andamento".into());
        }
    });
    bind!(on_install_update, |u, s, _r| {
        if let Some(asset) = s.latest.clone() {
            if confirmation(
                "Baixar o MSI verificado por SHA-256 e abrir o instalador? O Orbit será fechado e o Windows poderá solicitar UAC.",
            ) {
                s.updater.request(updater::Request::Download(
                    s.config.settings.repository.clone(),
                    asset,
                    s.store.directory.join("updates"),
                ));
                u.set_update_status("Baixando e verificando instalador…".into());
            }
        } else {
            u.set_update_status("Nenhuma atualização com MSI disponível".into());
        }
    });
    bind!(on_macro_add, |u, s, _r| {
        let value = u.get_macro_value().to_string();
        let action = match u.get_macro_kind() {
            0 => Action::Shortcut { keys: value },
            1 => Action::Text { text: value },
            2 => Action::Open { target: value },
            4 => s.config.menus[s.menu].slots[u.get_macro_source_slot().clamp(0, 7) as usize]
                .as_ref()
                .map(|s| s.action.clone())
                .unwrap_or(Action::Text {
                    text: String::new(),
                }),
            _ => Action::Launch {
                program: value,
                arguments: vec![],
                elevated: false,
            },
        };
        s.steps.push(MacroStep {
            delay_ms: u.get_macro_delay() as u64,
            action,
        });
        macro_list(&u, s);
    });
    bind!(on_macro_remove, |u, s, _r| {
        let i = u.get_macro_index() as usize;
        if i < s.steps.len() {
            s.steps.remove(i);
            macro_list(&u, s);
        }
    });
    bind!(on_macro_up, |u, s, _r| {
        let i = u.get_macro_index() as usize;
        if i > 0 && i < s.steps.len() {
            s.steps.swap(i, i - 1);
            macro_list(&u, s);
            u.set_macro_index(i as i32 - 1);
        }
    });
    {
        let weak = ui.as_weak();
        tray.on_open_editor(move || {
            if let Some(ui) = weak.upgrade() {
                let _ = ui.show();
            }
        });
    }
    {
        let weak = ui.as_weak();
        tray.on_toggle(move || {
            if let Some(ui) = weak.upgrade() {
                ui.invoke_toggle_capture();
            }
        });
    }
    {
        let state = state.clone();
        tray.on_quit(move || {
            state
                .borrow()
                .executor
                .cancel
                .store(true, Ordering::Release);
            state.borrow_mut().session = None;
            let _ = slint::quit_event_loop();
        });
    }
}
