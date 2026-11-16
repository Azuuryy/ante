//! The original soundtrack, read from the user's local installation and never shipped.
//!
//! Like `modulate_sound` and the sound manager (`SET_SFX`, `RESTART_MUSIC`), the five tracks
//! loop together at pitch 0.7 and volume 0.6; the desired track fades up and the others
//! fade out at `3·dt`, so changing screens crossfades between them. A missing track stays
//! silent.

use std::path::Path;

use ante_core::contract::{PackFamily, Phase};
use bevy::{
    audio::{AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume},
    prelude::*,
};

use super::{
    catalog::{AtlasSource, read_local_file},
    sounds::{sound_path, validate},
};
use crate::{
    app::{navigation::Screen, session::Session},
    services::profile::{Profile, VolumeChannel},
};

/// Music tracks the importer may copy and the game may play.
pub const MUSIC_FILES: &[&str] = &["music1", "music2", "music3", "music4", "music5"];

/// `RESTART_MUSIC` plays every track at `per = 0.7` and `vol = 0.6`.
const MUSIC_PITCH: f32 = 0.7;
const MUSIC_VOLUME: f32 = 0.6;

/// One looping track.
#[derive(Component)]
pub struct MusicTrack {
    index: usize,
}

/// The crossfade state: each track's `current_volume` and `G.PITCH_MOD`.
#[derive(Resource)]
pub struct MusicMix {
    volumes: [f32; 5],
    pitch_mod: f32,
    started: bool,
}

impl Default for MusicMix {
    fn default() -> Self {
        Self {
            volumes: [0.0; 5],
            pitch_mod: 1.0,
            started: false,
        }
    }
}

impl MusicMix {
    /// One frame of `SET_SFX`: the desired track (index) eases toward 1, the others toward
    /// 0, and the pitch toward 0.5 on a game over or 1 otherwise. The first frame starts the
    /// desired track at full volume, as a source created for it does.
    pub fn step(&mut self, desired: usize, game_over: bool, dt: f32) {
        let k = (3.0 * dt).min(1.0);
        for (index, volume) in self.volumes.iter_mut().enumerate() {
            let target = if index == desired { 1.0 } else { 0.0 };
            *volume = if self.started {
                target * k + (1.0 - k) * *volume
            } else {
                target
            };
        }
        self.started = true;
        let pitch = if game_over { 0.5 } else { 1.0 };
        let k = dt.min(1.0);
        self.pitch_mod = self.pitch_mod * (1.0 - k) + k * pitch;
    }

    /// Output volume of a track: `current × 0.6 × master/100 × music/100`.
    pub fn output(&self, index: usize, master: u8, music: u8) -> f32 {
        self.volumes[index] * MUSIC_VOLUME * f32::from(master) / 100.0 * f32::from(music) / 100.0
    }

    pub fn pitch(&self) -> f32 {
        MUSIC_PITCH * self.pitch_mod
    }
}

/// What the run shows, for the track choice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicScene {
    pub in_run: bool,
    pub phase: Phase,
    pub pack: Option<PackFamily>,
    pub boss: bool,
}

/// `modulate_sound`'s `desired_track`, as an index into [`MUSIC_FILES`]: Arcana and
/// Spectral packs (sparkles) and other packs play music2, Celestial packs (meteors) music3,
/// the shop music4, a boss blind music5, and everything else music1.
pub fn desired_track(scene: MusicScene) -> usize {
    if !scene.in_run {
        return 0;
    }
    match scene.phase {
        Phase::Pack => match scene.pack {
            Some(PackFamily::Celestial) => 2,
            _ => 1,
        },
        Phase::Shop => 3,
        Phase::Playing if scene.boss => 4,
        _ => 0,
    }
}

fn scene(session: &Session) -> MusicScene {
    let navigation = &session.navigation;
    let in_run = navigation.screen == Screen::Run || navigation.overlays_run();
    match session.confirmed.as_ref().filter(|_| in_run) {
        Some(view) => MusicScene {
            in_run: true,
            phase: view.phase,
            pack: view.open_pack,
            boss: view
                .blind_chip
                .is_some_and(|id| !matches!(id, "bl_small" | "bl_big")),
        },
        None => MusicScene {
            in_run: false,
            phase: Phase::BeforeBlind,
            pack: None,
            boss: false,
        },
    }
}

