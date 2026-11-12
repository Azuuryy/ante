//! Original sound effects, read from the user's local installation and never shipped.
//!
//! Like the original `play_sound(name, per, vol)`, a request names a sound, a pitch and a
//! volume; the final volume is `vol × master/100 × game sounds/100`. A missing file keeps
//! the game silent for that sound.

use std::{collections::HashMap, path::Path};

use bevy::{
    audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume},
    prelude::*,
};
#[cfg(feature = "dev-tools")]
use serde_json::json;

use super::catalog::{AtlasSource, read_local_file};
use crate::services::profile::Profile;

/// Sound effects the importer may copy and the game may play.
pub const SOUND_FILES: &[&str] = &[
    "button",
    "cancel",
    "card1",
    "cardFan2",
    "cardSlide1",
    "cardSlide2",
    "chips1",
    "chips2",
    "coin1",
    "coin2",
    "coin3",
    "crumple1",
    "crumple2",
    "crumple3",
    "crumple4",
    "crumple5",
    "foil1",
    "generic1",
    "highlight1",
    "highlight2",
    "holo1",
    "multhit1",
    "multhit2",
    "negative",
    "paper1",
    "polychrome1",
    "tarot1",
    "whoosh2",
];

pub fn sound_path(name: &str) -> String {
    format!("resources/sounds/{name}.ogg")
}

/// Import check: an Ogg stream starts with its capture pattern.
pub fn validate(bytes: &[u8]) -> std::io::Result<()> {
    if bytes.starts_with(b"OggS") {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "not an Ogg file",
        ))
    }
}

/// Sounds loaded from the local installation, by original file name.
#[derive(Resource, Default)]
pub struct SoundBank {
    loaded: HashMap<&'static str, Handle<AudioSource>>,
}

/// A request to play one original sound, like `play_sound(name, per, vol)`.
#[derive(Message, Clone, Copy, Debug)]
pub struct Sfx {
    pub name: &'static str,
    pub pitch: f32,
    pub volume: f32,
}

impl Sfx {
    pub const fn new(name: &'static str, pitch: f32, volume: f32) -> Self {
        Self {
            name,
            pitch,
            volume,
        }
    }
}

/// Startup: reads every available sound; a missing one stays silent.
pub fn load(
    mut commands: Commands,
    source: Res<AtlasSource>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    let mut bank = SoundBank::default();
    if let Some(root) = source.0.as_deref() {
        for &name in SOUND_FILES {
            match read_sound(root, name) {
                Ok(bytes) => {
                    let handle = assets.add(AudioSource {
                        bytes: bytes.into(),
                    });
                    bank.loaded.insert(name, handle);
                }
                Err(_error) => {
                    crate::journal_event!(
                        "sound_fallback",
                        json!({"file": name, "error": _error.to_string()})
                    );
                }
            }
        }
        crate::journal_event!("sounds_loaded", json!({"count": bank.loaded.len()}));
    }
    commands.insert_resource(bank);
}

fn read_sound(root: &Path, name: &str) -> std::io::Result<Vec<u8>> {
    let bytes = read_local_file(root, &sound_path(name))?;
    validate(&bytes)?;
    Ok(bytes)
}

/// Plays requested sounds at the volumes saved in the profile.
pub fn play(
    mut commands: Commands,
    mut requests: MessageReader<Sfx>,
    bank: Option<Res<SoundBank>>,
    profile: Res<Profile>,
) {
    let Some(bank) = bank else {
        requests.clear();
        return;
    };
    let master = profile.volume() as f32 / 100.0 * profile.game_volume() as f32 / 100.0;
    for request in requests.read() {
        let Some(handle) = bank.loaded.get(request.name) else {
            continue;
        };
        let volume = request.volume * master;
        if volume <= 0.001 {
            continue;
        }
        commands.spawn((
            AudioPlayer(handle.clone()),
            PlaybackSettings::DESPAWN
                .with_speed(request.pitch)
                .with_volume(Volume::Linear(volume)),
        ));
    }
}

/// Original pitch jitter: `0.9 + 0.2 × random`.
pub fn jitter(base: f32, spread: f32) -> f32 {
    base + spread * rand::random::<f32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_ogg_streams_are_accepted() {
        assert!(validate(b"OggS\0\x02").is_ok());
        assert!(validate(b"RIFF....WAVE").is_err());
    }

    #[test]
    fn sound_paths_follow_the_installation_layout() {
        assert_eq!(sound_path("card1"), "resources/sounds/card1.ogg");
    }
}
