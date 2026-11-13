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
