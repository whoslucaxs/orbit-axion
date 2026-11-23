use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuStyle {
    pub background: String,
    pub surface: String,
    pub icons: String,
    pub text: String,
    pub muted: String,
    pub border: String,
    pub title: String,
    pub selection_background: String,
    pub selection_icon: String,
    pub selection_border: String,
    pub shadow: String,
    pub opacity: f32,
    pub center_background: String,
    pub center_border: String,
    pub center_radius: f32,
    pub liquid_glass: bool,
    pub glass_strength: f32,
    pub glass_highlight: String,
}
impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            background: "#111212".into(),
            surface: "#232525D9".into(),
            icons: "#DFDEDC".into(),
            text: "#DFDEDC".into(),
            muted: "#A6A7A2".into(),
            border: "#DFDEDC28".into(),
            title: "#00ACAC".into(),
            selection_background: "#00ACAC33".into(),
            selection_icon: "#00ACAC".into(),
            selection_border: "#00ACAC".into(),
            shadow: "#00000030".into(),
            opacity: 0.92,
            center_background: "#111212C0".into(),
            center_border: "#DFDEDC28".into(),
            center_radius: 64.0,
            liquid_glass: false,
            glass_strength: 0.5,
            glass_highlight: "#FFFFFF".into(),
        }
    }
}
impl MenuStyle {
    pub fn validate(&self) -> Result<(), String> {
        if [
            &self.background,
            &self.surface,
            &self.icons,
            &self.text,
            &self.muted,
            &self.border,
            &self.title,
            &self.selection_background,
            &self.selection_icon,
            &self.selection_border,
            &self.shadow,
            &self.center_background,
            &self.center_border,
            &self.glass_highlight,
        ]
        .into_iter()
        .any(|v| !valid_menu_color(v))
            || !self.center_radius.is_finite()
            || !(24.0..=76.0).contains(&self.center_radius)
            || !self.glass_strength.is_finite()
            || !(0.0..=1.0).contains(&self.glass_strength)
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err(
                "Paleta do menu inválida: use #RRGGBB ou #RRGGBBAA e opacidade de 0 a 100%".into(),
            );
        }
        Ok(())
    }
}
pub fn valid_menu_color(value: &str) -> bool {
    matches!(value.len(), 7 | 9)
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mouse_button: u32,
    pub keyboard_key: Option<u32>,
    pub hold: bool,
    pub hold_ms: u64,
    pub distance: f64,
    pub size: f32,
    pub opacity: f32,
    pub background_color: String,
    pub liquid_glass: bool,
    pub menu_style: Option<MenuStyle>,
    pub theme: String,
    pub language: String,
    pub startup: bool,
    pub sounds: bool,
    pub sound_volume: u32,
    pub updates: bool,
    pub repository: String,
    pub accent: String,
    pub automatic_profiles: bool,
    pub manual_profile: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            mouse_button: 1,
            keyboard_key: None,
            hold: true,
            hold_ms: 220,
            distance: 48.0,
            size: 360.0,
            opacity: 0.92,
            background_color: String::new(),
            liquid_glass: false,
            menu_style: None,
            theme: "system".into(),
            language: "pt-BR".into(),
            startup: false,
            sounds: false,
            sound_volume: 30,
            updates: true,
            repository: "whoslucaxs/orbit-axion".into(),
            accent: "#00ACAC".into(),
            automatic_profiles: true,
            manual_profile: "global".into(),
        }
    }
}

