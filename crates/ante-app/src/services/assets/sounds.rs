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
    "explosion_buildup1",
    "explosion_release1",
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
    "tarot2",
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
    use crate::services::profile::VolumeChannel::{Game, Master};
    let master = profile.volume_of(Master) as f32 / 100.0 * profile.volume_of(Game) as f32 / 100.0;
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

/// Sounds due later, like a `play_sound` inside a delayed event: (real time due, sound).
#[derive(Resource, Default)]
pub struct ScheduledSfx {
    due: Vec<(f32, Sfx)>,
}

impl ScheduledSfx {
    /// Plays `sound` `delay` real seconds after `now`.
    pub fn after(&mut self, now: f32, delay: f32, sound: Sfx) {
        self.due.push((now + delay, sound));
    }

    /// Takes the sounds due at `now`, in the order they were scheduled.
    fn take_due(&mut self, now: f32) -> Vec<Sfx> {
        let (ready, waiting) = self.due.drain(..).partition(|(at, _)| *at <= now);
        self.due = waiting;
        ready.into_iter().map(|(_, sound)| sound).collect()
    }
}

/// Sends the scheduled sounds whose time has come.
pub fn release_scheduled(
    time: Res<Time<Real>>,
    mut scheduled: ResMut<ScheduledSfx>,
    mut sounds: MessageWriter<Sfx>,
) {
    if scheduled.due.is_empty() {
        return;
    }
    for sound in scheduled.take_due(time.elapsed_secs()) {
        sounds.write(sound);
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
    fn scheduled_sounds_wait_for_their_time() {
        let mut scheduled = ScheduledSfx::default();
        scheduled.after(1.0, 0.5, Sfx::new("explosion_buildup1", 1.0, 1.0));
        scheduled.after(1.0, 2.0, Sfx::new("explosion_release1", 1.0, 1.0));
        assert!(scheduled.take_due(1.4).is_empty());
        let first = scheduled.take_due(1.5);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].name, "explosion_buildup1");
        assert_eq!(scheduled.take_due(3.0)[0].name, "explosion_release1");
        assert!(scheduled.due.is_empty());
    }

    #[test]
    fn sound_paths_follow_the_installation_layout() {
        assert_eq!(sound_path("card1"), "resources/sounds/card1.ogg");
    }
}
