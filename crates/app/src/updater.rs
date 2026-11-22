use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
#[derive(Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub assets: Vec<Asset>,
}
#[derive(Clone, Deserialize)]
pub struct Asset {
    pub id: u64,
    pub name: String,
    pub browser_download_url: String,
    pub digest: Option<String>,
}
pub enum Request {
    Check(String),
    Download(String, Asset, PathBuf),
}
pub enum Reply {
    Checked(Result<Option<Release>, String>),
    Downloaded(Result<PathBuf, String>),
}
pub struct Updater {
    tx: Option<SyncSender<Request>>,
    pub replies: Receiver<Reply>,
    thread: Option<JoinHandle<()>>,
}
fn repo_valid(repo: &str) -> bool {
    let p: Vec<_> = repo.split('/').collect();
    p.len() == 2
        && p.iter().all(|s| {
            !s.is_empty()
                && s.len() <= 100
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        })
}
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .https_only(true)
        .build()
        .into()
}
fn check(repo: &str) -> Result<Option<Release>, String> {
    if !repo_valid(repo) {
        return Err("Repositório inválido: use owner/name".into());
    }
    let mut request = agent()
        .get(&format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ))
        .header("User-Agent", "OrbitAxion")
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("ORBIT_GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let mut response = request.call().map_err(|e| {
        format!("Não foi possível consultar a release (repositório privado/sem releases/rede): {e}")
    })?;
    let mut text = String::new();
    response
        .body_mut()
        .as_reader()
        .take(2 * 1024 * 1024)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    let release: Release = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let latest = semver::Version::parse(release.tag_name.trim_start_matches('v'))
        .map_err(|e| e.to_string())?;
    let current = semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    Ok((latest > current).then_some(release))
}
fn download(repo: &str, asset: &Asset, dir: PathBuf) -> Result<PathBuf, String> {
    if !repo_valid(repo)
        || !asset.name.ends_with(".msi")
        || !asset
            .browser_download_url
            .starts_with(&format!("https://github.com/{repo}/releases/download/"))
    {
        return Err("Origem do instalador inválida".into());
    }
    let digest = asset
        .digest
        .as_ref()
        .and_then(|s| s.strip_prefix("sha256:"))
        .ok_or("Release sem digest SHA-256; instalação automática bloqueada")?;
    let token = std::env::var("ORBIT_GITHUB_TOKEN").ok();
    let url = if token.is_some() {
        format!(
            "https://api.github.com/repos/{repo}/releases/assets/{}",
            asset.id
        )
    } else {
        asset.browser_download_url.clone()
    };
    let mut request = agent()
        .get(&url)
        .header("User-Agent", "OrbitAxion")
        .header("Accept", "application/octet-stream");
    if let Some(token) = token {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let mut response = request.call().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join("OrbitAxion-update.msi");
    let temp = dir.join("OrbitAxion-update.part");
    let mut file = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
    let mut reader = response.body_mut().as_reader();
    let mut hash = Sha256::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 65536];
    loop {
        let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total += n;
        if total > 200 * 1024 * 1024 {
            return Err("Instalador excede 200 MB".into());
        }
        hash.update(&buffer[..n]);
        file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
    }
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    if format!("{:x}", hash.finalize()) != digest {
        return Err("SHA-256 do instalador não confere".into());
    }
    if target.exists() {
        std::fs::remove_file(&target).map_err(|e| e.to_string())?;
    }
    std::fs::rename(temp, &target).map_err(|e| e.to_string())?;
    Ok(target)
}
impl Updater {
    pub fn start(wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, rx) = mpsc::sync_channel(1);
        let (reply, replies) = mpsc::channel();
        let thread = thread::spawn(move || {
            while let Ok(request) = rx.recv() {
                let result = match request {
                    Request::Check(repo) => Reply::Checked(check(&repo)),
                    Request::Download(repo, asset, path) => {
                        Reply::Downloaded(download(&repo, &asset, path))
                    }
                };
                if reply.send(result).is_err() {
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
    pub fn request(&self, r: Request) -> bool {
        self.tx.as_ref().is_some_and(|tx| tx.try_send(r).is_ok())
    }
}
impl Drop for Updater {
    fn drop(&mut self) {
        self.tx.take();
        // The network worker owns its data; a timeout must not delay process exit.
        self.thread.take();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restricts_repository_origin() {
        assert!(repo_valid("whoslucaxs/orbit-axion"));
        assert!(!repo_valid("https://evil/x"));
        assert!(!repo_valid("owner/repo?x=y"));
    }
}
