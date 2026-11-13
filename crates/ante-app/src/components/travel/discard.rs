use super::*;

/// Card discarded with the Discard button. It is detached from the hand so it outlives the
/// rebuilt hand, and is removed once it reaches the off-screen discard pile.
#[derive(Component)]
pub(crate) struct DiscardedCard {
    pub(super) age: f32,
}

/// A discarded card is removed after this many real seconds even if it is still moving.
pub(super) const DISCARDED_LIFETIME: f32 = 3.0;

/// After a Discard, detaches the cards that left the hand and sends them to the discard
/// pile one by one, 0.1 game seconds apart, like `G.FUNCS.discard_cards_from_highlighted`.
/// Runs after the session applied the click and before the hand is rebuilt.
#[allow(clippy::type_complexity)]
pub(crate) fn detach_discards(
    mut commands: Commands,
    mut clicks: MessageReader<Click>,
    session: Res<Session>,
    mut trail: ResMut<CardTrail>,
    mut previous: Local<Vec<(CardKey, bool)>>,
    mut cards: Query<(
        Entity,
        &mut Travel,
        &mut Node,
        &ComputedNode,
        &UiGlobalTransform,
        &UiTransform,
    )>,
) {
    let discarded = clicks
        .read()
        .any(|click| matches!(click, Click::Play(Command::Discard)));
    let hand: Vec<(CardKey, bool)> = session
        .confirmed
        .as_ref()
        .map(|view| {
            view.cards
                .iter()
                .map(|card| (card.id, card.selected))
                .collect()
        })
        .unwrap_or_default();
    if discarded {
        let leaving: Vec<CardKey> = previous
            .iter()
            .filter(|(key, selected)| *selected && !hand.iter().any(|(id, _)| id == key))
            .map(|(key, _)| *key)
            .collect();
        let mut ghosts: Vec<_> = cards
            .iter_mut()
            .filter(|(_, travel, ..)| travel.ready && leaving.contains(&travel.key))
            .collect();
        ghosts.sort_by_key(|(_, travel, ..)| travel.order);
        trail.discarding = ghosts.len();
        let speed = session.speed.factor();
        for (index, (entity, mut travel, mut node, computed, global, transform)) in
            ghosts.into_iter().enumerate()
        {
            pin_in_place(&mut node, computed, global, transform);
            travel.discard(index as f32 * DRAW_DELAY / speed);
            commands.entity(entity).remove::<ChildOf>().insert((
                DiscardedCard { age: 0.0 },
                GlobalZIndex(400),
                Pickable::IGNORE,
            ));
        }
    }
    if session.is_changed() {
        *previous = hand;
    }
}

/// Keeps a card where it is drawn once detached: its layout slot becomes an absolute
/// position at the root, and its own transform keeps applying on top.
pub(crate) fn pin_in_place(
    node: &mut Node,
    computed: &ComputedNode,
    global: &UiGlobalTransform,
    transform: &UiTransform,
) {
    let scale = computed.inverse_scale_factor();
    let size = computed.size() * scale;
    let shown = global.affine().translation * scale;
    let moved = Vec2::new(
        px_value(transform.translation.x),
        px_value(transform.translation.y),
    );
    let slot = shown - moved;
    node.position_type = PositionType::Absolute;
    node.left = px(slot.x - size.x / 2.0);
    node.top = px(slot.y - size.y / 2.0);
    node.margin = UiRect::ZERO;
}

/// Removes discarded cards once they reach the pile.
pub(crate) fn retire_discards(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut cards: Query<(Entity, &mut DiscardedCard, &Travel)>,
) {
    for (entity, mut card, travel) in &mut cards {
        card.age += time.delta_secs();
        if travel.discarded() || card.age > DISCARDED_LIFETIME {
            commands.entity(entity).despawn();
        }
    }
}
