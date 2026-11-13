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

#[derive(Component)]
struct GraphBar(usize);

#[derive(Component)]
enum PanelText {
    Summary,
    Scale,
    Details,
}

#[derive(Resource, Default)]
struct Panel {
    entities: Option<(Entity, Entity, Entity)>,
    refreshed: Option<Instant>,
    status: String,
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

fn profiler_controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
    store: Res<DiagnosticsStore>,
    mut panel: ResMut<Panel>,
    mut metrics: ResMut<FrameMetrics>,
    (mut pacing, mut toggle_pacing): (ResMut<FramePacing>, MessageReader<ToggleFramePacing>),
) {
    let requested_toggle = toggle_pacing.read().next().is_some();
    if requested_toggle {
        pacing.benchmark = !pacing.benchmark;
        pacing.previous_tick = None;
        for mut window in &mut windows {
            window.present_mode = pacing.present_mode();
        }
        metrics.clear_history();
        panel.refreshed = None;
        panel.status = pacing.label().to_owned();
        journal::event("frame_pacing_changed", json!({"mode": pacing.label()}));
    }
    let toggle = keys.just_pressed(KeyCode::F3);
    if keys.just_pressed(KeyCode::F2) {
        metrics.clear_history();
        panel.status = "Frame history reset".to_owned();
    }
    if let Some((window, camera, text)) = panel.entities {
        let closed = windows.get(window).is_err();
        if toggle || closed {
            if !closed {
                commands.entity(window).despawn();
            }
            commands.entity(camera).despawn();
            commands.entity(text).despawn();
            panel.entities = None;
        }
    } else if toggle {
        let window = commands
            .spawn(Window {
                title: "Ante | Frame profiler".into(),
                resolution: (850, 775).into(),
                present_mode: pacing.present_mode(),
                ..default()
            })
            .id();
        let camera = commands
            .spawn((Camera2d, RenderTarget::Window(WindowRef::Entity(window))))
            .id();
        let root = spawn_panel(&mut commands, camera);
        panel.entities = Some((window, camera, root));
        panel.refreshed = None;
    }
    if keys.just_pressed(KeyCode::F4) {
        metrics.passes = render_passes(&store);
        panel.status = match journal::export(&metrics.csv()) {
            Ok(path) => {
                journal::event("diagnostics_exported", json!({}));
                format!("Exported {}", path.display())
            }
            Err(error) => {
                journal::event("export_failed", json!({"error": error.to_string()}));
                format!("Export failed: {error}")
            }
        };
        eprintln!("{}", panel.status);
    }
}

fn spawn_panel(commands: &mut Commands, camera: Entity) -> Entity {
    commands
        .spawn((
            UiTargetCamera(camera),
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(16)),
                row_gap: px(5),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(crate::components::theme::FELT),
        ))
        .with_children(|panel| {
            panel_text(
                panel,
                PanelText::Summary,
                "Collecting frame measurements...",
                16.0,
            );
            panel_text(panel, PanelText::Scale, "", 13.0);
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(108),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.06, 0.10, 0.11)),
                ))
                .with_children(|plot| {
                    for grid in [0, 25, 50, 75, 100] {
                        plot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: percent(grid),
                                width: percent(100),
                                height: px(1),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.19, 0.27, 0.27)),
                        ));
                    }
                    for index in 0..GRAPH_BARS {
                        plot.spawn((
                            GraphBar(index),
                            Node {
                                position_type: PositionType::Absolute,
                                left: percent(index as f32 * 100.0 / GRAPH_BARS as f32),
                                bottom: px(0),
                                width: percent(80.0 / GRAPH_BARS as f32),
                                height: percent(0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.33, 0.80, 0.67)),
                        ));
                    }
                });
            panel
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|axis| {
                    axis.spawn((
                        Text::new("-10 s"),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                    ));
                    axis.spawn((
                        Text::new("Now"),
                        TextFont {
                            font_size: FontSize::Px(13.0),
                            ..default()
                        },
                    ));
                });
            panel_text(panel, PanelText::Details, "", 14.0);
        })
        .id()
}

fn panel_text(panel: &mut ChildSpawnerCommands, part: PanelText, text: &str, size: f32) {
    panel.spawn((
        part,
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
    ));
}

fn render_passes(store: &DiagnosticsStore) -> Vec<(String, f64)> {
    let mut passes: Vec<_> = store
        .iter()
        .filter_map(|diagnostic| {
            let path = diagnostic.path().as_str();
            (path.ends_with("elapsed_cpu") || path.ends_with("elapsed_gpu"))
                .then(|| Some((path.to_owned(), diagnostic.value()?)))
                .flatten()
        })
        .collect();
    passes.sort_by(|a, b| b.1.total_cmp(&a.1));
    passes
}

