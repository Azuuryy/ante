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

#[allow(clippy::too_many_arguments)]
pub(crate) fn animate(
    mut commands: Commands,
    mut session: ResMut<Session>,
    time: Res<Time<Real>>,
    mut text: Query<&mut Text, (With<PlaybackLabel>, Without<run_ui::ScoreTile>)>,
    mut tiles: Query<(&run_ui::ScoreTile, &mut Text, &mut UiTransform), Without<PlaybackLabel>>,
    mut cards: Query<AnimatedCard>,
    (popups, mut jokers): (
        Query<Entity, With<ScorePopup>>,
        Query<AnimatedJoker, Without<ScoredCard>>,
    ),
    (mut metrics, mut shown_popup): (ResMut<FrameMetrics>, Local<Option<(usize, usize)>>),
    (mut sounds, mut room, mut shown_stage): (
        MessageWriter<Sfx>,
        ResMut<RoomShake>,
        Local<Option<PlaybackStage>>,
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
            // `card_eval_status_text` shakes the room by 0.7 per popup.
            room.kick(0.7);
        }
        let stage = playback.stage();
        if *shown_stage != Some(stage) {
            *shown_stage = Some(stage);
            if stage == PlaybackStage::Total {
                sounds.write(Sfx::new("chips2", 1.0, 1.0));
            }
        }
        let popup = current.filter(|_| new_popup).and_then(|(index, _, popup)| {
            playback
                .current_step()
                .filter(|(step, _)| *step == index)
                .map(|(_, step)| (step.source, popup))
        });
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
                }
            }
            if let Some((ScoreSource::Card { id, .. }, popup)) = &popup
                && *id == card.id
            {
                motion.juice_up(0.6, 0.1, rand::random());
                let (label, color) = popup_text(*popup);
                commands
                    .entity(entity)
                    .with_child(score_popup(label, color, PopupSide::Above));
            }
        }
        if let Some((ScoreSource::Joker { id, .. }, popup)) = &popup {
            for (entity, joker, motion) in &mut jokers {
                if joker.id != *id {
                    continue;
                }
                if let Some(mut motion) = motion {
                    motion.juice_up(0.6, 0.1, rand::random());
                }
                let (label, color) = popup_text(*popup);
                commands
                    .entity(entity)
                    .with_child(score_popup(label, color, PopupSide::Below));
            }
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

/// Sound of a popup, as in `card_eval_status_text`: pitch 0.8 + 0.2 × (0.9 to 1.1).
pub(super) fn popup_sound(popup: Popup) -> Sfx {
    let pitch = 0.8 + 0.2 * (0.9 + 0.2 * rand::random::<f32>());
    match popup {
        Popup::Again => Sfx::new("generic1", pitch, 1.0),
        Popup::Chips(_) => Sfx::new("chips1", pitch, 1.0),
        Popup::Mult(_) => Sfx::new("multhit1", pitch, 1.0),
        Popup::XMult(_) => Sfx::new("multhit2", pitch, 0.7),
        Popup::Money(_) => Sfx::new("coin3", pitch, 1.0),
    }
}

/// Popup label and color, as in `card_eval_status_text`: chips blue, mult and xmult red,
/// money yellow (red when lost), "Again!" in the filter orange.
pub(super) fn popup_text(popup: Popup) -> (String, Color) {
    match popup {
        Popup::Again => ("Again!".into(), POPUP_AGAIN),
        Popup::Chips(amount) => (format!("{amount:+}"), POPUP_CHIPS),
        Popup::Mult(amount) => (format!("{amount:+} Mult"), POPUP_MULT),
        Popup::XMult(hundredths) => (format!("X{} Mult", multiplier(hundredths)), POPUP_MULT),
        Popup::Money(amount) if amount < 0 => (format!("-${}", -amount), POPUP_MULT),
        Popup::Money(amount) => (format!("${amount}"), POPUP_MONEY),
    }
}

/// A factor in hundredths written like the original: X2, X1.5, X1.25.
pub(super) fn multiplier(hundredths: usize) -> String {
    let text = format!("{}.{:02}", hundredths / 100, hundredths % 100);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub(super) const POPUP_AGAIN: Color = Color::srgb(1.0, 0.604, 0.0);

pub(super) const POPUP_MONEY: Color = Color::srgb(0.953, 0.725, 0.357);

pub(super) const POPUP_CHIPS: Color = Color::srgb(0.0, 0.616, 1.0);

pub(super) const POPUP_MULT: Color = Color::srgb(0.996, 0.373, 0.333);

/// Original status text sits above played cards and below Jokers and consumables.
#[derive(Clone, Copy)]
pub(super) enum PopupSide {
    Above,
    Below,
}

pub(super) fn score_popup(label: String, color: Color, side: PopupSide) -> impl Bundle {
    let (top, bottom) = match side {
        PopupSide::Above => (px(-34), Val::Auto),
        PopupSide::Below => (Val::Auto, px(-34)),
    };
    (
        ScorePopup,
        Node {
            position_type: PositionType::Absolute,
            top,
            bottom,
            padding: UiRect::axes(px(8), px(2)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(color),
        GlobalZIndex(500),
        Pickable::IGNORE,
        children![(
            Text::new(label),
            TextFont {
                font_size: bevy::text::FontSize::Px(22.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        )],
    )
}
