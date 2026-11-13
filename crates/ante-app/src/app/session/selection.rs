use super::*;

pub(super) fn guard_simulation<T>(
    context: &str,
    operation: impl FnOnce() -> Result<T, RunError>,
) -> Result<T, RunError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|panic| {
        let detail = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or("unexpected simulation failure");
        Err(RunError::Simulation(format!("{context}. {detail}")))
    })
}

pub(super) fn apply_card_selection(
    session: &mut Session,
    metrics: &mut FrameMetrics,
    id: ante_core::contract::CardKey,
) {
    let Some(before) = session.confirmed.take() else {
        return;
    };
    let Some(run) = session.run.as_mut() else {
        session.confirmed = Some(before);
        return;
    };
    let start = diagnostics::measurement();
    let result = guard_simulation("Could not change the card selection", || {
        run.apply(Command::ToggleCard(id), TraceMode::Off)
    });
    metrics.record_simulation(start.elapsed(), default());
    match result {
        Ok(update) => session.confirmed = Some(update.view),
        Err(error) => {
            session.faulted = matches!(error, RunError::Simulation(_));
            session.confirmed = Some(before);
            session.error = Some(error.to_string());
        }
    }
}

pub(crate) fn right_click_clear(
    mouse: Res<ButtonInput<MouseButton>>,
    session: Res<Session>,
    mut clicks: MessageWriter<Click>,
) {
    if mouse.just_pressed(MouseButton::Right)
        && session.navigation.screen == Page::Run
        && session.playback.is_none()
        && session
            .confirmed
            .as_ref()
            .is_some_and(|view| view.phase == Phase::Playing)
    {
        clicks.write(Click::ClearSelection);
    }
}