/// Startup: starts every available track, silent until [`modulate_music`] fades it in.
pub fn load(
    mut commands: Commands,
    source: Res<AtlasSource>,
    mut assets: ResMut<Assets<AudioSource>>,
) {
    commands.init_resource::<MusicMix>();
    let Some(root) = source.0.as_deref() else {
        return;
    };
    for (index, &name) in MUSIC_FILES.iter().enumerate() {
        match read_track(root, name) {
            Ok(bytes) => {
                let handle = assets.add(AudioSource {
                    bytes: bytes.into(),
                });
                commands.spawn((
                    AudioPlayer(handle),
                    PlaybackSettings::LOOP
                        .with_speed(MUSIC_PITCH)
                        .with_volume(Volume::Linear(0.0)),
                    MusicTrack { index },
                ));
            }
            Err(_error) => {
                crate::journal_event!(
                    "music_fallback",
                    serde_json::json!({"file": name, "error": _error.to_string()})
                );
            }
        }
    }
}

fn read_track(root: &Path, name: &str) -> std::io::Result<Vec<u8>> {
    let bytes = read_local_file(root, &sound_path(name))?;
    validate(&bytes)?;
    Ok(bytes)
}

/// Every frame: crossfades toward the desired track and applies the volumes and pitch.
pub fn modulate_music(
    time: Res<Time<Real>>,
    session: Res<Session>,
    profile: Res<Profile>,
    mix: Option<ResMut<MusicMix>>,
    mut tracks: Query<(&MusicTrack, &mut AudioSink)>,
) {
    let Some(mut mix) = mix else {
        return;
    };
    let scene = scene(&session);
    mix.step(
        desired_track(scene),
        scene.in_run && scene.phase == Phase::Lost,
        time.delta_secs(),
    );
    let master = profile.volume_of(VolumeChannel::Master);
    let music = profile.volume_of(VolumeChannel::Music);
    for (track, mut sink) in &mut tracks {
        sink.set_volume(Volume::Linear(mix.output(track.index, master, music)));
        sink.set_speed(mix.pitch());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(phase: Phase) -> MusicScene {
        MusicScene {
            in_run: true,
            phase,
            pack: None,
            boss: false,
        }
    }

    #[test]
    fn each_screen_has_its_original_track() {
        let menu = MusicScene {
            in_run: false,
            ..run(Phase::Shop)
        };
        assert_eq!(desired_track(menu), 0);
        assert_eq!(desired_track(run(Phase::BeforeBlind)), 0);
        assert_eq!(desired_track(run(Phase::Playing)), 0);
        let boss = MusicScene {
            boss: true,
            ..run(Phase::Playing)
        };
        assert_eq!(desired_track(boss), 4);
        assert_eq!(desired_track(run(Phase::Shop)), 3);
        for (family, track) in [
            (PackFamily::Arcana, 1),
            (PackFamily::Spectral, 1),
            (PackFamily::Celestial, 2),
            (PackFamily::Buffoon, 1),
            (PackFamily::Standard, 1),
        ] {
            let pack = MusicScene {
                pack: Some(family),
                ..run(Phase::Pack)
            };
            assert_eq!(desired_track(pack), track, "{family:?}");
        }
    }

    #[test]
    fn tracks_crossfade_at_three_dt() {
        let mut mix = MusicMix::default();
        mix.step(0, false, 0.1);
        assert_eq!(mix.volumes, [1.0, 0.0, 0.0, 0.0, 0.0]);
        mix.step(3, false, 0.1);
        assert!((mix.volumes[0] - 0.7).abs() < 1e-6);
        assert!((mix.volumes[3] - 0.3).abs() < 1e-6);
        assert!((mix.output(3, 50, 100) - 0.3 * 0.6 * 0.5).abs() < 1e-6);
        assert_eq!(mix.output(3, 50, 0), 0.0);
    }

    #[test]
    fn a_game_over_slows_the_music_toward_half_pitch() {
        let mut mix = MusicMix::default();
        assert!((mix.pitch() - 0.7).abs() < 1e-6);
        mix.step(0, true, 0.5);
        assert!((mix.pitch_mod - 0.75).abs() < 1e-6);
        for _ in 0..100 {
            mix.step(0, true, 0.5);
        }
        assert!((mix.pitch() - 0.35).abs() < 1e-4);
    }
}
