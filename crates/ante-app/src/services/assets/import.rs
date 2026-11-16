use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};

use crate::services::assets::{
    catalog::{LOCAL_FILES, read_local_file, validate_local_file},
    music::MUSIC_FILES,
    shaders::{SHADER_FILES, shader_path, validate as validate_shader},
    sounds::{SOUND_FILES, sound_path, validate as validate_sound},
    ui_text::{TEXT_FILES, validate as validate_text},
};

pub struct AssetSettings {
    root: PathBuf,
    pub source: String,
    pub active: Option<PathBuf>,
    pub editing: bool,
    pub confirm_remove: bool,
    pub status: String,
}

impl AssetSettings {
    pub fn load() -> Self {
        Self::from_root(crate::services::storage::local_dir().join("local-assets"))
    }

    fn from_root(root: PathBuf) -> Self {
        let saved = fs::read(root.join("active.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .filter(|value| value["version"].as_u64() == Some(1));
        let source = saved
            .as_ref()
            .and_then(|v| v["source"].as_str())
            .unwrap_or_default()
            .to_owned();
        let active = saved
            .as_ref()
            .and_then(|v| v["lot"].as_str())
            .filter(|name| is_lot_name(name))
            .map(|name| root.join(name))
            .filter(|lot| {
                lot.canonicalize()
                    .ok()
                    .zip(root.canonicalize().ok())
                    .is_some_and(|(lot, root)| lot.parent() == Some(root.as_path()) && lot.is_dir())
            });
        let status = if active.is_some() {
            "Local assets installed"
        } else {
            "Using fallback assets"
        }
        .into();
        Self {
            root,
            source,
            active,
            editing: false,
            confirm_remove: false,
            status,
        }
    }

    pub fn destination(&self) -> &Path {
        &self.root
    }

    pub fn import(&mut self) -> io::Result<usize> {
        let entered = self.source.trim().trim_matches('"');
        if entered.is_empty() {
            return Err(io::Error::other("Choose the Balatro folder first."));
        }
        let source = Path::new(entered).canonicalize()?;
        let _lock = lock_store(&self.root)?;
        let root = self.root.canonicalize()?;
        if source.starts_with(&root) {
            return Err(io::Error::other("Choose the original Balatro folder."));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let lot = format!("lot-{}-{now}", std::process::id());
        let stage = root.join(&lot);
        let pointer = root.join(format!(".{lot}.json"));
        fs::create_dir(&stage)?;
        let result = (|| {
            let mut count = 0;
            for &(relative, dimensions) in LOCAL_FILES {
                match fs::symlink_metadata(source.join(relative)) {
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                    Ok(_) => {}
                }
                let bytes = read_local_file(&source, relative)?;
                validate_local_file(&bytes, dimensions)
                    .map_err(|error| io::Error::other(format!("{relative}: {error}")))?;
                let destination = stage.join(relative);
                fs::create_dir_all(destination.parent().expect("asset parent"))?;
                fs::write(destination, bytes)?;
                count += 1;
            }
            for &name in SHADER_FILES {
                let relative = shader_path(name);
                if fs::symlink_metadata(source.join(&relative)).is_err() {
                    continue;
                }
                let bytes = read_local_file(&source, &relative)?;
                // A shader the converter cannot handle is skipped; the sprite stays plain.
                if validate_shader(&bytes).is_err() {
                    continue;
                }
                let destination = stage.join(&relative);
                fs::create_dir_all(destination.parent().expect("shader parent"))?;
                fs::write(destination, bytes)?;
                count += 1;
            }
            // Interface strings: read by key at startup, never executed.
            for &relative in TEXT_FILES {
                if fs::symlink_metadata(source.join(relative)).is_err() {
                    continue;
                }
                let bytes = read_local_file(&source, relative)?;
                if validate_text(&bytes).is_err() {
                    continue;
                }
                let destination = stage.join(relative);
                fs::create_dir_all(destination.parent().expect("text parent"))?;
                fs::write(destination, bytes)?;
                count += 1;
            }
            for &name in SOUND_FILES.iter().chain(MUSIC_FILES) {
                let relative = sound_path(name);
                if fs::symlink_metadata(source.join(&relative)).is_err() {
                    continue;
                }
                let bytes = read_local_file(&source, &relative)?;
                if validate_sound(&bytes).is_err() {
                    continue;
                }
                let destination = stage.join(&relative);
                fs::create_dir_all(destination.parent().expect("sound parent"))?;
                fs::write(destination, bytes)?;
                count += 1;
            }
            if count == 0 {
                return Err(io::Error::other(
                    "The folder contains no recognised Ante assets.",
                ));
            }
            fs::write(
                &pointer,
                json!({"version": 1, "lot": lot, "source": entered}).to_string(),
            )?;
            fs::rename(&pointer, root.join("active.json"))?;
            Ok(count)
        })();
        match result {
            Ok(count) => {
                self.active = Some(stage);
                Ok(count)
            }
            Err(error) => {
                let _ = fs::remove_file(pointer);
                let _ = fs::remove_dir_all(stage);
                Err(error)
            }
        }
    }

    pub fn remove(&mut self) -> io::Result<()> {
        let _lock = lock_store(&self.root)?;
        match fs::remove_file(self.root.join("active.json")) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        self.active = None;
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() && entry.file_name().to_str().is_some_and(is_lot_name) {
                fs::remove_dir_all(entry.path())?;
            }
        }
        Ok(())
    }
}

fn lock_store(root: &Path) -> io::Result<File> {
    fs::create_dir_all(root)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(".lock"))?;
    lock.try_lock()
        .map_err(|error| io::Error::other(format!("Asset store is unavailable: {error}")))?;
    Ok(lock)
}

fn is_lot_name(name: &str) -> bool {
    name.strip_prefix("lot-")
        .and_then(|body| body.split_once('-'))
        .is_some_and(|(pid, nonce)| {
            !pid.is_empty()
                && !nonce.is_empty()
                && pid.bytes().chain(nonce.bytes()).all(|b| b.is_ascii_digit())
        })
}

pub fn steam_source() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles(x86)")
        .map(|base| PathBuf::from(base).join("Steam/steamapps/common/Balatro/Balatro"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires an explicitly provided local installation"]
    fn imports_partial_lot_independently_of_source() {
        let reference = PathBuf::from(std::env::var_os("ANTE_TEST_ASSET_DIR").unwrap());
        let root = std::env::temp_dir().join(format!("ante-assets-partial-{}", std::process::id()));
        let source = root.join("Balatro with spaces");
        let private = root.join("private");
        let relative = "resources/textures/1x/boosters.png";
        fs::create_dir_all(source.join("resources/textures/1x")).unwrap();
        fs::copy(reference.join(relative), source.join(relative)).unwrap();
        let mut settings = AssetSettings::from_root(private.clone());
        settings.source = source.to_string_lossy().into();
        assert_eq!(settings.import().unwrap(), 1);
        fs::remove_dir_all(source).unwrap();
        let loaded = AssetSettings::from_root(private).active.unwrap();
        assert!(loaded.join(relative).is_file());
        settings.remove().unwrap();
        assert!(!loaded.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_import_preserves_active_installation() {
        let root = std::env::temp_dir().join(format!("ante-assets-test-{}", std::process::id()));
        let source = root.join("Balatro with spaces");
        let store = root.join("private");
        fs::create_dir_all(source.join("resources/textures/1x")).unwrap();
        fs::create_dir_all(store.join("lot-1-2")).unwrap();
        fs::write(
            store.join("active.json"),
            r#"{"version":1,"lot":"lot-1-2","source":"prior"}"#,
        )
        .unwrap();
        fs::write(
            source.join("resources/textures/1x/boosters.png"),
            b"invalid",
        )
        .unwrap();
        let mut settings = AssetSettings::from_root(store.clone());
        let old = settings.active.clone();
        settings.source = source.to_string_lossy().into();
        assert!(settings.import().is_err());
        assert_eq!(AssetSettings::from_root(store).active, old);
        settings.remove().unwrap();
        assert!(settings.active.is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
