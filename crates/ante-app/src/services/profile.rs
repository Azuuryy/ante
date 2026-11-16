use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use serde_json::Value;
use serde_json::json;

/// Version 4 adds the music volume; version 3 profiles read it as 100.
const VERSION: u64 = 4;

#[derive(Default)]
struct Stored {
    discovered: BTreeSet<String>,
    all_unlocked: bool,
    game_speed: Option<f32>,
    volumes: Volumes,
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
        match version {
            Some(1) => Ok(Stored {
                discovered,
                all_unlocked: false,
                game_speed: None,
                volumes: Volumes::default(),
            }),
            Some(2) => Ok(Stored {
                discovered,
                all_unlocked,
                game_speed: None,
                volumes: Volumes::default(),
            }),
            Some(3 | 4) => Ok(Stored {
                discovered,
                all_unlocked,
                game_speed,
                volumes,
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
        let data = json!({
            "version": VERSION,
            "discovered": self.discovered,
            "all_unlocked": self.all_unlocked,
            "game_speed": self.game_speed,
            "volume": self.volumes.master,
            "music_volume": self.volumes.music,
            "game_volume": self.volumes.game,
        });
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

    /// Whether the collection may show the real art for this item. Discovery is
    /// the only in-game unlock trigger implemented so far, so it doubles as an
    /// unlock when the debug "Unlock All" flag is off.
    pub fn is_unlocked(&self, id: &str, condition: Option<&str>) -> bool {
        self.all_unlocked || condition.is_none() || self.discovered(id)
    }

    #[cfg_attr(not(feature = "dev-tools"), allow(dead_code))]
    pub fn all_unlocked(&self) -> bool {
        self.all_unlocked
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }
}

#[cfg(test)]
mod tests;
