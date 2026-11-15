use super::*;

/// Drawn debuff cross and tint, hidden once the original debuff shader is loaded.
#[derive(Component)]
pub(crate) struct DebuffFallback;

/// Hides the drawn debuff look when the local debuff shader replaces it.
pub(crate) fn hide_debuff_fallback(
    shaders: Option<Res<crate::services::assets::shaders::LoveShaders>>,
    mut fallbacks: Query<&mut Node, Added<DebuffFallback>>,
) {
    if shaders.is_some_and(|shaders| shaders.get("debuff").is_some()) {
        for mut node in &mut fallbacks {
            node.display = Display::None;
        }
    }
}

/// A card shown in the play area while its hand is scored.
#[derive(Component)]
pub(crate) struct PlayedCard;

pub(crate) fn card_control(
    ui: &mut ChildSpawnerCommands,
    action: Option<Click>,
    card: &CardView,
    tip: tooltip::Tip,
    (count, position): (usize, usize),
    (reorderable, played): (bool, bool),
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    // The row stays centred; `row_gap` follows the hand or play area of `align_cards`.
    let spacing = row_gap(count, played);
    let lifted = card.selected && !played;
    let mut entity = ui.spawn((
        Node {
            width: px(CARD_WIDTH),
            height: px(CARD_HEIGHT),
            flex_shrink: 0.0,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            margin: UiRect {
                left: px(if position > 0 { spacing } else { 0.0 }),
                top: px(if lifted { -HIGHLIGHT_LIFT } else { 0.0 }),
                ..default()
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BorderColor::all(Color::NONE),
        UiTransform::IDENTITY,
        ScoredCard { id: card.id },
    ));
    tooltip::attach(
        &mut entity,
        tip.technical(format!("Card #{} · {}{}", card.id.0, card.rank, card.suit))
            .technical(format!(
                "Selected {} · enabled {} · debuffed {} · base chips {}",
                card.selected, card.enabled, card.debuffed, card.chips
            ))
            .technical(format!(
                "Enhancement {} · edition {} · seal {}",
                card.enhancement, card.edition, card.seal
            )),
    );
    card_hover(&mut entity);
    entity.insert(Travel::new(card.id, played, position));
    if played {
        entity.insert(PlayedCard);
    }
    if reorderable {
        card_drag::attach(
            &mut entity,
            card_drag::Lane::Hand,
            position,
            action,
            Some(card.id),
            card.selected,
            // Later cards draw over earlier ones, like `card.rank` in the original; distinct
            // layers keep the order stable when a drag rewrites one of them.
            position as i32,
        );
    }
    entity.with_children(content);
}

pub(crate) fn fallback_art(ui: &mut ChildSpawnerCommands, name: &str, width: f32) {
    ui.spawn((
        Node {
            width: px(width),
            height: px(width * 95.0 / 71.0),
            padding: UiRect::all(px(6)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(DISABLED),
        Pickable::IGNORE,
    ))
    .with_children(|art| label(art, name, 15.0, INK));
}

/// A playing card as `Card:draw` layers it: the enhancement centre (or the plain base), the
/// rank and suit face unless it is a Stone card, the edition shader and the seal.
pub(crate) struct PlayingFace<'a> {
    pub(crate) rank: char,
    pub(crate) suit: &'a str,
    pub(crate) enhancement: &'a str,
    pub(crate) edition: &'a str,
    pub(crate) seal: &'a str,
    pub(crate) debuffed: bool,
    /// Drawn with the `played` shader, like a card the deck view greys out.
    pub(crate) greyed: bool,
    pub(crate) seed: usize,
}

/// Draws `card`; false when the local card art is missing, so the caller can fall back.
pub(crate) fn playing_card_face(
    ui: &mut ChildSpawnerCommands,
    atlases: &crate::services::assets::catalog::Atlases,
    card: &PlayingFace,
    width: f32,
) -> bool {
    let Some(face) = atlases.card(card.rank, card.suit) else {
        return false;
    };
    let stone = card.enhancement == "Stone";
    card_face_with_seal(
        ui,
        atlases.card_base_for(card.enhancement),
        (!stone).then_some(face),
        (card.debuffed, card.greyed),
        width,
        Some((card.seal, atlases.seal(card.seal))),
        (card.edition, card.seed),
    );
    true
}

pub(crate) fn card_face_with_seal(
    ui: &mut ChildSpawnerCommands,
    base: Option<ImageNode>,
    face: Option<ImageNode>,
    (debuffed, greyed): (bool, bool),
    width: f32,
    seal: Option<(&str, Option<ImageNode>)>,
    (edition, seed): (&str, usize),
) {
    ui.spawn(Node {
        width: px(width),
        height: px(width * 95.0 / 71.0),
        ..default()
    })
    .with_children(|layers| {
        let seal_sprite = seal.as_ref().and_then(|(_, sprite)| sprite.clone());
        // Like `Card:draw`, the edition shader covers the base and the face, under the seal.
        let sprites: Vec<ImageNode> = base.into_iter().chain(face).collect();
        for image in &sprites {
            spawn_layer(layers, image.clone());
        }
        for image in &sprites {
            crate::components::edition::overlay(layers, edition, image, seed);
        }
        if let Some(image) = seal_sprite {
            spawn_layer(layers, image.clone());
            // `Card:draw` shines the Gold seal with the voucher shader.
            if seal.as_ref().is_some_and(|(name, _)| *name == "Gold") {
                crate::components::edition::overlay(layers, "Voucher", &image, seed);
            }
        }
        fn spawn_layer(layers: &mut ChildSpawnerCommands, image: ImageNode) {
            layers.spawn((
                image,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        }
        if let Some((name, None)) = seal {
            let color = match name {
                "Gold" => Some(Color::srgb(0.94, 0.65, 0.18)),
                "Red" => Some(Color::srgb(0.83, 0.16, 0.17)),
                "Blue" => Some(Color::srgb(0.13, 0.45, 0.88)),
                "Purple" => Some(Color::srgb(0.54, 0.22, 0.72)),
                _ => None,
            };
            if let Some(color) = color {
                layers.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(19),
                        top: percent(14),
                        width: px(width * 0.40),
                        height: px(width * 0.40),
                        border: UiRect::all(px(width * 0.025)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.52)),
                    Pickable::IGNORE,
                ));
            }
        }
        if greyed {
            // `Card:draw` covers a greyed card with the `played` shader; a dark tint stands
            // in without local shaders.
            for image in &sprites {
                crate::components::edition::overlay(layers, "Played", image, seed);
            }
            layers.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                Pickable::IGNORE,
                DebuffFallback,
            ));
        }
        if debuffed {
            // `Card:draw` covers a debuffed card's base and face with the debuff shader;
            // the drawn cross and tint stay as the fallback without local shaders.
            for image in &sprites {
                crate::components::edition::overlay(layers, "Debuff", image, seed);
            }
            layers.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.88, 0.12, 0.10, 0.20)),
                Pickable::IGNORE,
                DebuffFallback,
            ));
            for rotation in [-0.78, 0.78] {
                layers.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(7),
                        top: percent(43),
                        width: percent(120),
                        height: percent(14),
                        ..default()
                    },
                    UiTransform {
                        rotation: Rot2::radians(rotation),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.88, 0.06, 0.08, 0.84)),
                    Pickable::IGNORE,
                    DebuffFallback,
                ));
            }
        }
    });
}
