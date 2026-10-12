use crate::{Document, atomic_write};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

#[derive(Default, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub favorites: BTreeSet<String>,
    #[serde(default)]
    pub library_folder: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Recovery {
    pub version: u32,
    pub document: Document,
    pub source: Option<PathBuf>,
    pub png_source: Option<PathBuf>,
    pub selected: usize,
    #[serde(default)]
    pub layer: usize,
}

pub fn root() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(Path::to_owned))
                .unwrap_or_default()
        })
        .join("IconSetWizard2")
}

pub fn load_preferences(root: &Path) -> Result<Preferences, String> {
    let path = root.join("preferences.json");
    if !path.exists() {
        return Ok(Preferences::default());
    }
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 4 * 1024 * 1024 {
        return Err("Файл настроек слишком большой".into());
    }
    serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

pub fn save_preferences(root: &Path, preferences: &Preferences) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    atomic_write(
        &root.join("preferences.json"),
        &serde_json::to_vec(preferences).map_err(|e| e.to_string())?,
    )
}

pub fn read_recovery(path: &Path) -> Result<Recovery, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 128 * 1024 * 1024 {
        return Err("Копия превышает 128 MB".into());
    }
    let snapshot: Recovery =
        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if snapshot.version != 1 {
        return Err("Неизвестный формат копии".into());
    }
    snapshot.document.validate()?;
    Ok(snapshot)
}

pub struct RecoveryStore {
    pub path: PathBuf,
    lock: Option<File>,
}

#[derive(Serialize, Deserialize)]
struct RecoveryInfo {
    name: String,
    saved_at: u64,
}

pub fn recovery_description(path: &Path) -> String {
    let info = std::fs::read(path.with_extension("info"))
        .ok()
        .filter(|b| b.len() < 16384)
        .and_then(|bytes| serde_json::from_slice::<RecoveryInfo>(&bytes).ok());
    if let Some(info) = info {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|t| t.as_secs())
            .unwrap_or(info.saved_at);
        format!(
            "{} · {} мин. назад",
            info.name,
            now.saturating_sub(info.saved_at) / 60
        )
    } else {
        "Проект без описания".into()
    }
}

impl RecoveryStore {
    pub fn new(root: &Path) -> Result<Self, String> {
        let folder = root.join("recovery");
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let path = folder.join(format!("recovery-{}-{nonce}.json", std::process::id()));
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.with_extension("lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock().map_err(|e| e.to_string())?;
        Ok(Self {
            path,
            lock: Some(lock),
        })
    }
    pub fn write(&self, snapshot: &Recovery) -> Result<(), String> {
        snapshot.document.validate()?;
        let bytes = serde_json::to_vec(snapshot).map_err(|e| e.to_string())?;
        if bytes.len() > 128 * 1024 * 1024 {
            return Err("Автосохранение превышает 128 MB".into());
        }
        atomic_write(&self.path, &bytes)?;
        let info = RecoveryInfo {
            name: snapshot
                .source
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Новый Iconset".into()),
            saved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|t| t.as_secs())
                .unwrap_or(0),
        };
        if let Ok(bytes) = serde_json::to_vec(&info) {
            let _ = atomic_write(&self.path.with_extension("info"), &bytes);
        }
        Ok(())
    }
    pub fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(self.path.with_extension("info"));
    }
}

impl Drop for RecoveryStore {
    fn drop(&mut self) {
        drop(self.lock.take());
        let _ = std::fs::remove_file(self.path.with_extension("lock"));
    }
}

pub fn candidates(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root.join("recovery")) else {
        return vec![];
    };
    let mut paths = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .filter(|p| {
            let lock_path = p.with_extension("lock");
            if !lock_path.exists() {
                return true;
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(lock_path)
                .is_ok_and(|file| file.try_lock().is_ok())
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| {
        std::cmp::Reverse(std::fs::metadata(path).and_then(|m| m.modified()).ok())
    });
    paths
}

pub fn remove_recovery(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("lock"));
    let _ = std::fs::remove_file(path.with_extension("info"));
}
