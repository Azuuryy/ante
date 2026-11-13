use std::{
    collections::VecDeque,
    fmt::Write as _,
    time::{Duration, Instant},
};

use bevy::{
    camera::RenderTarget,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
    text::FontSize,
    time::TimeSystems,
    ui::UiSystems,
    window::{PresentMode, WindowRef},
};
use serde_json::json;

use super::journal;

mod panel;

use panel::*;

const HISTORY_MS: f64 = 10_000.0;
const MAX_SAMPLES: usize = 10_000;
const GRAPH_BARS: usize = 120;
const MIN_FRAME_INTERVAL: Duration = Duration::from_millis(1);
const REFRESH: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Default)]
struct Frame {
    ms: f64,
    simulation: f64,
    trace: f64,
    playback: f64,
    ui: f64,
    layout: f64,
    assets: f64,
    profiler: f64,
}

#[derive(Resource, Default)]
pub struct FrameMetrics {
    frames: VecDeque<Frame>,
    history_ms: f64,
    pending: Frame,
    previous: Option<Frame>,
    layout_start: Option<Instant>,
    passes: Vec<(String, f64)>,
    startup_assets_ms: Option<f64>,
}

impl FrameMetrics {
    pub fn record_simulation(&mut self, total: Duration, trace: Duration) {
        self.pending.simulation += milliseconds(total.saturating_sub(trace));
        self.pending.trace += milliseconds(trace);
    }

    pub fn record_ui(&mut self, elapsed: Duration) {
        self.pending.ui += milliseconds(elapsed);
    }

    pub fn record_playback(&mut self, elapsed: Duration) {
        self.pending.playback += milliseconds(elapsed);
    }

    pub fn record_assets(&mut self, elapsed: Duration) {
        let ms = milliseconds(elapsed);
        self.pending.assets += ms;
        self.startup_assets_ms = Some(ms);
    }

    fn clear_history(&mut self) {
        self.frames.clear();
        self.history_ms = 0.0;
        self.previous = None;
    }

    fn push(&mut self, frame: Frame) {
        self.history_ms += frame.ms;
        self.frames.push_back(frame);
        while self.frames.len() > MAX_SAMPLES
            || self.history_ms > HISTORY_MS && self.frames.len() > 1
        {
            if let Some(frame) = self.frames.pop_front() {
                self.history_ms -= frame.ms;
            }
        }
    }

    fn graph_peaks(&self) -> [f64; GRAPH_BARS] {
        let mut peaks = [0.0_f64; GRAPH_BARS];
        let mut age = 0.0;
        for frame in self.frames.iter().rev() {
            let bin = (age / HISTORY_MS * GRAPH_BARS as f64) as usize;
            if bin >= GRAPH_BARS {
                break;
            }
            let peak = &mut peaks[GRAPH_BARS - 1 - bin];
            *peak = peak.max(frame.ms);
            age += frame.ms;
        }
        peaks
    }

    fn recent_fps(&self) -> f64 {
        let mut duration = 0.0;
        let mut count = 0;
        for frame in self.frames.iter().rev() {
            duration += frame.ms;
            count += 1;
            if duration >= 1_000.0 {
                break;
            }
        }
        if duration > 0.0 {
            count as f64 * 1000.0 / duration
        } else {
            0.0
        }
    }

    fn csv(&self) -> String {
        let mut csv = String::from("kind,frame,metric,milliseconds\n");
        for (index, frame) in self.frames.iter().enumerate() {
            for (metric, value) in [
                ("frame", frame.ms),
                ("simulation", frame.simulation),
                ("trace", frame.trace),
                ("playback", frame.playback),
                ("ui", frame.ui),
                ("layout", frame.layout),
                ("assets", frame.assets),
                ("profiler", frame.profiler),
            ] {
                let _ = writeln!(csv, "frame,{index},{metric},{value:.6}");
            }
        }
        if let Some(ms) = self.startup_assets_ms {
            let _ = writeln!(csv, "startup,,atlas,{ms:.6}");
        }
        for (name, ms) in &self.passes {
            let _ = writeln!(csv, "render,,\"{}\",{ms:.6}", name.replace('"', "\"\""));
        }
        csv
    }
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[derive(Resource, Default)]
pub struct FramePacing {
    benchmark: bool,
    previous_tick: Option<Instant>,
}

impl FramePacing {
    pub fn new(benchmark: bool) -> Self {
        Self {
            benchmark,
            previous_tick: None,
        }
    }

    pub fn present_mode(&self) -> PresentMode {
        if self.benchmark {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::Fifo
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        if self.benchmark {
            "No VSync / 1000 FPS ceiling"
        } else {
            "VSync"
        }
    }
}

fn pace_frame(mut pacing: ResMut<FramePacing>) {
    if !pacing.benchmark {
        return;
    }
    if let Some(previous) = pacing.previous_tick {
        let remaining = MIN_FRAME_INTERVAL.saturating_sub(previous.elapsed());
        if !remaining.is_zero() {
            std::thread::sleep(remaining);
        }
    }
    pacing.previous_tick = Some(Instant::now());
}

#[derive(Message)]
pub struct ToggleFramePacing;

pub struct PerformancePlugin;

impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleFramePacing>()
            .init_resource::<FrameMetrics>()
            .init_resource::<Panel>()
            .init_resource::<FramePacing>()
            .add_systems(First, pace_frame.before(TimeSystems))
            .add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                RenderDiagnosticsPlugin,
            ))
            .add_systems(
                PostUpdate,
                (
                    begin_layout.before(UiSystems::Layout),
                    end_layout.after(UiSystems::Layout),
                ),
            )
            .add_systems(
                Last,
                (profiler_controls, refresh_panel, collect_frame).chain(),
            );
    }
}

fn begin_layout(mut metrics: ResMut<FrameMetrics>) {
    metrics.layout_start = Some(Instant::now());
}

fn end_layout(mut metrics: ResMut<FrameMetrics>) {
    if let Some(start) = metrics.layout_start.take() {
        metrics.pending.layout = milliseconds(start.elapsed());
    }
}

fn collect_frame(time: Res<Time<Real>>, mut metrics: ResMut<FrameMetrics>) {
    let pending = std::mem::take(&mut metrics.pending);
    let Some(mut frame) = metrics.previous.replace(pending) else {
        return;
    };
    frame.ms = milliseconds(time.delta());
    if frame.ms == 0.0 {
        return;
    }
    metrics.push(frame);
}

pub fn measurement() -> Instant {
    Instant::now()
}

#[cfg(test)]
mod tests;