fn refresh_panel(
    mut panel: ResMut<Panel>,
    mut metrics: ResMut<FrameMetrics>,
    mut labels: Query<(&PanelText, &mut Text)>,
    mut bars: Query<(&GraphBar, &mut Node, &mut BackgroundColor)>,
    store: Res<DiagnosticsStore>,
    pacing: Res<FramePacing>,
) {
    if panel.entities.is_none() {
        return;
    };
    let now = Instant::now();
    if panel
        .refreshed
        .is_some_and(|previous| now.duration_since(previous) < REFRESH)
    {
        return;
    }
    panel.refreshed = Some(now);
    metrics.passes = render_passes(&store);
    let mut frames: Vec<_> = metrics.frames.iter().map(|frame| frame.ms).collect();
    let current = frames.last().copied().unwrap_or_default();
    frames.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        frames
            .get((frames.len() as f64 * p).ceil() as usize - usize::from(!frames.is_empty()))
            .copied()
            .unwrap_or_default()
    };
    let peak = frames.last().copied().unwrap_or_default();
    let scale = 4.0 * (peak / 4.0).max(1.0).log2().ceil().exp2();
    let peaks = metrics.graph_peaks();
    for (bar, mut node, mut color) in &mut bars {
        let ms = peaks[bar.0];
        node.height = percent((100.0 * ms / scale) as f32);
        color.0 = if ms > 16.67 {
            Color::srgb(0.97, 0.62, 0.30)
        } else {
            Color::srgb(0.33, 0.80, 0.67)
        };
    }
    let summary = format!(
        "Frame profiler | F2 reset | F3 close | F4 export\n{} | {:.0} FPS (last 1 s) | {:.2} ms latest | {} samples\nMedian {:.2} ms | P95 {:.2} ms | Peak {:.2} ms",
        pacing.label(),
        metrics.recent_fps(),
        current,
        frames.len(),
        percentile(0.5),
        percentile(0.95),
        peak,
    );
    let graph_scale = format!("Frame time in ms | 0 to {scale:.0} ms | peak per bin");
    let mut output = String::from("CPU stages (peak in the sample window)\n");
    let mut stages = [
        ("Simulation", 0.0_f64),
        ("Trace recording", 0.0),
        ("Score playback", 0.0),
        ("UI update", 0.0),
        ("UI layout", 0.0),
        ("Atlas loading", 0.0),
        ("Profiler update", 0.0),
    ];
    for frame in &metrics.frames {
        for (stage, value) in stages.iter_mut().zip([
            frame.simulation,
            frame.trace,
            frame.playback,
            frame.ui,
            frame.layout,
            frame.assets,
            frame.profiler,
        ]) {
            stage.1 = stage.1.max(value);
        }
    }
    stages.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (stage, ms) in stages {
        let _ = writeln!(output, "{stage}   {ms:.3} ms");
    }
    for (label, suffix) in [("Render CPU", "elapsed_cpu"), ("Render GPU", "elapsed_gpu")] {
        let _ = writeln!(output, "\n{label} (latest)");
        let mut passes = metrics
            .passes
            .iter()
            .filter(|(path, _)| path.ends_with(suffix))
            .take(3)
            .peekable();
        if passes.peek().is_none() {
            output.push_str("N/D\n");
        }
        for (path, ms) in passes {
            let _ = writeln!(
                output,
                "{}   {ms:.3} ms",
                path.trim_end_matches(suffix).trim_end_matches('/')
            );
        }
    }
    if let Some(ms) = metrics.startup_assets_ms {
        let _ = writeln!(output, "\nAtlas load at startup   {ms:.3} ms");
    }
    let _ = write!(output, "\n{}", panel.status);
    for (part, mut text) in &mut labels {
        text.0 = match part {
            PanelText::Summary => summary.clone(),
            PanelText::Scale => graph_scale.clone(),
            PanelText::Details => output.clone(),
        };
    }
    metrics.pending.profiler += milliseconds(now.elapsed());
}

pub fn measurement() -> Instant {
    Instant::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::diagnostic::DiagnosticsPlugin;

    #[test]
    fn time_buckets_keep_spikes_and_bound_history_at_high_fps() {
        let mut metrics = FrameMetrics::default();
        for _ in 0..10_100 {
            metrics.push(Frame {
                ms: 1.0,
                ..default()
            });
        }
        assert_eq!(metrics.frames.len(), 10_000);
        assert_eq!(metrics.recent_fps(), 1000.0);
        metrics.push(Frame {
            ms: 24.0,
            ..default()
        });
        assert_eq!(metrics.graph_peaks()[GRAPH_BARS - 1], 24.0);
        assert!(metrics.history_ms <= HISTORY_MS);
    }

    #[test]
    fn f3_toggles_one_window_with_its_own_camera() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DiagnosticsPlugin, PerformancePlugin))
            .init_resource::<ButtonInput<KeyCode>>();
        app.finish();
        app.cleanup();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F3);
        app.update();
        let (window, camera, panel) = app.world().resource::<Panel>().entities.unwrap();
        assert!(app.world().get::<Window>(window).is_some());
        assert_eq!(app.world().get::<UiTargetCamera>(panel).unwrap().0, camera);
        assert!(matches!(
            app.world().get::<RenderTarget>(camera),
            Some(RenderTarget::Window(WindowRef::Entity(id))) if *id == window
        ));
        let main = app.world_mut().spawn(Window::default()).id();
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear_just_pressed(KeyCode::F3);
        }
        app.world_mut().write_message(ToggleFramePacing);
        app.update();
        for id in [main, window] {
            assert_eq!(
                app.world().get::<Window>(id).unwrap().present_mode,
                PresentMode::AutoNoVsync
            );
        }
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::F3);
        keys.press(KeyCode::F3);
        app.update();
        assert!(app.world().resource::<Panel>().entities.is_none());
        assert!(app.world().get_entity(window).is_err());
        assert!(app.world().get_entity(camera).is_err());
    }
}