pub const KEYBOARD_TRIGGER_BASE: u32 = 0x100;
pub fn valid_keyboard_key(key: u32) -> bool {
    (0x08..=0xfe).contains(&key) && key != 0xe7
}
impl Settings {
    pub fn resolved_menu_style(&self, light: bool) -> MenuStyle {
        if let Some(style) = &self.menu_style {
            return style.clone();
        }
        MenuStyle {
            background: if self.background_color.is_empty() {
                if light { "#DFDEDC" } else { "#111212" }.into()
            } else {
                self.background_color.clone()
            },
            surface: if light { "#f7f7f4D9" } else { "#232525D9" }.into(),
            icons: if light { "#151515" } else { "#DFDEDC" }.into(),
            text: if light { "#151515" } else { "#DFDEDC" }.into(),
            muted: if light { "#646560" } else { "#A6A7A2" }.into(),
            border: if light { "#FFFFFFB0" } else { "#DFDEDC28" }.into(),
            title: self.accent.clone(),
            selection_background: format!("{}33", self.accent),
            selection_icon: self.accent.clone(),
            selection_border: self.accent.clone(),
            shadow: "#00000030".into(),
            opacity: self.opacity,
            ..MenuStyle::default()
        }
    }
    pub fn gesture_distance(&self) -> f64 {
        self.distance.max(
            self.menu_style
                .as_ref()
                .map_or(64.0, |style| style.center_radius as f64),
        )
    }
    pub fn activation_code(&self) -> u32 {
        self.keyboard_key
            .map_or(self.mouse_button, |key| KEYBOARD_TRIGGER_BASE + key)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub schema_version: u32,
    pub settings: Settings,
    pub profiles: Vec<Profile>,
    pub menus: Vec<Menu>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub menu: String,
    pub executables: Vec<String>,
    pub priority: i32,
    pub inherit_global: bool,
    pub enabled: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Menu {
    pub id: String,
    pub name: String,
    pub slots: [Option<Slot>; 8],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default)]
pub struct IconFilters {
    pub grayscale: u8,
    pub brightness: i16,
    pub contrast: u16,
    pub invert: bool,
}
impl Default for IconFilters {
    fn default() -> Self {
        Self {
            grayscale: 0,
            brightness: 0,
            contrast: 100,
            invert: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slot {
    pub name: String,
    pub icon: String,
    #[serde(default)]
    pub icon_filters: IconFilters,
    pub confirm: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    pub action: Action,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    Open {
        target: String,
    },
    Launch {
        program: String,
        arguments: Vec<String>,
        elevated: bool,
    },
    Shortcut {
        keys: String,
    },
    Text {
        text: String,
    },
    Media {
        command: String,
    },
    Window {
        command: String,
    },
    System {
        command: String,
    },
    Volume {
        provider: String,
        target: u8,
        bus: bool,
        step: f32,
        #[serde(default)]
        endpoint: Option<String>,
    },
    Submenu {
        menu: String,
        #[serde(default)]
        click_only: bool,
    },
    Macro {
        steps: Vec<MacroStep>,
    },
    Settings,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MacroStep {
    pub delay_ms: u64,
    pub action: Action,
}

impl Default for Config {
    fn default() -> Self {
        let definitions = [
            (
                "Volume Windows",
                "volume",
                Action::Volume {
                    provider: "windows".into(),
                    target: 0,
                    bus: false,
                    step: 2.0,
                    endpoint: None,
                },
            ),
            (
                "Reproduzir / pausar",
                "play",
                Action::Media {
                    command: "play_pause".into(),
                },
            ),
            (
                "Explorador",
                "folder",
                Action::Launch {
                    program: "explorer.exe".into(),
                    arguments: vec![],
                    elevated: false,
                },
            ),
            (
                "Copiar",
                "copy",
                Action::Shortcut {
                    keys: "CTRL+C".into(),
                },
            ),
            (
                "Colar",
                "paste",
                Action::Shortcut {
                    keys: "CTRL+V".into(),
                },
            ),
            (
                "Maximizar / restaurar",
                "window",
                Action::Window {
                    command: "maximize_restore".into(),
                },
            ),
            (
                "Microfone Potato",
                "mic",
                Action::Volume {
                    provider: "voicemeeter_potato".into(),
                    target: 0,
                    bus: false,
                    step: 1.0,
                    endpoint: None,
                },
            ),
            ("Configurações", "settings", Action::Settings),
        ];
        Self {
            schema_version: 1,
            settings: Settings::default(),
            profiles: vec![Profile {
                id: "global".into(),
                name: "Global".into(),
                menu: "global".into(),
                executables: vec![],
                priority: 0,
                inherit_global: false,
                enabled: true,
            }],
            menus: vec![Menu {
                id: "global".into(),
                name: "Principal".into(),
                slots: definitions.map(|(name, icon, action)| {
                    Some(Slot {
                        name: name.into(),
                        icon: icon.into(),
                        icon_filters: IconFilters::default(),
                        confirm: false,
                        color: None,
                        background_color: None,
                        border_color: None,
                        action,
                    })
                }),
            }],
        }
    }
}
fn action_valid(action: &Action, depth: usize) -> Result<(), String> {
    if depth > 16 {
        return Err("Macro aninhada excessivamente".into());
    }
    match action {
        Action::Open { target } if target.trim().is_empty() || target.contains('\0') => {
            Err("Destino vazio ou inválido".into())
        }
        Action::Launch {
            program, arguments, ..
        } if program.trim().is_empty()
            || program.contains('\0')
            || arguments.iter().any(|a| a.contains('\0')) =>
        {
            Err("Programa ou argumentos inválidos".into())
        }
        Action::Shortcut { keys } if keys.trim().is_empty() => Err("Atalho vazio".into()),
        Action::Volume {
            provider,
            target,
            bus,
            step,
            endpoint,
        } => {
            if endpoint
                .as_ref()
                .is_some_and(|e| e.contains('\0') || e.len() > 4096)
            {
                return Err("Endpoint inválido".into());
            }
            if !["windows", "voicemeeter_potato"].contains(&provider.as_str())
                || (*provider == "voicemeeter_potato" && *target > if *bus { 4 } else { 7 })
                || !step.is_finite()
                || *step <= 0.0
                || *step > 100.0
            {
                Err("Alvo ou passo de volume inválido".into())
            } else {
                Ok(())
            }
        }
        Action::Media { command }
            if !["play_pause", "next", "previous", "stop"].contains(&command.as_str()) =>
        {
            Err("Comando de mídia inválido".into())
        }
        Action::Window { command }
            if !["maximize_restore", "minimize", "close"].contains(&command.as_str()) =>
        {
            Err("Comando de janela inválido".into())
        }
        Action::System { command }
            if !["lock", "sleep", "shutdown", "restart"].contains(&command.as_str()) =>
        {
            Err("Comando de sistema inválido".into())
        }
        Action::Macro { steps } => {
            if steps.is_empty() || steps.len() > 256 {
                return Err("Macro deve conter 1 a 256 etapas".into());
            }
            for s in steps {
                if s.delay_ms > 600_000 {
                    return Err("Atraso maior que 10 minutos".into());
                }
                if matches!(s.action, Action::Submenu { .. } | Action::Settings) {
                    return Err("Submenu/configurações não são etapas de macro".into());
                }
                action_valid(&s.action, depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!(
                "Esquema {} não suportado; configuração preservada",
                self.schema_version
            ));
        }
        let s = &self.settings;
        if let Some(style) = &s.menu_style {
            style.validate()?;
        }
        if !valid_color(&s.accent) {
            return Err("Cor de destaque inválida: use #RRGGBB".into());
        }
        if !s.background_color.is_empty() && !valid_color(&s.background_color) {
            return Err(
                "Cor do fundo inválida: use #RRGGBB ou deixe vazio para seguir o tema".into(),
            );
        }
        if s.keyboard_key.is_some_and(|key| !valid_keyboard_key(key)) {
            return Err("Capture uma tecla válida do teclado".into());
        }
        if s.mouse_button > 4
            || !(50..=2000).contains(&s.hold_ms)
            || !s.distance.is_finite()
            || !(10.0..=200.0).contains(&s.distance)
            || !s.size.is_finite()
            || !(280.0..=640.0).contains(&s.size)
            || !s.opacity.is_finite()
            || !(0.0..=1.0).contains(&s.opacity)
        {
            return Err("Parâmetros de interação inválidos".into());
        }
        if !["pt-BR", "en"].contains(&s.language.as_str())
            || !["system", "dark", "light"].contains(&s.theme.as_str())
        {
            return Err("Idioma ou tema inválido".into());
        }
        if self.menus.is_empty() || self.menus.len() > 4096 || self.profiles.len() > 1024 {
            return Err("Quantidade inválida de menus/perfis".into());
        }
        let menus: HashMap<_, _> = self.menus.iter().map(|m| (m.id.as_str(), m)).collect();
        if menus.len() != self.menus.len() || self.menus.iter().any(|m| m.id.trim().is_empty()) {
            return Err("IDs de menu vazios/duplicados".into());
        }
        let mut profiles = HashSet::new();
        for p in &self.profiles {
            if !profiles.insert(&p.id) || p.id.is_empty() || !menus.contains_key(p.menu.as_str()) {
                return Err("Perfil inválido, duplicado ou com menu ausente".into());
            }
        }
        if !self.profiles.iter().any(|p| p.id == "global" && p.enabled) {
            return Err("Perfil Global obrigatório e ativo".into());
        }
        for m in &self.menus {
            for slot in m.slots.iter().flatten() {
                if slot.name.len() > 200 || slot.icon.contains('\0') {
                    return Err("Nome/ícone inválido".into());
                }
                if [&slot.color, &slot.background_color, &slot.border_color]
                    .into_iter()
                    .flatten()
                    .any(|c| !valid_menu_color(c))
                {
                    return Err("Cor do ícone inválida: use #RRGGBB".into());
                }
                if slot.icon_filters.grayscale > 100
                    || !(-100..=100).contains(&slot.icon_filters.brightness)
                    || slot.icon_filters.contrast > 200
                {
                    return Err("Filtros do ícone fora do intervalo".into());
                }
                action_valid(&slot.action, 0)?;
            }
        }
        // Iterative DFS permits deep menus without recursively consuming the call stack.
        let mut done = HashSet::new();
        for root in &self.menus {
            let mut path = HashSet::new();
            let mut stack = vec![(root.id.as_str(), false)];
            while let Some((id, exit)) = stack.pop() {
                if exit {
                    path.remove(id);
                    done.insert(id);
                    continue;
                }
                if path.contains(id) {
                    return Err("Ciclo em submenus".into());
                }
                if done.contains(id) {
                    continue;
                }
                let menu = menus.get(id).ok_or("Submenu inexistente")?;
                path.insert(id);
                stack.push((id, true));
                for slot in menu.slots.iter().flatten() {
                    if let Action::Submenu { menu, .. } = &slot.action {
                        stack.push((menu, false));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn profile_for(&self, executable: &str) -> &Profile {
        self.profiles
            .iter()
            .filter(|p| {
                p.id != "global"
                    && p.executables
                        .iter()
                        .any(|e| normalize(e) == normalize(executable))
            })
            .max_by_key(|p| p.priority)
            .unwrap_or_else(|| {
                self.profiles
                    .iter()
                    .find(|p| p.id == "global")
                    .expect("validated global")
            })
    }
    pub fn menu(&self, id: &str) -> Option<&Menu> {
        self.menus.iter().find(|m| m.id == id)
    }
    pub fn effective_slots(&self, profile: &Profile, menu: &str) -> [Option<Slot>; 8] {
        let mut slots = self.menu(menu).map(|m| m.slots.clone()).unwrap_or_default();
        if profile.inherit_global && profile.menu == menu && profile.id != "global" {
            let global = self
                .profiles
                .iter()
                .find(|p| p.id == "global")
                .and_then(|p| self.menu(&p.menu));
            if let Some(g) = global {
                for (slot, fallback) in slots.iter_mut().zip(&g.slots) {
                    if slot.is_none() {
                        *slot = fallback.clone();
                    }
                }
            }
        }
        slots
    }
    pub fn search(&self, profile: &Profile, query: &str) -> Vec<Slot> {
        let mut stack = vec![profile.menu.clone()];
        let mut seen = HashSet::new();
        let mut hits = vec![];
        while let Some(id) = stack.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            for s in self.effective_slots(profile, &id).into_iter().flatten() {
                if let Action::Submenu { menu, .. } = &s.action {
                    stack.push(menu.clone());
                } else if let Some(score) = fuzzy(&s.name, query) {
                    hits.push((score, s));
                }
            }
        }
        hits.sort_by_key(|h| h.0);
        hits.into_iter().take(8).map(|h| h.1).collect()
    }
}
fn normalize(s: &str) -> String {
    s.replace('/', "\\").to_lowercase()
}
fn fuzzy(text: &str, query: &str) -> Option<usize> {
    let t = text.to_lowercase();
    let q = query.to_lowercase();
    let mut chars = q.chars();
    let mut wanted = chars.next()?;
    let mut score = 0;
    for (i, c) in t.chars().enumerate() {
        if c == wanted {
            score += i;
            if let Some(n) = chars.next() {
                wanted = n;
            } else {
                return Some(score);
            }
        }
    }
    None
}

pub fn valid_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_filters_roundtrip_validate_and_default_for_legacy_slots() {
        let mut config = Config::default();
        let filters = IconFilters {
            grayscale: 100,
            brightness: -25,
            contrast: 150,
            invert: true,
        };
        config.menus[0].slots[0].as_mut().unwrap().icon_filters = filters;
        config.validate().unwrap();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(
            loaded.menus[0].slots[0].as_ref().unwrap().icon_filters,
            filters
        );
        for invalid in [
            IconFilters {
                grayscale: 101,
                ..filters
            },
            IconFilters {
                brightness: 101,
                ..filters
            },
            IconFilters {
                contrast: 201,
                ..filters
            },
        ] {
            config.menus[0].slots[0].as_mut().unwrap().icon_filters = invalid;
            assert!(config.validate().is_err());
        }
        let mut legacy = serde_json::to_value(Config::default()).unwrap();
        legacy["menus"][0]["slots"][0]
            .as_object_mut()
            .unwrap()
            .remove("icon_filters");
        let legacy: Config = serde_json::from_value(legacy).unwrap();
        assert_eq!(
            legacy.menus[0].slots[0].as_ref().unwrap().icon_filters,
            IconFilters::default()
        );
    }
    #[test]
    fn per_button_colors_and_center_geometry_are_validated_and_legacy_defaults_work() {
        let mut config = Config::default();
        let slot = config.menus[0].slots[0].as_mut().unwrap();
        slot.background_color = Some("#00884480".into());
        slot.border_color = Some("#FFAA0000".into());
        slot.color = Some("#FFFFFF80".into());
        config.settings.menu_style = Some(MenuStyle {
            center_radius: 76.0,
            liquid_glass: true,
            ..MenuStyle::default()
        });
        config.validate().unwrap();
        assert_eq!(config.settings.gesture_distance(), 76.0);
        config.settings.menu_style.as_mut().unwrap().center_radius = 100.0;
        assert!(config.validate().is_err());
        let legacy = serde_json::json!({"name":"Old","icon":"volume","confirm":false,"action":{"kind":"settings"}});
        let slot: Slot = serde_json::from_value(legacy).unwrap();
        assert!(slot.background_color.is_none() && slot.border_color.is_none());
    }

    #[test]
    fn defaults_roundtrip_and_profile_fallback() {
        let c = Config::default();
        c.validate().unwrap();
        let json = serde_json::to_vec(&c).unwrap();
        serde_json::from_slice::<Config>(&json)
            .unwrap()
            .validate()
            .unwrap();
        assert_eq!(c.profile_for("unknown.exe").id, "global");
        assert!(!c.search(&c.profiles[0], "vlw").is_empty());
    }
    #[test]
    fn accepts_mouse_buttons_and_custom_background_with_legacy_defaults() {
        let mut c = Config::default();
        for button in 0..5 {
            c.settings.mouse_button = button;
            c.settings.background_color = "#464545".into();
            c.settings.opacity = 0.0;
            c.validate().unwrap();
        }
        c.settings.mouse_button = 5;
        assert!(c.validate().is_err());
        c.settings.mouse_button = 0;
        c.settings.background_color = "invalid".into();
        assert!(c.validate().is_err());
        let mut legacy = serde_json::to_value(Config::default()).unwrap();
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("background_color");
        let loaded: Config = serde_json::from_value(legacy).unwrap();
        assert!(loaded.settings.background_color.is_empty());
        loaded.validate().unwrap();
    }
    #[test]
    fn frozen_menu_palette_is_independent_from_app_theme_and_colors() {
        let mut settings = Settings {
            background_color: "#464545".into(),
            accent: "#123456".into(),
            opacity: 0.37,
            ..Settings::default()
        };
        let migrated = settings.resolved_menu_style(false);
        assert_eq!(migrated.background, "#464545");
        assert_eq!(migrated.selection_icon, "#123456");
        settings.menu_style = Some(migrated.clone());
        settings.background_color = "#FFFFFF".into();
        settings.accent = "#FF0000".into();
        settings.opacity = 1.0;
        settings.theme = "light".into();
        assert_eq!(settings.resolved_menu_style(true), migrated);
        let mut menu = migrated;
        menu.selection_background = "#12345678".into();
        menu.validate().unwrap();
        menu.border = "invalid".into();
        assert!(menu.validate().is_err());
    }
    #[test]
    fn keyboard_trigger_roundtrips_and_old_mouse_configs_still_load() {
        let mut c = Config::default();
        for key in [0x41, 0x08, 0x20, 0x70, 0x7c, 0x87, 0xa3] {
            c.settings.keyboard_key = Some(key);
            c.validate().unwrap();
            let loaded: Config = serde_json::from_slice(&serde_json::to_vec(&c).unwrap()).unwrap();
            assert_eq!(
                loaded.settings.activation_code(),
                KEYBOARD_TRIGGER_BASE + key
            );
        }
        for key in [0, 1, 0xe7, 255, 256] {
            c.settings.keyboard_key = Some(key);
            assert!(c.validate().is_err());
        }
        let mut legacy = serde_json::to_value(Config::default()).unwrap();
        legacy["settings"]
            .as_object_mut()
            .unwrap()
            .remove("keyboard_key");
        let loaded: Config = serde_json::from_value(legacy).unwrap();
        assert_eq!(
            loaded.settings.activation_code(),
            loaded.settings.mouse_button
        );
    }
    #[test]
    fn submenu_click_mode_preserves_legacy_and_round_trips() {
        let legacy = r#"{"kind":"submenu","menu":"child"}"#;
        let action: Action = serde_json::from_str(legacy).unwrap();
        assert!(matches!(
            action,
            Action::Submenu {
                click_only: false,
                ..
            }
        ));
        let action = Action::Submenu {
            menu: "child".into(),
            click_only: true,
        };
        let encoded = serde_json::to_string(&action).unwrap();
        let loaded: Action = serde_json::from_str(&encoded).unwrap();
        assert!(matches!(loaded, Action::Submenu { menu, click_only: true } if menu == "child"));
    }
    #[test]
    fn rejects_cycles_and_invalid_audio() {
        let mut c = Config::default();
        c.menus[0].slots[0].as_mut().unwrap().action = Action::Submenu {
            menu: "global".into(),
            click_only: false,
        };
        assert!(c.validate().is_err());
        c.menus[0].slots[0].as_mut().unwrap().action = Action::Volume {
            provider: "voicemeeter_potato".into(),
            target: 8,
            bus: false,
            step: 1.0,
            endpoint: None,
        };
        assert!(c.validate().is_err());
    }
    #[test]
    fn inherits_and_matches_executables_by_priority() {
        let mut c = Config::default();
        c.menus.push(Menu {
            id: "work".into(),
            name: "Work".into(),
            slots: Default::default(),
        });
        c.profiles.push(Profile {
            id: "work".into(),
            name: "Work".into(),
            menu: "work".into(),
            executables: vec!["C:\\Code.exe".into()],
            priority: 2,
            inherit_global: true,
            enabled: true,
        });
        c.validate().unwrap();
        let p = c.profile_for("c:/CODE.exe");
        assert_eq!(p.id, "work");
        assert!(c.effective_slots(p, "work")[0].is_some());
    }
}
