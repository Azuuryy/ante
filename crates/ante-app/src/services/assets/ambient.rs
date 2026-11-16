//! The fire and organ ambience under big scores, read from the user's local installation
//! and never shipped. Like `modulate_sound` and `AMBIENT`, each loop eases its volume
//! toward a target set by the score flames (`components::flames::ScoreIntensity`):
//! `ambientFire2` swells as soon as the flames burn, `ambientFire1` once they pass 0.3,
//! `ambientFire3` while they grow, and `ambientOrgan1` with the margin over the blind.

use std::path::Path;

use bevy::{
    audio::{AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume},
    prelude::*,
};

use super::{
    catalog::{AtlasSource, read_local_file},
    sounds::{sound_path, validate},
};
use crate::{
    components::flames::ScoreIntensity,
    services::profile::{Profile, VolumeChannel},
};

/// Ambient loops the importer may copy and the game may play.
pub const AMBIENT_FILES: &[&str] = &[
    "ambientFire1",
    "ambientFire2",
    "ambientFire3",
    "ambientOrgan1",
];

/// Pitch of each loop (`AC[k].per`).
const PITCHES: [f32; 4] = [1.1, 1.05, 1.0, 0.7];

/// One looping ambience, by its index in [`AMBIENT_FILES`].
#[derive(Component)]
pub struct AmbientTrack(usize);

/// Target volume of each loop for a score intensity, before the volume settings:
/// `volfunc` of `G.ARGS.ambient_sounds`.
pub fn ambient_targets(intensity: ScoreIntensity, music_volume: u8) -> [f32; 4] {
    let flames = intensity.flames;
    [
        0.8 * if flames > 0.3 {
            (flames - 0.3) / 0.7
        } else {
            0.0
        },
        0.9 * if flames > 0.3 { 1.0 } else { flames / 0.3 },
        0.4 * intensity.change,
        0.6 * (f32::from(music_volume) + 100.0) / 200.0 * intensity.organ,
    ]
}

/// Startup: starts every available loop, silent until the score burns.
pub fn load(
    mut commands: Commands,
    source: Res<AtlasSource>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    let Some(root) = source.0.as_deref() else {
        return;
    };
    for (index, &name) in AMBIENT_FILES.iter().enumerate() {
        match read_loop(root, name) {
            Ok(bytes) => {
                let handle = assets.add(AudioSource {
                    bytes: bytes.into(),
                });
                commands.spawn((
                    AudioPlayer(handle),
                    PlaybackSettings::LOOP
                        .with_speed(PITCHES[index])
                        .with_volume(Volume::Linear(0.0)),
                    AmbientTrack(index),
                ));
            }
            Err(_error) => {
                crate::journal_event!(
                    "ambient_fallback",
                    serde_json::json!({"file": name, "error": _error.to_string()})
                );
            }
        }
    }
}

fn read_loop(root: &Path, name: &str) -> std::io::Result<Vec<u8>> {
    let bytes = read_local_file(root, &sound_path(name))?;
    validate(&bytes)?;
    Ok(bytes)
}

/// Every frame: `vol = vol × (1 − dt) + dt × target`, played at
/// `vol × master/100 × game sounds/100` like `SET_SFX`.
pub fn modulate_ambient(
    time: Res<Time<Real>>,
    intensity: Res<ScoreIntensity>,
    profile: Res<Profile>,
    mut volumes: Local<[f32; 4]>,
    mut tracks: Query<(&AmbientTrack, &mut AudioSink)>,
) {
    let dt = time.delta_secs().min(1.0);
    let targets = ambient_targets(*intensity, profile.volume_of(VolumeChannel::Music));
    for (volume, target) in volumes.iter_mut().zip(targets) {
        *volume = *volume * (1.0 - dt) + dt * target;
    }
    let gain = f32::from(profile.volume_of(VolumeChannel::Master)) / 100.0
        * f32::from(profile.volume_of(VolumeChannel::Game))
        / 100.0;
    for (track, mut sink) in &mut tracks {
        sink.set_volume(Volume::Linear(volumes[track.0] * gain));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fire_swells_with_the_flames_and_the_organ_with_the_margin() {
        let quiet = ambient_targets(ScoreIntensity::default(), 100);
        assert_eq!(quiet, [0.0; 4]);
        let low = ambient_targets(
            ScoreIntensity {
                flames: 0.15,
                organ: 0.0,
                change: 0.0,
            },
            100,
        );
        assert_eq!(low[0], 0.0, "ambientFire1 waits for flames above 0.3");
        assert!((low[1] - 0.45).abs() < 1e-6);
        let hot = ambient_targets(
            ScoreIntensity {
                flames: 1.0,
                organ: 0.4,
                change: 1.0,
            },
            100,
        );
        assert!((hot[0] - 0.8).abs() < 1e-6);
        assert!((hot[1] - 0.9).abs() < 1e-6);
        assert!((hot[2] - 0.4).abs() < 1e-6);
        assert!((hot[3] - 0.24).abs() < 1e-6);
    }
}
