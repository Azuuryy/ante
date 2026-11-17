use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use serde_json::Value;
use serde_json::json;

mod progress;
pub(crate) use progress::Progress;

/// Version 5 adds the progress (unlocked items, career statistics, deck wins); older
/// profiles start with none, so what the original locks is locked. Version 4 added the
/// music volume; version 3 profiles read it as 100.
const VERSION: u64 = 5;

/// Legendary Jokers are found through The Soul, never unlocked by a condition: they
/// count as unlocked once discovered.
const LEGENDARIES: [&str; 5] = ["j_caino", "j_triboulet", "j_yorick", "j_chicot", "j_perkeo"];

#[derive(Default)]
struct Stored {
    discovered: BTreeSet<String>,
    all_unlocked: bool,
    game_speed: Option<f32>,
    volumes: Volumes,
    progress: Progress,
}

/// Original sound settings (`G.SETTINGS.SOUND`): master volume 50, music and game sounds
/// 100 by default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Volumes {
    master: u8,
    music: u8,
    game: u8,
}

impl Default for Volumes {
    fn default() -> Self {
        Self {
            master: 50,
            music: 100,
            game: 100,
        }
    }
}

/// A volume of the Audio settings tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeChannel {
    /// `SETTINGS.SOUND.volume`.
    Master,
    /// `SETTINGS.SOUND.music_volume`.
    Music,
    /// `SETTINGS.SOUND.game_sounds_volume`.
    Game,
}

#[derive(Resource)]
pub struct Profile {
    path: PathBuf,
    discovered: BTreeSet<String>,
    all_unlocked: bool,
    game_speed: Option<f32>,
    volumes: Volumes,
    progress: Progress,
    writable: bool,
    dirty: bool,
}

impl Profile {
    pub fn load() -> Self {
        let path = crate::services::storage::local_dir().join("profile.json");
        match Self::read(&path) {
            Ok(stored) => Self {
                path,
                discovered: stored.discovered,
                all_unlocked: stored.all_unlocked,
                game_speed: stored.game_speed,
                volumes: stored.volumes,
                progress: stored.progress,
                writable: true,
                dirty: false,
            },
            Err(_error) => {
                crate::journal_event!("profile_invalid", json!({"error": _error.to_string()}));
                Self {
                    path,
                    discovered: BTreeSet::new(),
                    all_unlocked: false,
                    game_speed: None,
                    volumes: Volumes::default(),
                    progress: Progress::default(),
                    writable: false,
                    dirty: false,
                }
            }
        }
    }

    fn read(path: &Path) -> io::Result<Stored> {
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Stored::default()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(64 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 {
            return Err(io::Error::other("Profile exceeds size limit"));
        }
        let mut data: Value = serde_json::from_slice(&bytes)?;
        let version = data.get("version").and_then(Value::as_u64);
        let discovered = match data.get_mut("discovered").map(Value::take) {
            Some(value) => serde_json::from_value(value)?,
            None => BTreeSet::new(),
        };
        let all_unlocked = data
            .get("all_unlocked")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let game_speed = data
            .get("game_speed")
            .and_then(Value::as_f64)
            .map(|speed| speed as f32);
        let volume = |key: &str, default: u8| {
            data.get(key)
                .and_then(Value::as_u64)
                .map_or(default, |value| value.min(100) as u8)
        };
        let defaults = Volumes::default();
        let volumes = Volumes {
            master: volume("volume", defaults.master),
            music: volume("music_volume", defaults.music),
            game: volume("game_volume", defaults.game),
        };
        let progress = Progress::read(&mut data);
        match version {
            Some(1) => Ok(Stored {
                discovered,
                all_unlocked: false,
                game_speed: None,
                volumes: Volumes::default(),
                progress: Progress::default(),
            }),
            Some(2) => Ok(Stored {
                discovered,
                all_unlocked,
                game_speed: None,
                volumes: Volumes::default(),
                progress: Progress::default(),
            }),
            Some(3 | 4) => Ok(Stored {
                discovered,
                all_unlocked,
                game_speed,
                volumes,
                progress: Progress::default(),
            }),
            Some(5) => Ok(Stored {
                discovered,
                all_unlocked,
                game_speed,
                volumes,
                progress,
            }),
            _ => Err(io::Error::other("Unsupported profile version")),
        }
    }

    pub fn discover(&mut self, id: &str) -> io::Result<()> {
        self.dirty |= self.discovered.insert(id.to_owned());
        if !self.dirty {
            return Ok(());
        }
        if !self.writable {
            return Err(io::Error::other(
                "Existing profile is unreadable and was preserved",
            ));
        }
        self.save()?;
        self.dirty = false;
        Ok(())
    }

    /// Debug equivalent of Balatro's "Unlock All": every catalog entry becomes
    /// discovered and the profile remembers that everything is unlocked.
    pub fn unlock_all(&mut self, ids: impl IntoIterator<Item = &'static str>) -> io::Result<()> {
        let mut changed = !self.all_unlocked;
        self.all_unlocked = true;
        for id in ids {
            changed |= self.discovered.insert(id.to_owned());
        }
        if !changed {
            return Ok(());
        }
        if !self.writable {
            return Err(io::Error::other(
                "Existing profile is unreadable and was preserved",
            ));
        }
        self.save()?;
        self.dirty = false;
        Ok(())
    }

    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let pending = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        let mut data = json!({
            "version": VERSION,
            "discovered": self.discovered,
            "all_unlocked": self.all_unlocked,
            "game_speed": self.game_speed,
            "volume": self.volumes.master,
            "music_volume": self.volumes.music,
            "game_volume": self.volumes.game,
        });
        self.progress.write(&mut data);
        fs::write(&pending, serde_json::to_vec(&data)?)?;
        fs::rename(pending, &self.path)
    }

