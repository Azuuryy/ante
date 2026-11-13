use super::*;

#[derive(Component)]
pub(super) struct GraphBar(usize);

#[derive(Component)]
pub(super) enum PanelText {
    Summary,
    Scale,
    Details,
}

#[derive(Resource, Default)]
pub(super) struct Panel {
    pub(super) entities: Option<(Entity, Entity, Entity)>,
    pub(super) refreshed: Option<Instant>,
    pub(super) status: String,
}

pub(super) fn profiler_controls(
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

pub(super) fn spawn_panel(commands: &mut Commands, camera: Entity) -> Entity {
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

pub(super) fn panel_text(panel: &mut ChildSpawnerCommands, part: PanelText, text: &str, size: f32) {
    panel.spawn((
        part,
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
    ));
}

pub(super) fn render_passes(store: &DiagnosticsStore) -> Vec<(String, f64)> {
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

pub(super) fn refresh_panel(
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
