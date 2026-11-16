use super::*;

pub struct RunBoard {
    pub root: Entity,
    pub overlay: Entity,
    pub run_id: u64,
    pub(super) center: Entity,
    pub(super) inventory: [Entity; 2],
    pub(super) counts: [Entity; 2],
    pub(super) tags: Entity,
    pub(super) deck_count: Entity,
    /// The deck area holding the stacked backs.
    pub(super) deck_pile: Entity,
    /// The "View Deck" label, centred on the top card of the pile.
    pub(super) deck_label: Entity,
    pub(super) deck_back: Option<ImageNode>,
    /// Cards in the deck when its backs were last drawn.
    pub(super) deck_shown: Option<usize>,
    /// "/ 8" after the ante number.
    pub(super) ante_end: Entity,
    /// `create_UIBox_HUD_blind`, shown while a blind is played.
    pub(super) blind_box: Entity,
    /// Blind name, rule, target and reward.
    pub(super) blind_texts: [Entity; 4],
    pub(super) hud: [Entity; 8],
    pub(super) blind_chip: Entity,
    pub(super) shop_sign: Entity,
    pub(super) score_tiles: [Entity; 2],
    pub(super) hand_title: Entity,
    pub(super) error: Entity,
    pub(super) frame: Option<BoardFrame>,
    pub(super) tag_keys: Vec<&'static str>,
    /// The hand and its actions while a hand is steady, updated in place on selection.
    pub(super) playing: Option<crate::screens::hand::HandSlots>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct BoardFrame {
    pub(super) revision: u64,
    pub(super) trace: bool,
    pub(super) speed: Speed,
    pub(super) preview: bool,
}

/// The HUD sits 0.7 tiles left of the room (`align = 'cli'`, offset −0.7) over the full
/// canvas height; its content is about 5.1 tiles wide.
pub(super) const HUD_LEFT: f32 = -0.7 * TILE;

pub(super) const HUD_WIDTH: f32 = 5.1 * TILE;

/// How far the HUD body runs past the room, in tiles: `(30 - ROOM.y) / 2`.
pub(super) const HUD_OVERHANG: f32 = (30.0 - 11.5) / 2.0;

/// Card areas from `set_screen_positions`: the hand starts at 20 − 6·CARD_W − 2.85 tiles and
/// the Jokers 0.1 tile further left, at the top of the room.
pub(super) const HAND_X: f32 = 20.0 - 6.0 * CARD_W - 2.85;

pub(super) const CENTER_LEFT: f32 = (HAND_X - 0.1) * TILE;

pub(super) const AREA_HEIGHT: f32 = 0.95 * CARD_H * TILE;

/// Deck area: 1.1 card wide, 0.5 tile from the room's right edge, level with the hand.
pub(super) const DECK_LEFT: f32 = (20.0 - 1.1 * CARD_W - 0.5) * TILE;

pub(super) const DECK_TOP: f32 = (11.5 - 0.95 * CARD_H) * TILE;

pub(super) const DECK_AREA: Vec2 = Vec2::new(1.1 * CARD_W * TILE, 0.95 * CARD_H * TILE);

/// Scale of the stack spread; 1 is the original `deck_height`. Lower it to tighten the pile.
pub(super) const DECK_STACK_SCALE: f32 = 0.6;

/// Brightness of the backs under the top card.
pub(super) const DECK_UNDER_SHADE: f32 = 0.78;

/// `shadow_parrallax.x` of the deck area: (centre − 10) / 10 x 1.5.
pub(super) const DECK_PARALLAX_X: f32 = ((20.0 - 0.55 * CARD_W - 0.5) - 10.0) / 10.0 * 1.5;

/// Offset in tiles of card `k` (1 on top) in a deck of `n`, from `CardArea:align_cards`:
/// `shadow_parrallax` x 0.15 / 52 per card below it, plus the card's own parallax / 30.
pub(crate) fn deck_offset(n: usize, k: usize) -> Vec2 {
    let below = n.saturating_sub(k) as f32 * 0.15 / 52.0 * DECK_STACK_SCALE;
    Vec2::new(
        DECK_PARALLAX_X * below + DECK_PARALLAX_X / 30.0,
        -1.5 * below,
    )
}

/// The original only draws card 1, every ninth card and the last (`thin_draw`).
pub(super) fn drawn_backs(n: usize) -> impl Iterator<Item = usize> {
    (1..=n)
        .rev()
        .filter(move |&k| k == 1 || k % 9 == 0 || k == n)
}

#[derive(Component)]
pub struct ScoreTile(pub bool);

impl RunBoard {
    pub fn spawn(
        commands: &mut Commands,
        atlases: &Atlases,
        deck: DeckChoice,
        run_id: u64,
        stake: usize,
    ) -> Self {
        let root = commands
            .spawn((
                Screen,
                Node {
                    width: px(crate::components::tiles::CANVAS.x),
                    height: px(crate::components::tiles::CANVAS.y),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                UiTransform::IDENTITY,
            ))
            .with_children(|canvas| {
                canvas.spawn(crate::components::background::layer());
                canvas.spawn(crate::components::sparkles::layer());
            })
            .id();
        let room = commands
            .spawn((ChildOf(root), crate::app::presentation::room_node()))
            .id();
        let deck_back = atlases.deck_back(deck);
        let mut deck_pile = Entity::PLACEHOLDER;
        let mut deck_label = Entity::PLACEHOLDER;
        let mut ante_end = Entity::PLACEHOLDER;
        let mut blind_box = Entity::PLACEHOLDER;
        let mut blind_texts = [Entity::PLACEHOLDER; 4];
        let mut hud = [Entity::PLACEHOLDER; 8];
        let mut blind_chip = Entity::PLACEHOLDER;
        let mut shop_sign = Entity::PLACEHOLDER;
        let mut score_tiles = [Entity::PLACEHOLDER; 2];
        let mut hand_title = Entity::PLACEHOLDER;
        let mut inventory = [Entity::PLACEHOLDER; 2];
        let mut counts = [Entity::PLACEHOLDER; 2];
        let mut tags = Entity::PLACEHOLDER;
        let mut deck_count = Entity::PLACEHOLDER;
        commands.entity(room).with_children(|ui| {
            // `create_UIBox_HUD`: a faint dark rim, the DYN_UI.MAIN outline, then the body.
            // The body is `minh = 30` tiles, centred on the room: its top and bottom edges are
            // off screen and only the left and right MAIN bands show.
            ui.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(HUD_LEFT),
                    top: px(-HUD_OVERHANG * TILE),
                    bottom: px(-HUD_OVERHANG * TILE),
                    width: px(HUD_WIDTH),
                    padding: UiRect::all(px(0.03 * TILE)),
                    ..default()
                },
                PixelBox::new(TRANSPARENT_DARK),
            ))
            .with_children(|rim| {
                rim.spawn((
                    Node {
                        flex_grow: 1.0,
                        padding: UiRect::all(px(0.05 * TILE)),
                        ..default()
                    },
                    PixelBox::new(BLACK),
                    DynRole::Main,
                ))
                .with_children(|outline| {
                    outline
                        .spawn((
                            Node {
                                flex_grow: 1.0,
                                padding: UiRect::all(px(0.08 * TILE)),
                                justify_content: JustifyContent::Center,
                                ..column(0.08 * TILE)
                            },
                            PixelBox::new(BLACK),
                            DynRole::BossDark,
                        ))
                        .with_children(|panel| {
                            panel.spawn(Node {
                                height: px(0.3 * TILE),
                                ..default()
                            });
                            // `row_blind`: the blind or the shop sign.
                            panel
                                .spawn(Node {
                                    width: percent(100),
                                    height: px(3.75 * TILE),
                                    justify_content: JustifyContent::Center,
                                    ..column(0.12 * TILE)
                                })
                                .with_children(|blind| {
                                    hud[0] = value(blind, text_px(0.6), INK);
                                    hud[1] = value(blind, text_px(0.34), INK);
                                    blind_box = spawn_blind_box(
                                        blind,
                                        atlases.stake_chip(stake),
                                        &mut blind_chip,
                                        &mut blind_texts,
                                    );
                                    if let Some(image) = atlases.shop_sign() {
                                        shop_sign = spawn_shop_sign(blind, image);
                                    }
                                });
                            round_score(panel, atlases.stake_chip(stake), &mut hud[2]);
                            hand_area(panel, &mut hand_title, &mut score_tiles);
                            bottom_hud(panel, &mut hud, &mut ante_end);
                        });
                });
            });
            ui.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(CENTER_LEFT),
                top: px(0),
                column_gap: px(0.2 * TILE),
                ..default()
            })
            .with_children(|bar| {
                for (index, width) in [
                    crate::components::tiles::JOKER_AREA,
                    crate::components::tiles::CONSUMABLE_AREA,
                ]
                .into_iter()
                .enumerate()
                {
                    bar.spawn(Node {
                        width: px(width),
                        flex_shrink: 0.0,
                        align_items: AlignItems::Start,
                        ..column(3.0)
                    })
                    .with_children(|zone| {
                        inventory[index] = zone
                            .spawn((
                                Node {
                                    width: percent(100),
                                    height: px(AREA_HEIGHT),
                                    ..default()
                                },
                                PixelBox::new(Color::srgba(0.0, 0.0, 0.0, 0.1)),
                            ))
                            .id();
                        counts[index] = value(zone, text_px(0.3), INK);
                    });
                }
            });
            // Deck area of `set_screen_positions`; the pile is drawn by `deck_backs`.
            ui.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(DECK_LEFT),
                top: px(DECK_TOP),
                width: px(DECK_AREA.x),
                align_items: AlignItems::End,
                ..column(0.05 * TILE)
            })
            .with_children(|region| {
                let card = crate::components::tiles::card_px();
                let mut stack = region.spawn((
                    Node {
                        width: px(DECK_AREA.x),
                        height: px(DECK_AREA.y),
                        ..default()
                    },
                    crate::components::travel::DeckPile,
                ));
                stack
                    .observe(
                        |_: On<bevy::picking::prelude::PointerOver>,
                         mut clicks: MessageWriter<Click>,
                         mut labels: Query<&mut Visibility, With<DeckHoverLabel>>| {
                            clicks.write(Click::HoverDeck(true));
                            for mut label in &mut labels {
                                *label = Visibility::Inherited;
                            }
                        },
                    )
                    .observe(
                        |_: On<bevy::picking::prelude::PointerOut>,
                         mut clicks: MessageWriter<Click>,
                         mut labels: Query<&mut Visibility, With<DeckHoverLabel>>| {
                            clicks.write(Click::HoverDeck(false));
                            for mut label in &mut labels {
                                *label = Visibility::Hidden;
                            }
                        },
                    )
                    .observe(
                        |event: On<bevy::picking::prelude::PointerClick>,
                         mut clicks: MessageWriter<Click>| {
                            if event.button == bevy::picking::prelude::PointerButton::Primary {
                                clicks.write(Click::OpenDeck);
                            }
                        },
                    );
                deck_pile = stack.id();
                if deck_back.is_none() {
                    stack.with_children(|cards| {
                        cards
                            .spawn(Node {
                                position_type: PositionType::Absolute,
                                left: px((DECK_AREA.x - card.x) / 2.0),
                                top: px((DECK_AREA.y - card.y) / 2.0),
                                ..default()
                            })
                            .with_children(|fallback| {
                                crate::components::tiles::deck_preview_large(fallback, deck, None);
                            });
                    });
                }
                deck_label = deck_hover_label(region);
                deck_count = value(region, text_px(0.3), INK);
                // Held tags stand left of the deck without moving it.
                tags = region
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        right: percent(100),
                        bottom: px(0),
                        width: px(44),
                        flex_direction: FlexDirection::ColumnReverse,
                        row_gap: px(5),
                        ..default()
                    })
                    .id();
            });
        });
        for (index, entity) in score_tiles.into_iter().enumerate() {
            commands
                .entity(entity)
                .insert((ScoreTile(index == 0), UiTransform::IDENTITY));
        }
        let center = commands
            .spawn((ChildOf(room), center_layout(Phase::BeforeBlind)))
            .id();
        let error = commands
            .spawn((
                ChildOf(room),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(CENTER_LEFT),
                    bottom: px(10),
                    ..default()
                },
                Text::new(""),
                TextFont {
                    font_size: bevy::text::FontSize::Px(20.0),
                    ..default()
                },
                TextColor(RED),
                Pickable::IGNORE,
            ))
            .id();
        let overlay = commands
            .spawn((
                ChildOf(root),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                Pickable::IGNORE,
                ZIndex(50),
            ))
            .id();
        Self {
            root,
            overlay,
            run_id,
            center,
            inventory,
            counts,
            tags,
            deck_count,
            deck_pile,
            deck_label,
            deck_back,
            deck_shown: None,
            ante_end,
            blind_box,
            blind_texts,
            hud,
            blind_chip,
            shop_sign,
            score_tiles,
            hand_title,
            error,
            frame: None,
            tag_keys: Vec::new(),
            playing: None,
        }
    }
}

mod update;
