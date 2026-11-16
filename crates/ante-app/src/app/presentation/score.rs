use super::*;

#[derive(Component)]
pub(crate) struct ScoredCard {
    pub(crate) id: CardKey,
}

/// Held Joker, so its scoring step can juice it and show its popup below it.
#[derive(Component)]
pub(crate) struct ScoredJoker {
    pub(crate) id: JokerKey,
}

/// Colored popup over a card while its scoring step shows, like the original status text.
#[derive(Component)]
pub(crate) struct ScorePopup;

pub(super) type AnimatedCard = (
    Entity,
    &'static ScoredCard,
    &'static mut BorderColor,
    Option<&'static mut HoverMotion>,
    Has<PlayedCard>,
    Option<&'static mut Travel>,
);

pub(super) type AnimatedJoker = (
    Entity,
    &'static ScoredJoker,
    Option<&'static mut HoverMotion>,
);

/// Existing popups, the held Jokers and the parents of played cards (the play area).
type PopupTargets<'w, 's> = (
    Query<'w, 's, Entity, With<ScorePopup>>,
    Query<'w, 's, AnimatedJoker, Without<ScoredCard>>,
    Query<'w, 's, &'static ChildOf>,
);

#[allow(clippy::too_many_arguments)]
pub(crate) fn animate(
    mut commands: Commands,
    mut session: ResMut<Session>,
    time: Res<Time<Real>>,
    mut text: Query<&mut Text, (With<PlaybackLabel>, Without<run_ui::ScoreTile>)>,
    mut tiles: Query<(&run_ui::ScoreTile, &mut Text, &mut UiTransform), Without<PlaybackLabel>>,
    mut cards: Query<AnimatedCard>,
    (popups, mut jokers, parents): PopupTargets,
    (mut metrics, mut shown_popup): (ResMut<FrameMetrics>, Local<Option<(usize, usize)>>),
    (mut sounds, mut room, mut shown_stage, mut scheduled): (
        MessageWriter<Sfx>,
        ResMut<RoomShake>,
        Local<Option<PlaybackStage>>,
        ResMut<ScheduledSfx>,
    ),
) {
    if session.navigation.screen != Page::Run {
        return;
    }
    let speed = session.speed;
    let Some((_, playback)) = &mut session.bypass_change_detection().playback else {
        *shown_popup = None;
        return;
    };
    let start = diagnostics::measurement();
    if playback.advance(time.delta(), speed) {
        session.playback = None;
        *shown_popup = None;
    } else {
        let lifted = matches!(playback.stage(), PlaybackStage::Scoring);
        let discarding = matches!(playback.stage(), PlaybackStage::Discard);
        let scoring = playback.scoring_cards();
        // Each popup of a step shows in turn and juices its source, like `card_eval_status_text`.
        let current = playback.current_popup();
        let key = current.map(|(index, position, _)| (index, position));
        let new_popup = key != *shown_popup;
        if new_popup {
            *shown_popup = key;
            for popup in &popups {
                commands.entity(popup).despawn();
            }
        }
        if let Some((_, _, popup)) = current.filter(|_| new_popup) {
            sounds.write(popup_sound(popup));
            if popup == Popup::NotAllowed {
                // The boss juices with `tarot2` as the refused hand is evaluated, and
                // `play_area_status_text` shakes the room by 2.
                sounds.write(Sfx::new("tarot2", 1.0, 0.4));
                room.kick(2.0);
            } else {
                // `card_eval_status_text` shakes the room by 0.7 per popup.
                room.kick(0.7);
            }
        }
        let stage = playback.stage();
        if *shown_stage != Some(stage) {
            *shown_stage = Some(stage);
            if stage == PlaybackStage::Total {
                // `evaluate_play`: the total shows with the `button` sound 0.4 s after the
                // last effect, then `ease_chips` adds it with `chips2` 0.8 s later.
                use crate::app::playback::timing::{TOTAL_HOLD, TOTAL_SHOWN};
                let now = time.elapsed_secs();
                let rate = speed.factor();
                scheduled.after(now, TOTAL_SHOWN / rate, Sfx::new("button", 0.9, 0.6));
                scheduled.after(
                    now,
                    (TOTAL_SHOWN + TOTAL_HOLD) / rate,
                    Sfx::new("chips2", 1.0, 1.0),
                );
            }
        }
        let popup = current.filter(|_| new_popup).and_then(|(index, _, popup)| {
            playback
                .current_step()
                .filter(|(step, _)| *step == index)
                .map(|(_, step)| (step.source, popup))
        });
        let mut moved = 0;
        for (entity, card, _, motion, played, travel) in &mut cards {
            if discarding
                && played
                && let Some(mut travel) = travel
            {
                // Played cards go to the discard pile one by one, 0.1 game seconds apart.
                let delay = travel.order() as f32 * DRAW_DELAY / speed.factor();
                travel.discard(delay);
            }
            let Some(mut motion) = motion else { continue };
            if played {
                let raise = if lifted && scoring.contains(&card.id) {
                    HIGHLIGHT_LIFT
                } else {
                    0.0
                };
                if motion.raise != raise {
                    motion.raise = raise;
                    moved += 1;
                }
            }
            if let Some((
                ScoreSource::Card { id, .. } | ScoreSource::JokerOnCard { card: id, .. },
                popup,
            )) = &popup
                && *id == card.id
            {
                motion.juice_up(0.6, 0.1, rand::random());
                commands
                    .entity(entity)
                    .with_child(score_popup(*popup, PopupSide::Above));
            }
        }
        // `highlight_card` plays cardSlide1 for each scoring card that rises or falls, 0.1
        // game seconds apart with a pitch climbing by 0.2 over the hand; at once, the five
        // sounds saturated.
        for (index, pitch) in highlight_pitches(moved).into_iter().enumerate() {
            let delay = index as f32 * crate::app::playback::timing::HIGHLIGHT_PER_CARD;
            scheduled.after(
                time.elapsed_secs(),
                delay / speed.factor(),
                Sfx::new("cardSlide1", pitch, 1.0),
            );
        }
        // A per-card Joker effect juices the Joker; its popup is on the card.
        if let Some((ScoreSource::JokerOnCard { joker: id, .. }, _)) = &popup {
            for (_, joker, motion) in &mut jokers {
                if joker.id == *id
                    && let Some(mut motion) = motion
                {
                    motion.juice_up(0.6, 0.1, rand::random());
                }
            }
        }
        if let Some((ScoreSource::Joker { id, .. } | ScoreSource::JokerMessage { id, .. }, popup)) =
            &popup
        {
            for (entity, joker, motion) in &mut jokers {
                if joker.id != *id {
                    continue;
                }
                if let Some(mut motion) = motion {
                    motion.juice_up(0.6, 0.1, rand::random());
                }
                commands
                    .entity(entity)
                    .with_child(score_popup(*popup, PopupSide::Below));
            }
        }
        // `play_area_status_text`: "Not Allowed!" over the middle of the play area.
        if let Some((ScoreSource::NotAllowed, _)) = &popup
            && let Some(area) = cards
                .iter()
                .find(|(.., played, _)| *played)
                .and_then(|(entity, ..)| parents.get(entity).ok())
        {
            commands.entity(area.parent()).with_child(play_area_popup());
        }
        for mut text in &mut text {
            let next = playback.description();
            if next != text.0 {
                text.0 = next;
            }
        }
        if let Some((chips, mult)) = playback.score_values() {
            let pop = 1.0 + 0.12 * (1.0 - playback.step_progress());
            for (tile, mut text, mut transform) in &mut tiles {
                let next = if tile.0 {
                    run_ui::score_text(chips)
                } else {
                    run_ui::mult_text(mult)
                };
                if text.0 != next {
                    text.0 = next;
                }
                if transform.scale != Vec2::splat(pop) {
                    transform.scale = Vec2::splat(pop);
                }
            }
        }
        let focused = playback.focused_card();
        for (_, card, mut background, ..) in &mut cards {
            let next = BorderColor::all(if focused == Some(card.id) {
                INK
            } else {
                Color::NONE
            });
            if *background != next {
                *background = next;
            }
        }
    }
    metrics.record_playback(start.elapsed());
}

/// Pitch of each `highlight_card` sound for `count` cards: `0.85 + percent × 0.2`, with
/// `percent` from 0 to 1 over the hand like the original's `(i - 0.999) / 5`.
pub(super) fn highlight_pitches(count: usize) -> Vec<f32> {
    (0..count)
        .map(|index| 0.85 + 0.2 * index as f32 / count.max(5) as f32)
        .collect()
}
