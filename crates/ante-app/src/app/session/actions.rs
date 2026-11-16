use super::*;

/// Starts a new run, or restarts the last configuration.
pub(super) fn start_run(session: &mut Session, backend: &dyn GameBackend, click: Click) {
    let config = if matches!(click, Click::Restart) {
        session.last_config
    } else {
        session
            .navigation
            .deck
            .and_then(DeckChoice::deck)
            .filter(|_| session.stake == 0)
            .and_then(|deck| {
                let seed = if session.seeded_run {
                    parse_seed(backend, &session.seed)?
                } else {
                    rand::random()
                };
                Some(RunConfig {
                    seed,
                    deck,
                    goal: session.goal,
                    starting_money: STARTING_MONEY_OVERRIDE,
                })
            })
    };
    if let Some(config) = config {
        let (run, view) = match guard_simulation("Could not start the run", || {
            let run = backend.start(config);
            let view = run.view();
            Ok((run, view))
        }) {
            Ok(started) => started,
            Err(error) => {
                session.faulted = true;
                session.error = Some(error.to_string());
                return;
            }
        };
        session.playback = None;
        session.editing_seed = false;
        session.inventory_selection = None;
        session.shop_selection = None;
        session.deck_hovered = false;
        session.confirmed = Some(view);
        session.run = Some(run);
        session.run_log = Some(crate::services::run_log::RunLogFile::start(config));
        session.notes.clear();
        session.run_id = session.run_id.wrapping_add(1);
        session.last_config = Some(config);
        session.navigation.screen = Page::Run;
        journal_event!(
            "run_started",
            json!({"seed": config.seed, "deck": format!("{:?}", config.deck)}),
        );
    } else {
        session.error =
            Some("Choose an available deck, White Stake and a valid seed if enabled.".into());
    }
}

/// Applies a game command and starts the score playback of its trace.
pub(super) fn play(
    session: &mut Session,
    metrics: &mut FrameMetrics,
    profile: &mut Profile,
    action: Command,
) {
    session.inventory_selection = None;
    session.shop_selection = None;
    session.deck_hovered = false;
    #[cfg(feature = "dev-tools")]
    let action_id = {
        session.action_id += 1;
        session.action_id
    };
    let Some(before) = session.confirmed.take() else {
        return;
    };
    let Some(run) = session.run.as_mut() else {
        session.faulted = true;
        session.error = Some("The run is unavailable. Return to the seed menu.".into());
        return;
    };
    journal_event!(
        "action_started",
        json!({
            "id": action_id, "seed": before.seed, "phase": format!("{:?}", before.phase),
            "action": format!("{action:?}"),
        }),
    );
    let start = diagnostics::measurement();
    let result = guard_simulation("Could not apply the game action", || {
        run.apply(
            action,
            if cfg!(feature = "dev-tools") {
                TraceMode::Profile
            } else {
                TraceMode::Capture
            },
        )
    });
    let elapsed = start.elapsed();
    if let Some(log) = session.run_log.as_mut() {
        match &result {
            Ok(update) => log.record(&before, action, Ok(update), &update.view),
            Err(error) => log.record(&before, action, Err(error), &before),
        }
    }
    match result {
        Ok(update) => {
            session.confirmed = Some(update.view);
            session.notes.extend(update.notes);
            let outcome = update.outcome;
            metrics.record_simulation(elapsed, outcome.trace_time);
            #[cfg(feature = "dev-tools")]
            if outcome.campaign_won && !before.campaign_won {
                journal_event!(
                    "run_victory",
                    json!({"seed": before.seed, "ante": 8, "score": outcome.round_score}),
                );
            }
            journal_event!(
                "action_accepted",
                json!({
                    "id": action_id, "seed": before.seed, "action": format!("{action:?}"),
                    "before": format!("{:?}", before.phase), "after": format!("{:?}", outcome.phase),
                    "score": outcome.round_score, "last_hand": outcome.hand_score,
                    "trace_steps": outcome.trace.as_ref().map_or(0, |trace| trace.0.len()),
                    "elapsed_us": elapsed.as_micros(), "trace_us": outcome.trace_time.as_micros(),
                }),
            );
            if let Some(id) = update.events.iter().find_map(|event| match event {
                ante_core::contract::RunEvent::Discovered(id) => Some(id.0),
                _ => None,
            }) && let Err(error) = profile.discover(id)
            {
                journal_event!("profile_write_failed", json!({"error": error.to_string()}),);
                session.error = Some(format!("Collection could not be saved: {error}"));
            }
            if let Some(trace) = outcome.trace {
                let played = before.cards.iter().filter(|card| card.selected).count();
                session.playback = Some((before, Playback::new(trace, played)));
            }
        }
        Err(error) => {
            metrics.record_simulation(elapsed, default());
            session.faulted = matches!(error, RunError::Simulation(_));
            journal_event!(
                if session.faulted {
                    "simulation_error"
                } else {
                    "action_refused"
                },
                json!({
                    "id": action_id, "seed": before.seed, "phase": format!("{:?}", before.phase),
                    "action": format!("{action:?}"), "error": error.to_string(),
                }),
            );
            session.confirmed = Some(before);
            session.error = Some(error.to_string());
        }
    }
}
