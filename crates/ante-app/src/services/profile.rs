use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use bevy::prelude::*;
use serde_json::Value;
use serde_json::json;

const VERSION: u64 = 3;

#[derive(Default)]
struct Stored {
    discovered: BTreeSet<String>,
    all_unlocked: bool,
    game_speed: Option<f32>,
    volumes: Volumes,
}

/// Original sound settings: master volume 50 and game sounds 100 by default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Volumes {
    master: u8,
    game: u8,
}

impl Default for Volumes {
    fn default() -> Self {
        Self {
            master: 50,
            game: 100,
        }
    }
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
            Some(3) => Ok(Stored {
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

    /// Master volume, 0 to 100, like the original `SETTINGS.SOUND.volume`.
    pub fn volume(&self) -> u8 {
        self.volumes.master
    }

    /// Game sounds volume, 0 to 100, like `SETTINGS.SOUND.game_sounds_volume`.
    pub fn game_volume(&self) -> u8 {
        self.volumes.game
    }

    /// Steps a volume by 10 and wraps from 100 back to 0, then saves it.
    pub fn step_volume(&mut self, game: bool) -> io::Result<()> {
        let value = if game {
            &mut self.volumes.game
        } else {
            &mut self.volumes.master
        };
        *value = if *value >= 100 {
            0
        } else {
            (*value + 10).min(100)
        };
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
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ante-{name}-{}.json", std::process::id()))
    }

    fn profile_at(path: PathBuf) -> Profile {
        Profile {
            path,
            discovered: BTreeSet::new(),
            all_unlocked: false,
            game_speed: None,
            volumes: Volumes::default(),
            writable: true,
            dirty: false,
        }
    }

    #[test]
    fn volumes_default_to_the_original_and_step_by_ten() {
        let path = temporary("profile-volume");
        let mut profile = profile_at(path.clone());
        assert_eq!((profile.volume(), profile.game_volume()), (50, 100));
        profile.step_volume(false).unwrap();
        profile.step_volume(true).unwrap();
        let stored = Profile::read(&path).unwrap();
        assert_eq!(
            stored.volumes,
            Volumes {
                master: 60,
                game: 0
            }
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn game_speed_survives_a_save_and_older_versions_have_none() {
        let path = temporary("profile-speed");
        let mut profile = profile_at(path.clone());
        profile.set_game_speed(4.0).unwrap();
        assert_eq!(Profile::read(&path).unwrap().game_speed, Some(4.0));
        fs::write(&path, r#"{"version":2,"discovered":[],"game_speed":8}"#).unwrap();
        assert_eq!(Profile::read(&path).unwrap().game_speed, None);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn versioned_profile_saves_discoveries_across_writes() {
        let path = temporary("profile");
        let mut profile = profile_at(path.clone());
        profile.discover("j_joker").unwrap();
        profile.discover("j_misprint").unwrap();
        let stored = Profile::read(&path).unwrap();
        assert_eq!(
            stored.discovered,
            BTreeSet::from(["j_joker".into(), "j_misprint".into()])
        );
        assert!(!stored.all_unlocked);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn version_one_profiles_migrate_without_losing_discoveries() {
        let path = temporary("profile-v1");
        fs::write(&path, r#"{"version":1,"discovered":["j_joker","v_blank"]}"#).unwrap();
        let stored = Profile::read(&path).unwrap();
        assert_eq!(
            stored.discovered,
            BTreeSet::from(["j_joker".into(), "v_blank".into()])
        );
        assert!(!stored.all_unlocked);

        let mut profile = Profile {
            path: path.clone(),
            discovered: stored.discovered,
            all_unlocked: stored.all_unlocked,
            game_speed: stored.game_speed,
            volumes: stored.volumes,
            writable: true,
            dirty: false,
        };
        profile.discover("j_brainstorm").unwrap();
        let rewritten: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(rewritten["version"], VERSION);
        assert_eq!(rewritten["all_unlocked"], false);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn unlock_all_persists_flag_and_catalog_entries() {
        let path = temporary("profile-unlock");
        let mut profile = profile_at(path.clone());
        profile
            .unlock_all(["j_brainstorm", "v_blank", "tag_rare"])
            .unwrap();
        assert!(profile.all_unlocked());
        assert!(profile.discovered("j_brainstorm"));
        assert!(profile.is_unlocked("j_any", Some("Locked")));

        let stored = Profile::read(&path).unwrap();
        assert!(stored.all_unlocked);
        assert_eq!(
            stored.discovered,
            BTreeSet::from(["j_brainstorm".into(), "v_blank".into(), "tag_rare".into()])
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn unlocked_items_require_discovery_without_the_debug_flag() {
        let profile = profile_at(temporary("profile-locks"));
        assert!(profile.is_unlocked("j_joker", None));
        assert!(!profile.is_unlocked("j_brainstorm", Some("Discard a Royal Flush")));
    }

    #[test]
    fn corrupted_profile_is_rejected() {
        let path = temporary("invalid-profile");
        fs::write(&path, "not JSON").unwrap();
        assert!(Profile::read(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