    /// Saved score animation speed factor, the original `GAMESPEED` setting.
    pub fn game_speed(&self) -> Option<f32> {
        self.game_speed
    }

    pub fn set_game_speed(&mut self, speed: f32) -> io::Result<()> {
        if self.game_speed == Some(speed) {
            return Ok(());
        }
        self.game_speed = Some(speed);
        if !self.writable {
            return Err(io::Error::other(
                "Existing profile is unreadable and was preserved",
            ));
        }
        self.save()
    }

    /// A volume from 0 to 100.
    pub fn volume_of(&self, channel: VolumeChannel) -> u8 {
        match channel {
            VolumeChannel::Master => self.volumes.master,
            VolumeChannel::Music => self.volumes.music,
            VolumeChannel::Game => self.volumes.game,
        }
    }

    /// Sets a volume (capped at 100) without saving, as a slider does while it is dragged.
    pub fn set_volume(&mut self, channel: VolumeChannel, value: u8) {
        let slot = match channel {
            VolumeChannel::Master => &mut self.volumes.master,
            VolumeChannel::Music => &mut self.volumes.music,
            VolumeChannel::Game => &mut self.volumes.game,
        };
        *slot = value.min(100);
    }

    /// Saves the volumes, once a slider is released.
    pub fn save_volumes(&mut self) -> io::Result<()> {
        if !self.writable {
            return Err(io::Error::other(
                "Existing profile is unreadable and was preserved",
            ));
        }
        self.save()
    }

    pub fn discovered(&self, id: &str) -> bool {
        self.discovered.contains(id)
    }

    /// Whether `id` is unlocked: it has no condition, its condition was met, "Unlock all"
    /// was used, or it is a Legendary already found.
    pub fn is_unlocked(&self, id: &str, condition: Option<&str>) -> bool {
        self.all_unlocked
            || condition.is_none()
            || self.progress.unlocked.contains(id)
            || (LEGENDARIES.contains(&id) && self.discovered(id))
    }

    pub(crate) fn progress(&self) -> &Progress {
        &self.progress
    }

    /// Changes the progress and saves it when something changed.
    pub(crate) fn update_progress(&mut self, change: impl FnOnce(&mut Progress)) -> io::Result<()> {
        let before = self.progress.clone();
        change(&mut self.progress);
        if self.progress == before {
            return Ok(());
        }
        if !self.writable {
            return Err(io::Error::other(
                "Existing profile is unreadable and was preserved",
            ));
        }
        self.save()
    }

    pub fn all_unlocked(&self) -> bool {
        self.all_unlocked
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }

    /// A fresh profile saved at `path`, for tests.
    #[cfg(test)]
    pub(crate) fn fresh_at(path: PathBuf) -> Self {
        Self {
            path,
            discovered: BTreeSet::new(),
            all_unlocked: false,
            game_speed: None,
            volumes: Volumes::default(),
            progress: Progress::default(),
            writable: true,
            dirty: false,
        }
    }
}

#[cfg(test)]
mod tests;
