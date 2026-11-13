use super::*;

/// Right after layout, starts rebuilt cards from their previous position and keeps every
/// card where it was drawn when the layout around it changes, so it glides instead of
/// jumping. Then records where every card is drawn. Runs before clipping and visibility
/// propagation, so the first frame of a rebuilt card is already drawn at its old position.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn place(
    time: Res<Time<Real>>,
    mut trail: ResMut<CardTrail>,
    windows: Query<&Window, With<PrimaryWindow>>,
    deck: Query<&UiGlobalTransform, (With<DeckPile>, Without<Travel>)>,
    mut cards: Query<(
        Entity,
        &mut Travel,
        &ComputedNode,
        &mut UiTransform,
        &mut UiGlobalTransform,
        Option<&mut HoverMotion>,
        Has<DiscardedCard>,
    )>,
    children: Query<&Children>,
    mut globals: Query<
        &mut UiGlobalTransform,
        (Without<Travel>, Without<DeckPile>, Without<Screen>),
    >,
    rooms: Query<
        (&UiGlobalTransform, &UiTransform, &ComputedNode),
        (With<Screen>, Without<Travel>),
    >,
) {
    let now = time.elapsed_secs();
    // Positions are kept in room space, so the room sway never reads as card movement.
    let room = room_frame(rooms.iter().next());
    let to_room = room.inverse();
    let deck = deck
        .iter()
        .next()
        .map(|deck| to_room.transform_point2(deck.affine().translation));
    let width = windows
        .single()
        .map(|window| window.resolution.physical_width() as f32)
        .unwrap_or(0.0);
    for (entity, mut travel, node, mut transform, mut global, motion, detached) in &mut cards {
        if !travel.planned || node.size().y == 0.0 {
            continue;
        }
        // A detached discarded card is no longer inside the room.
        let (frame, to_frame) = if detached {
            (Affine2::IDENTITY, Affine2::IDENTITY)
        } else {
            (room, to_room)
        };
        let scale = node.inverse_scale_factor();
        let drawn = Vec2::new(
            px_value(transform.translation.x),
            px_value(transform.translation.y),
        );
        let layout = to_frame.transform_point2(global.affine().translation) - drawn / scale;
        if travel.ready {
            // The layout moved under the card: keep it where it is and let it ease over.
            let moved = travel.layout.map_or(Vec2::ZERO, |last| layout - last);
            travel.layout = Some(layout);
            if moved.length() < 0.25 {
                continue;
            }
            let moved_logical = moved * scale;
            if travel.target == travel.discard {
                travel.target -= moved_logical;
            }
            travel.discard -= moved_logical;
            travel.offset -= moved_logical;
            let next = drawn - moved_logical;
            transform.translation = Val2::new(Val::Px(next.x), Val::Px(next.y));
            let shift = frame * Affine2::from_translation(-moved) * to_frame;
            apply(&mut global, shift);
            for child in children.iter_descendants(entity) {
                if let Ok(mut global) = globals.get_mut(child) {
                    apply(&mut global, shift);
                }
            }
            continue;
        }
        let (from, velocity, pose) = match travel.start {
            Start::Seen(position, velocity, pose) => (position, velocity, Some(pose)),
            Start::Deck => (deck.unwrap_or(layout), Vec2::ZERO, None),
            Start::Here => (layout, Vec2::ZERO, None),
        };
        travel.ready = true;
        travel.layout = Some(layout);
        travel.offset = (from - layout) * scale;
        travel.velocity = velocity;
        travel.discard = Vec2::new((width - layout.x) * scale + CARD_WIDTH, -DISCARD_RISE);
        let pose = match (pose, motion) {
            (Some(pose), Some(mut motion)) => {
                motion.bypass_change_detection().restore(pose);
                pose
            }
            _ => Pose {
                scale: 1.0,
                ..Pose::default()
            },
        };
        // Draw this first frame exactly as the card looked before the rebuild.
        let moved = travel.offset + Vec2::new(0.0, pose.drop - pose.lift);
        *transform = UiTransform {
            translation: Val2::new(Val::Px(moved.x), Val::Px(moved.y)),
            scale: Vec2::splat(pose.scale),
            rotation: Rot2::radians(pose.tilt),
        };
        let look = frame
            * Affine2::from_translation(layout + moved / scale)
            * Affine2::from_scale_angle_translation(Vec2::splat(pose.scale), pose.tilt, Vec2::ZERO)
            * Affine2::from_translation(-layout)
            * to_frame;
        apply(&mut global, look);
        for child in children.iter_descendants(entity) {
            if let Ok(mut global) = globals.get_mut(child) {
                apply(&mut global, look);
            }
        }
    }
    for (_, travel, node, _, global, motion, detached) in &cards {
        if travel.ready {
            let to_frame = if detached { Affine2::IDENTITY } else { to_room };
            // Where the card is drawn, without its fan drop and lift, which the pose restores.
            // A dragged card keeps the drag translation, so a drop starts where it was let go.
            let pose = motion.map(|motion| motion.pose()).unwrap_or_default();
            let look = Vec2::new(0.0, pose.drop - pose.lift) / node.inverse_scale_factor();
            trail.seen.insert(
                travel.key,
                Seen {
                    position: to_frame.transform_point2(global.affine().translation) - look,
                    velocity: travel.velocity,
                    pose,
                    played: travel.played,
                    at: now,
                },
            );
        }
    }
}

/// Hides a drawn card's back once its flip passes the middle.
pub(crate) fn turn_card_backs(
    cards: Query<&Travel>,
    mut backs: Query<(&ChildOf, &mut Visibility), With<CardBack>>,
) {
    for (parent, mut visibility) in &mut backs {
        let shown = cards
            .get(parent.parent())
            .is_ok_and(|travel| travel.back_shown());
        let next = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}

pub(super) fn px_value(value: Val) -> f32 {
    match value {
        Val::Px(value) => value,
        _ => 0.0,
    }
}

pub(super) fn apply(global: &mut UiGlobalTransform, transform: Affine2) {
    *global = (transform * global.affine()).into();
}
