use super::*;
fn raw(ui: &Editor, target: i32) -> String {
    match target {
        0 => ui.get_menu_background_hex().to_string(),
        1 => ui.get_menu_surface_hex().to_string(),
        2 => ui.get_menu_icons_hex().to_string(),
        3 => ui.get_menu_text_hex().to_string(),
        4 => ui.get_menu_muted_hex().to_string(),
        5 => ui.get_menu_border_hex().to_string(),
        6 => ui.get_menu_title_hex().to_string(),
        7 => ui.get_menu_selection_background_hex().to_string(),
        8 => ui.get_menu_selection_icon_hex().to_string(),
        9 => ui.get_menu_selection_border_hex().to_string(),
        10 => ui.get_menu_shadow_hex().to_string(),
        11 => ui.get_menu_center_background_hex().to_string(),
        12 => ui.get_menu_center_border_hex().to_string(),
        13 => ui.get_menu_glass_highlight_hex().to_string(),
        100 => ui.get_background_hex().to_string(),
        101 => ui.get_accent_hex().to_string(),
        200 => ui.get_icon_color().to_string(),
        201 => ui.get_button_background().to_string(),
        202 => ui.get_button_border().to_string(),
        _ => String::new(),
    }
}
fn current(ui: &Editor, target: i32) -> slint::Color {
    let raw = raw(ui, target);
    if valid_menu_color(&raw) {
        return menu_color(&raw);
    }
    match target {
        100 => ui.global::<Ink>().get_panel_background(),
        101 => ui.global::<Ink>().get_accent(),
        200 => ui.global::<MenuInk>().get_icons(),
        201 => ui.global::<MenuInk>().get_surface(),
        202 => ui.global::<MenuInk>().get_border(),
        _ => slint::Color::from_rgb_u8(255, 255, 255),
    }
}
pub fn set(ui: &Editor, target: i32, value: String) {
    match target {
        0 => ui.set_menu_background_hex(value.into()),
        1 => ui.set_menu_surface_hex(value.into()),
        2 => ui.set_menu_icons_hex(value.into()),
        3 => ui.set_menu_text_hex(value.into()),
        4 => ui.set_menu_muted_hex(value.into()),
        5 => ui.set_menu_border_hex(value.into()),
        6 => ui.set_menu_title_hex(value.into()),
        7 => ui.set_menu_selection_background_hex(value.into()),
        8 => ui.set_menu_selection_icon_hex(value.into()),
        9 => ui.set_menu_selection_border_hex(value.into()),
        10 => ui.set_menu_shadow_hex(value.into()),
        11 => ui.set_menu_center_background_hex(value.into()),
        12 => ui.set_menu_center_border_hex(value.into()),
        13 => ui.set_menu_glass_highlight_hex(value.into()),
        100 => ui.set_background_hex(value.into()),
        101 => ui.set_accent_hex(value.into()),
        200 => ui.set_icon_color(value.into()),
        201 => ui.set_button_background(value.into()),
        202 => ui.set_button_border(value.into()),
        _ => {}
    }
}
pub fn choose(ui: &Editor, target: i32) {
    let initial = current(ui, target);
    let rgb =
        ((initial.red() as u32) << 16) | ((initial.green() as u32) << 8) | initial.blue() as u32;
    let owner = ui
        .window()
        .with_winit_window(|window| {
            window
                .window_handle()
                .ok()
                .and_then(|handle| match handle.as_raw() {
                    RawWindowHandle::Win32(h) => Some(hwnd(h.hwnd.get())),
                    _ => None,
                })
        })
        .flatten();
    match services::choose_color(owner, rgb) {
        Ok(Some(rgb)) => set(
            ui,
            target,
            if matches!(target, 100 | 101) {
                format!("#{rgb:06X}")
            } else {
                format!("#{rgb:06X}{:02X}", initial.alpha())
            },
        ),
        Err(error) => ui.set_status(error.into()),
        _ => {}
    }
}
pub fn alpha(ui: &Editor, target: i32, value: f32) {
    if matches!(target, 100 | 101) {
        return;
    }
    let color = current(ui, target);
    let alpha = (value.clamp(0.0, 100.0) * 255.0 / 100.0).round() as u8;
    set(
        ui,
        target,
        format!(
            "#{:02X}{:02X}{:02X}{alpha:02X}",
            color.red(),
            color.green(),
            color.blue()
        ),
    );
}
pub fn reset(ui: &Editor, target: i32) {
    let style = MenuStyle::default();
    set(
        ui,
        target,
        match target {
            0 => style.background,
            1 => style.surface,
            2 => style.icons,
            3 => style.text,
            4 => style.muted,
            5 => style.border,
            6 => style.title,
            7 => style.selection_background,
            8 => style.selection_icon,
            9 => style.selection_border,
            10 => style.shadow,
            11 => style.center_background,
            12 => style.center_border,
            13 => style.glass_highlight,
            101 => "#00ACAC".into(),
            _ => String::new(),
        },
    );
}
pub fn update_swatches(ui: &Editor) {
    ui.set_icon_color_preview(current(ui, 200));
    ui.set_button_background_preview(current(ui, 201));
    ui.set_button_border_preview(current(ui, 202));
}
