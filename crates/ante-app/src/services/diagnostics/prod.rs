use std::time::Duration;

use bevy::{
    prelude::*,
    window::{PresentMode, PrimaryWindow},
};

#[derive(Resource, Default)]
pub struct FrameMetrics;

impl FrameMetrics {
    pub fn record_simulation(&mut self, _: Duration, _: Duration) {}
    pub fn record_ui(&mut self, _: Duration) {}
    pub fn record_playback(&mut self, _: Duration) {}
    pub fn record_assets(&mut self, _: Duration) {}
}

pub struct Measurement;

pub fn measurement() -> Measurement {
    Measurement
}

impl Measurement {
    pub fn elapsed(self) -> Duration {
        Duration::ZERO
    }
}

#[derive(Resource, Default)]
pub struct FramePacing {
    immediate: bool,
}

impl FramePacing {
    pub fn new(_: bool) -> Self {
        Self::default()
    }

    pub fn present_mode(&self) -> PresentMode {
        if self.immediate {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::Fifo
        }
    }

    pub fn label(&self) -> &'static str {
        if self.immediate { "No VSync" } else { "VSync" }
    }
}

#[derive(Message)]
pub struct ToggleFramePacing;

pub struct PerformancePlugin;

impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameMetrics>()
            .init_resource::<FramePacing>()
            .add_message::<ToggleFramePacing>()
            .add_systems(
                Last,
                |mut events: MessageReader<ToggleFramePacing>,
                 mut pacing: ResMut<FramePacing>,
                 mut windows: Query<&mut Window, With<PrimaryWindow>>| {
                    if events.read().count() % 2 == 1 {
                        pacing.immediate = !pacing.immediate;
                        for mut window in &mut windows {
                            window.present_mode = pacing.present_mode();
                        }
                    }
                },
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_does_not_open_a_profiler_window() {
        let mut app = App::new();
        app.add_plugins(PerformancePlugin);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F3);
        app.update();
        assert_eq!(
            app.world_mut().query::<&Window>().iter(app.world()).count(),
            0
        );
    }
}
