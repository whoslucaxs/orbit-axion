use orbit_core::model::Config;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use windows::{
    Win32::Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW},
    core::PCWSTR,
};

#[derive(Clone)]
pub struct Store {
    pub directory: PathBuf,
}

pub enum Job {
    Save(Config),
    Import(PathBuf, Config),
    Export(Config, PathBuf, Option<String>),
    Icon(PathBuf, String, usize),
    Diagnostic(PathBuf, String),
    Logs,
    Log(&'static str),
}
pub enum Reply {
    Saved(Result<(), String>),
    Imported(Result<Config, String>),
    Exported(Result<(), String>),
    Icon(Result<String, String>, String, usize),
    Logs(String),
}
pub struct Writer {
    tx: Option<std::sync::mpsc::SyncSender<Job>>,
    pub replies: std::sync::mpsc::Receiver<Reply>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Writer {
    pub fn start(store: Store, wake: std::sync::Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<Job>(4);
        let (reply, replies) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            while let Ok(job) = rx.recv() {
                let response = match job {
                    Job::Save(c) => Reply::Saved(store.save(&c)),
                    Job::Import(path, base) => {
                        Reply::Imported(store.import_bundle(&path, base).and_then(|c| {
                            store.save(&c)?;
                            Ok(c)
                        }))
                    }
                    Job::Export(c, path, profile) => {
                        Reply::Exported(store.export_bundle(&c, &path, profile.as_deref()))
                    }
                    Job::Icon(path, menu, slot) => {
                        Reply::Icon(store.import_icon(&path), menu, slot)
                    }
                    Job::Diagnostic(path, text) => {
                        Reply::Exported(fs::write(path, text).map_err(|e| e.to_string()))
                    }
                    Job::Logs => Reply::Logs(store.read_logs()),
                    Job::Log(category) => {
                        store.log_error(category);
                        continue;
                    }
                };
                if reply.send(response).is_err() {
                    break;
                }
                wake();
            }
        });
        Self {
            tx: Some(tx),
            replies,
            thread: Some(thread),
        }
    }
    pub fn request(&self, job: Job) -> Result<(), String> {
        self.tx
            .as_ref()
            .ok_or("Armazenamento encerrado")?
            .try_send(job)
            .map_err(|_| "Armazenamento ocupado. Tente novamente.".into())
    }
    pub fn save(&self, c: Config) -> Result<(), String> {
        self.request(Job::Save(c))
    }
}
impl Drop for Writer {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn wide(p: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    p.as_os_str().encode_wide().chain(Some(0)).collect()
}
impl Store {
    pub fn open() -> Result<Self, String> {
        let root = std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA ausente")?;
        Self::at(PathBuf::from(root).join("OrbitAxion"))
    }
    pub(crate) fn at(directory: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(directory.join("backups")).map_err(|e| e.to_string())?;
        fs::create_dir_all(directory.join("icons")).map_err(|e| e.to_string())?;
        Ok(Self { directory })
    }
    pub fn read(path: &Path) -> Result<Config, String> {
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        if metadata.len() > 8 * 1024 * 1024 {
            return Err("Configuração excede 8 MB".into());
        }
        let c: Config = serde_json::from_reader(File::open(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        c.validate()?;
        Ok(c)
    }
    pub fn load(&self) -> Result<(Config, Option<String>), String> {
        let path = self.directory.join("config.json");
        if !path.exists() {
            let mut c = Config::default();
            c.settings.language = orbit_windows::services::language();
            self.save(&c)?;
            return Ok((c, None));
        }
        match Self::read(&path) {
            Ok(c) => Ok((c, None)),
            Err(original) => {
                for backup in self.backups()? {
                    if let Ok(c) = Self::read(&backup) {
                        let quarantine = self
                            .directory
                            .join(format!("config.corrupt-{}.json", timestamp()));
                        fs::copy(&path, &quarantine).map_err(|e| e.to_string())?;
                        self.write(&c)?;
                        return Ok((
                            c,
                            Some(format!(
                                "Configuração recuperada de backup. Original preservado em {} ({original})",
                                quarantine.display()
                            )),
                        ));
                    }
                }
                Err(format!(
                    "Configuração inválida e sem backup válido: {original}. Arquivo preservado; use --safe-mode."
                ))
            }
        }
    }
    fn write(&self, c: &Config) -> Result<(), String> {
        c.validate()?;
        let bytes = serde_json::to_vec_pretty(c).map_err(|e| e.to_string())?;
        if bytes.len() > 8 * 1024 * 1024 {
            return Err("Configuração excede 8 MB".into());
        }
        let target = self.directory.join("config.json");
        let tmp = self.directory.join(format!("config-{}.tmp", timestamp()));
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())?;
        drop(f);
        unsafe {
            MoveFileExW(
                PCWSTR(wide(&tmp).as_ptr()),
                PCWSTR(wide(&target).as_ptr()),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
            .map_err(|e| e.to_string())
        }
    }
    pub fn save(&self, c: &Config) -> Result<(), String> {
        c.validate()?;
        let old = self.directory.join("config.json");
        if old.exists() && Self::read(&old).is_ok() {
            fs::copy(
                &old,
                self.directory
                    .join("backups")
                    .join(format!("{}.json", timestamp())),
            )
            .map_err(|e| e.to_string())?;
        }
        if old.exists() && Self::read(&old).is_err() {
            fs::copy(
                &old,
                self.directory
                    .join(format!("config.corrupt-{}.json", timestamp())),
            )
            .map_err(|e| e.to_string())?;
        }
        self.write(c)?;
        for stale in self.backups()?.into_iter().skip(30) {
            fs::remove_file(stale).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn backups(&self) -> Result<Vec<PathBuf>, String> {
        let mut list: Vec<_> = fs::read_dir(self.directory.join("backups"))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        list.sort();
        list.reverse();
        Ok(list)
    }
    pub fn export_bundle(
        &self,
        c: &Config,
        path: &Path,
        profile: Option<&str>,
    ) -> Result<(), String> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        let mut export = if let Some(id) = profile {
            profile_export(c, id)?
        } else {
            c.clone()
        };
        export.validate()?;
        let mut assets = std::collections::HashMap::<String, String>::new();
        let mut names = std::collections::HashMap::<String, String>::new();
        let mut size = 0;
        for slot in export
            .menus
            .iter_mut()
            .flat_map(|m| m.slots.iter_mut().flatten())
        {
            if Path::new(&slot.icon).is_absolute() {
                if let Some(name) = names.get(&slot.icon) {
                    slot.icon = name.clone();
                    continue;
                }
                let bytes = fs::read(&slot.icon).map_err(|e| e.to_string())?;
                size += bytes.len();
                if size > 16 * 1024 * 1024 {
                    return Err("Ícones do pacote excedem 16 MB".into());
                }
                let ext = Path::new(&slot.icon)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png")
                    .to_lowercase();
                if !["png", "jpg", "jpeg", "ico"].contains(&ext.as_str()) {
                    return Err("Formato de ícone não exportável".into());
                }
                let name = format!("asset:{}.{}", names.len(), ext);
                assets.insert(name.clone(), STANDARD.encode(bytes));
                names.insert(slot.icon.clone(), name.clone());
                slot.icon = name;
            }
        }
        let primary = profile.map(|_| {
            export
                .profiles
                .iter()
                .find(|p| p.id != "global")
                .unwrap()
                .id
                .clone()
        });
        let mut value = serde_json::to_value(export).map_err(|e| e.to_string())?;
        value["assets"] = serde_json::to_value(assets).map_err(|e| e.to_string())?;
        if let Some(primary) = primary {
            value["profile_id"] = primary.into();
        }
        let mut f = File::create(path).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut f, &value).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())
    }
    pub fn import_bundle(&self, path: &Path, base: Config) -> Result<Config, String> {
        use base64::{Engine, engine::general_purpose::STANDARD};
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
            return Err("Pacote excede 32 MB".into());
        }
        let value: serde_json::Value =
            serde_json::from_reader(File::open(path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let mut c: Config = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        c.validate()?;
        let mut resolved = std::collections::HashMap::<String, String>::new();
        let mut total = 0;
        for slot in c
            .menus
            .iter_mut()
            .flat_map(|m| m.slots.iter_mut().flatten())
        {
            if slot.icon.starts_with("asset:") {
                if let Some(path) = resolved.get(&slot.icon) {
                    slot.icon = path.clone();
                    continue;
                }
                let content = value
                    .get("assets")
                    .and_then(|a| a.get(&slot.icon))
                    .and_then(|v| v.as_str())
                    .ok_or("Ícone incorporado ausente")?;
                let bytes = STANDARD.decode(content).map_err(|e| e.to_string())?;
                total += bytes.len();
                if total > 16 * 1024 * 1024 {
                    return Err("Ícones excedem 16 MB".into());
                }
                let ext = slot.icon.rsplit('.').next().unwrap_or("");
                if !["png", "jpg", "jpeg", "ico"].contains(&ext) {
                    return Err("Extensão de ícone inválida".into());
                }
                let path = self
                    .directory
                    .join("icons")
                    .join(format!("{}.{}", timestamp(), ext));
                fs::write(&path, bytes).map_err(|e| e.to_string())?;
                let path = path.to_string_lossy().into_owned();
                resolved.insert(slot.icon.clone(), path.clone());
                slot.icon = path;
            }
        }
        if let Some(id) = value.get("profile_id").and_then(|v| v.as_str()) {
            merge_profile(base, &c, id)
        } else {
            Ok(c)
        }
    }
    fn read_logs(&self) -> String {
        use std::io::{Read, Seek, SeekFrom};
        let Ok(mut f) = File::open(self.directory.join("technical.log")) else {
            return "Nenhum erro técnico registrado.".into();
        };
        let len = f.metadata().map_or(0, |m| m.len());
        let _ = f.seek(SeekFrom::Start(len.saturating_sub(16384)));
        let mut text = String::new();
        let _ = f.read_to_string(&mut text);
        text
    }
    pub fn import_icon(&self, path: &Path) -> Result<String, String> {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ["exe", "lnk"].contains(&ext.as_str()) {
            let pixels = orbit_windows::services::extract_icon(path)?;
            let dest = self
                .directory
                .join("icons")
                .join(format!("{}.png", timestamp()));
            image::save_buffer(&dest, &pixels, 32, 32, image::ColorType::Rgba8)
                .map_err(|e| e.to_string())?;
            return Ok(dest.to_string_lossy().into());
        }
        if !["png", "jpg", "jpeg", "ico"].contains(&ext.as_str()) {
            return Err("Use PNG, JPG ou ICO".into());
        }
        if fs::metadata(path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
            return Err("Ícone excede 16 MB".into());
        }
        let dest = self
            .directory
            .join("icons")
            .join(format!("{}.{}", timestamp(), ext));
        fs::copy(path, &dest).map_err(|e| e.to_string())?;
        Ok(dest.to_string_lossy().into())
    }
    pub fn log_error(&self, category: &str) {
        if let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.directory.join("technical.log"))
        {
            let _ = writeln!(f, "{} {category}", timestamp());
        }
    }
}
pub fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn profile_export(c: &Config, id: &str) -> Result<Config, String> {
    use orbit_core::model::*;
    let p = c
        .profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or("Perfil ausente")?;
    let mut profile = p.clone();
    profile.id = format!("exported-{}", timestamp());
    profile.inherit_global = false;
    let mut stack = vec![p.menu.clone()];
    let mut seen = std::collections::HashSet::new();
    let mut menus = vec![];
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let mut menu = c.menu(&id).ok_or("Menu ausente")?.clone();
        menu.slots = c.effective_slots(p, &id);
        for slot in menu.slots.iter().flatten() {
            if let Action::Submenu { menu } = &slot.action {
                stack.push(menu.clone());
            }
        }
        menus.push(menu);
    }
    let dummy = format!("global-empty-{}", timestamp());
    menus.push(Menu {
        id: dummy.clone(),
        name: "Global".into(),
        slots: Default::default(),
    });
    let global = Profile {
        id: "global".into(),
        name: "Global".into(),
        menu: dummy,
        executables: vec![],
        priority: 0,
        inherit_global: false,
        enabled: true,
    };
    Ok(Config {
        schema_version: 1,
        settings: Settings::default(),
        profiles: vec![global, profile],
        menus,
    })
}
fn merge_profile(mut base: Config, incoming: &Config, id: &str) -> Result<Config, String> {
    use orbit_core::model::*;
    let mut profile = incoming
        .profiles
        .iter()
        .find(|p| p.id == id)
        .ok_or("Perfil do pacote ausente")?
        .clone();
    let prefix = format!("imported-{}", timestamp());
    let mut map = std::collections::HashMap::new();
    let mut stack = vec![profile.menu.clone()];
    let mut menus = vec![];
    while let Some(id) = stack.pop() {
        if map.contains_key(&id) {
            continue;
        }
        let menu = incoming.menu(&id).ok_or("Menu do pacote ausente")?.clone();
        map.insert(id, format!("{prefix}-{}", map.len()));
        for slot in menu.slots.iter().flatten() {
            if let Action::Submenu { menu } = &slot.action {
                stack.push(menu.clone());
            }
        }
        menus.push(menu);
    }
    for mut menu in menus {
        menu.id = map[&menu.id].clone();
        for slot in menu.slots.iter_mut().flatten() {
            if let Action::Submenu { menu } = &mut slot.action {
                *menu = map[menu].clone();
            }
        }
        base.menus.push(menu);
    }
    profile.id = prefix;
    profile.menu = map[&profile.menu].clone();
    base.profiles.push(profile);
    base.validate()?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundles_embed_icons_and_merge_profiles_without_replacing_global() {
        let dir = std::env::temp_dir().join(format!("orbit-bundle-{}", timestamp()));
        let store = Store::at(dir.clone()).unwrap();
        let mut config = Config::default();
        let icon = dir.join("icon.png");
        image::save_buffer(&icon, &[0, 172, 172, 255], 1, 1, image::ColorType::Rgba8).unwrap();
        config.menus[0].slots[0].as_mut().unwrap().icon = icon.to_string_lossy().into();
        let package = dir.join("bundle.json");
        store.export_bundle(&config, &package, None).unwrap();
        fs::remove_file(&icon).unwrap();
        let imported = store.import_bundle(&package, Config::default()).unwrap();
        assert!(Path::new(&imported.menus[0].slots[0].as_ref().unwrap().icon).exists());
        store
            .export_bundle(&imported, &package, Some("global"))
            .unwrap();
        let base = Config::default();
        let original = serde_json::to_value(&base.profiles[0]).unwrap();
        let merged = store.import_bundle(&package, base).unwrap();
        assert_eq!(merged.profiles.len(), 2);
        assert_eq!(serde_json::to_value(&merged.profiles[0]).unwrap(), original);
        merged.validate().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn atomic_save_recovery_and_reject_invalid_import() {
        let dir = std::env::temp_dir().join(format!("orbit-test-{}", timestamp()));
        let s = Store::at(dir.clone()).unwrap();
        let (mut c, _) = s.load().unwrap();
        c.settings.hold = false;
        s.save(&c).unwrap();
        assert!(!s.load().unwrap().0.settings.hold);
        fs::write(dir.join("config.json"), b"broken").unwrap();
        assert!(s.load().unwrap().1.is_some());
        assert!(s.directory.join("config.json").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
