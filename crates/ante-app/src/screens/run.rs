use crate::components::{
    pixel_rect::{DynRole, PixelBox},
    tiles::{BLACK, CARD_H, CARD_W, IMPORTANT, MONEY, TILE, TRANSPARENT_DARK, darken, lighten},
    widgets::{shadowed_label, text_shadow},
};
use crate::{
    app::{
        navigation::{DeckChoice, STAKES, Screen as Page},
        playback::Speed,
        presentation::Screen,
        session::{Click, Session, parse_seed},
    },
    components::{
        cards::{inventory_cards, voucher_card},
        theme::{INK, MUTED},
        tiles::{BLUE, ORANGE, RED, text_px, tile_button, tile_optional},
        widgets::{atlas_image, card_hover, label, row, wrap_row},
    },
    screens::{
        hand::playing_board,
        shop::{pack_board, shop_board},
    },
    services::assets::catalog::Atlases,
};
use ante_core::contract::{BlindView, Command, HandKind, Phase, RANKS, RunView, SUITS};
use bevy::prelude::*;

const PANEL: Color = Color::srgb(0.20, 0.27, 0.28);
const WELL: Color = Color::srgb(0.10, 0.15, 0.16);
const CREAM: Color = Color::srgb(0.94, 0.95, 0.91);
const OUTLINE: Color = Color::srgb(0.66, 0.76, 0.77);
const GOLD: Color = Color::srgb(0.95, 0.71, 0.30);

fn column(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(gap),
        ..default()
    }
}

fn box_node(node: Node, color: Color) -> (Node, BackgroundColor) {
    (
        Node {
            border_radius: BorderRadius::all(px(12)),
            ..node
        },
        BackgroundColor(color),
    )
}

pub fn overlay(ui: &mut ChildSpawnerCommands, content: impl FnOnce(&mut ChildSpawnerCommands)) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.31, 0.40, 0.40, 0.65)),
        GlobalZIndex(200),
        Pickable {
            should_block_lower: true,
            is_hoverable: false,
        },
    ))
    .with_children(content);
}

fn modal(
    ui: &mut ChildSpawnerCommands,
    width: f32,
    gap: f32,
    content: impl FnOnce(&mut ChildSpawnerCommands),
) {
    ui.spawn((
        Node {
            width: px(width),
            flex_shrink: 0.0,
            padding: UiRect::all(px(16)),
            border: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(16)),
            ..column(gap)
        },
        BackgroundColor(PANEL),
        BorderColor::all(OUTLINE),
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.30),
            px(0),
            px(7),
            px(0),
            px(0),
        ),
    ))
    .with_children(content);
}

pub fn deck_select(
    ui: &mut ChildSpawnerCommands,
    session: &Session,
    atlases: &Atlases,
    backend: &dyn ante_core::contract::GameBackend,
) {
    let deck = session.navigation.deck.unwrap_or(DeckChoice::Red);
    let can_continue = session
        .run
        .as_ref()
        .is_some_and(|run| !matches!(run.view().phase, Phase::Won | Phase::Lost));
    modal(ui, 690.0, 18.0, |panel| {
        row(panel, |tabs| {
            tile_optional(tabs, "New Run", RED, (2.1, 0.68), 0.46, None, true);
            tile_optional(
                tabs,
                "Continue",
                RED,
                (2.1, 0.68),
                0.46,
                can_continue.then_some(Click::ContinueRun),
                false,
            );
            tile_optional(tabs, "Challenges", RED, (2.1, 0.68), 0.46, None, false);
        });
        row(panel, |picker| {
            tile_button(picker, "<", RED, (0.55, 2.9), 0.6, Click::ShiftDeck(-1));
            picker
                .spawn(box_node(
                    Node {
                        width: px(455),
                        height: px(211),
                        padding: UiRect::all(px(12)),
                        column_gap: px(16),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    WELL,
                ))
                .with_children(|preview| {
                    if let Some(back) = atlases.deck_back(deck) {
                        atlas_image(preview, back, 126.0, 169.0);
                    } else {
                        preview
                            .spawn(box_node(
                                Node {
                                    width: px(126),
                                    height: px(169),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                if deck == DeckChoice::Blue { BLUE } else { RED },
                            ))
                            .with_children(|back| {
                                label(back, "Deck", 26.0, INK);
                            });
                    }
                    preview
                        .spawn(box_node(
                            Node {
                                flex_grow: 1.0,
                                align_self: AlignSelf::Stretch,
                                justify_content: JustifyContent::Center,
                                ..column(8.0)
                            },
                            PANEL,
                        ))
                        .with_children(|details| {
                            label(details, deck.name(), 33.0, INK);
                            description(
                                details,
                                match deck {
                                    DeckChoice::Red => "+1 discard\neach round",
                                    DeckChoice::Blue => "+1 hand\neach round",
                                    _ => "Deck unavailable\nin this build",
                                },
                                256.0,
                                105.0,
                            );
                        });
                });
            tile_button(picker, ">", RED, (0.55, 2.9), 0.6, Click::ShiftDeck(1));
        });
        row(panel, |stake| {
            tile_optional(
                stake,
                "<",
                RED,
                (0.55, 1.36),
                0.5,
                (session.stake > 0).then_some(Click::ShiftStake(-1)),
                false,
            );
            stake
                .spawn(box_node(
                    Node {
                        width: px(455),
                        height: px(99),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        column_gap: px(12),
                        ..default()
                    },
                    WELL,
                ))
                .with_children(|preview| {
                    if let Some(chip) = atlases.stake_chip(session.stake) {
                        atlas_image(preview, chip, 58.0, 58.0);
                    }
                    preview.spawn(column(4.0)).with_children(|details| {
                        label(details, STAKES[session.stake], 24.0, INK);
                        description(
                            details,
                            if session.stake == 0 {
                                "Base difficulty"
                            } else {
                                "Locked in this build"
                            },
                            334.0,
                            46.0,
                        );
                    });
                });
            tile_optional(
                stake,
                ">",
                RED,
                (0.55, 1.36),
                0.5,
                (session.stake + 1 < STAKES.len()).then_some(Click::ShiftStake(1)),
                false,
            );
        });
        if session.seeded_run {
            row(panel, |seed| {
                let value = if session.seed.is_empty() {
                    "Enter seed"
                } else {
                    &session.seed
                };
                let value = if session.editing_seed {
                    format!("{value} _")
                } else {
                    value.into()
                };
                tile_button(seed, &value, BLUE, (5.0, 0.7), 0.37, Click::EditSeed);
                tile_button(seed, "Paste", BLUE, (1.65, 0.7), 0.39, Click::PasteSeed);
            });
        }
        row(panel, |actions| {
            let toggle = if session.seeded_run {
                "Seeded Run [x]"
            } else {
                "Seeded Run [ ]"
            };
            tile_button(
                actions,
                toggle,
                PANEL,
                (2.8, 0.82),
                0.34,
                Click::ToggleSeeded,
            );
            let can_start = deck.deck().is_some()
                && session.stake == 0
                && (!session.seeded_run || parse_seed(backend, &session.seed).is_some());
            tile_optional(
                actions,
                "PLAY",
                BLUE,
                (4.2, 0.96),
                0.73,
                can_start.then_some(Click::Start),
                false,
            );
        });
        if let Some(error) = &session.error {
            label(panel, error, 22.0, GOLD);
        }
        tile_button(panel, "Back", ORANGE, (8.8, 0.58), 0.46, Click::Back);
    });
}

fn description(ui: &mut ChildSpawnerCommands, text: &str, width: f32, height: f32) {
    ui.spawn(box_node(
        Node {
            width: px(width),
            height: px(height),
            padding: UiRect::all(px(8)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        CREAM,
    ))
    .with_children(|body| {
        label(body, text, 23.0, WELL);
    });
}

pub fn pause(ui: &mut ChildSpawnerCommands, playing_trace: bool) {
    modal(ui, 466.0, 10.0, |panel| {
        for (name, action) in [
            ("Settings", Click::OpenOptions),
            ("New Run", Click::OpenDecks),
            ("Restart Same Seed", Click::Restart),
            ("Main Menu", Click::MainMenu),
            ("Run Info", Click::OpenRunInfo),
            ("Collection", Click::Collection(0)),
        ] {
            tile_button(panel, name, RED, (4.4, 0.78), 0.50, action);
        }
        tile_optional(panel, "Customize Deck", RED, (4.4, 0.78), 0.45, None, false);
        tile_button(
            panel,
            if playing_trace { "Resume" } else { "Back" },
            ORANGE,
            (5.8, 0.62),
            0.45,
            Click::Back,
        );
    });
}

pub fn run_info(ui: &mut ChildSpawnerCommands, view: &RunView, atlases: &Atlases, tab: u8) {
    modal(ui, 760.0, 12.0, |panel| {
        row(panel, |tabs| {
            for (index, name) in [(0, "Poker Hands"), (1, "Blinds"), (2, "Vouchers")] {
                tile_optional(
                    tabs,
                    name,
                    RED,
                    (2.8, 0.65),
                    0.42,
                    Some(Click::RunInfoTab(index)),
                    tab == index,
                );
            }
        });
        match tab {
            1 => {
                for blind in &view.blinds {
                    row(panel, |line| {
                        if let Some(chip) = atlases.blind_chip(blind.id) {
                            atlas_image(line, chip, 33.0, 33.0);
                        }
                        label(
                            line,
                            format!("{}   {}", blind.name, score_text(blind.target)),
                            24.0,
                            INK,
                        );
                        label(line, &blind.description, 21.0, MUTED);
                    });
                }
            }
            2 => {
                if view.vouchers.is_empty() {
                    label(panel, "No vouchers redeemed", 24.0, MUTED);
                }
                wrap_row(panel, |cards| {
                    for voucher in &view.vouchers {
                        voucher_card(cards, voucher, atlases);
                    }
                });
            }
            _ => {
                for hand in view
                    .hand_levels
                    .iter()
                    .filter(|hand| hand.rank < HandKind::FiveOfAKind || hand.level.plays > 0)
                {
                    panel
                        .spawn(box_node(
                            Node {
                                width: percent(100),
                                min_height: px(35),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect::horizontal(px(12)),
                                ..default()
                            },
                            WELL,
                        ))
                        .with_children(|line| {
                            label(line, format!("Lv.{}", hand.level.level), 22.0, MUTED);
                            line.spawn(Node {
                                width: px(230),
                                justify_content: JustifyContent::Center,
                                ..default()
                            })
                            .with_children(|name| label(name, hand.name(), 25.0, INK));
                            label(line, score_text(hand.level.chips), 24.0, BLUE);
                            label(line, "x", 21.0, INK);
                            label(line, score_text(hand.level.mult), 24.0, RED);
                            label(line, format!("# {}", hand.level.plays), 22.0, GOLD);
                        });
                }
            }
        }
        tile_button(panel, "Back", ORANGE, (6.2, 0.62), 0.46, Click::Back);
    });
}

pub struct RunBoard {
    pub root: Entity,
    pub overlay: Entity,
    pub run_id: u64,
    center: Entity,
    inventory: [Entity; 2],
    counts: [Entity; 2],
    tags: Entity,
    deck_count: Entity,
    /// The deck area holding the stacked backs.
    deck_pile: Entity,
    deck_back: Option<ImageNode>,
    /// Cards in the deck when its backs were last drawn.
    deck_shown: Option<usize>,
    /// "/ 8" after the ante number.
    ante_end: Entity,
    /// `create_UIBox_HUD_blind`, shown while a blind is played.
    blind_box: Entity,
    /// Blind name, rule, target and reward.
    blind_texts: [Entity; 4],
    hud: [Entity; 8],
    blind_chip: Entity,
    shop_sign: Entity,
    score_tiles: [Entity; 2],
    hand_title: Entity,
    error: Entity,
    frame: Option<BoardFrame>,
    tag_keys: Vec<&'static str>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BoardFrame {
    revision: u64,
    trace: bool,
    speed: Speed,
    preview: bool,
    offer: Option<Command>,
    inventory_selection: Option<(crate::components::card_drag::Lane, usize, u64)>,
}

/// The HUD sits 0.7 tiles left of the room (`align = 'cli'`, offset −0.7) over the full
/// canvas height; its content is about 5.1 tiles wide.
const HUD_LEFT: f32 = -0.7 * TILE;
const HUD_WIDTH: f32 = 5.1 * TILE;
/// How far the HUD body runs past the room, in tiles: `(30 - ROOM.y) / 2`.
const HUD_OVERHANG: f32 = (30.0 - 11.5) / 2.0;
/// Card areas from `set_screen_positions`: the hand starts at 20 − 6·CARD_W − 2.85 tiles and
/// the Jokers 0.1 tile further left, at the top of the room.
const HAND_X: f32 = 20.0 - 6.0 * CARD_W - 2.85;
const CENTER_LEFT: f32 = (HAND_X - 0.1) * TILE;
const AREA_HEIGHT: f32 = 0.95 * CARD_H * TILE;
/// Deck area: 1.1 card wide, 0.5 tile from the room's right edge, level with the hand.
const DECK_LEFT: f32 = (20.0 - 1.1 * CARD_W - 0.5) * TILE;
const DECK_TOP: f32 = (11.5 - 0.95 * CARD_H) * TILE;
const DECK_AREA: Vec2 = Vec2::new(1.1 * CARD_W * TILE, 0.95 * CARD_H * TILE);
/// Scale of the stack spread; 1 is the original `deck_height`. Lower it to tighten the pile.
const DECK_STACK_SCALE: f32 = 0.6;
/// Brightness of the backs under the top card.
const DECK_UNDER_SHADE: f32 = 0.78;
/// `shadow_parrallax.x` of the deck area: (centre − 10) / 10 x 1.5.
const DECK_PARALLAX_X: f32 = ((20.0 - 0.55 * CARD_W - 0.5) - 10.0) / 10.0 * 1.5;

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
fn drawn_backs(n: usize) -> impl Iterator<Item = usize> {
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
                deck_hover_label(region);
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
        }
    }

    pub fn update(
        &mut self,
        commands: &mut Commands,
        view: &RunView,
        atlases: &Atlases,
        session: &Session,
        text: &crate::services::assets::ui_text::UiText,
    ) {
        let trace = session.playback.as_ref();
        let preview = session.deck_hovered
            && trace.is_none()
            && session.navigation.screen == Page::Run
            && view.phase == Phase::Playing;
        let frame = BoardFrame {
            revision: view.revision,
            trace: trace.is_some(),
            speed: session.speed,
            preview,
            offer: session.shop_selection,
            inventory_selection: session.inventory_selection,
        };
        commands
            .entity(self.error)
            .insert(Text::new(session.error.as_deref().unwrap_or_default()));
        if self.frame == Some(frame) {
            return;
        }
        self.frame = Some(frame);
        let title = match view.phase {
            Phase::BeforeBlind => "Choose your\nnext Blind",
            Phase::Shop if self.shop_sign != Entity::PLACEHOLDER => "",
            Phase::Shop => "Shop",
            _ => &view.blind,
        };
        let in_blind = view.phase == Phase::Playing;
        let title = if in_blind { "" } else { title };
        let rule = String::new();
        for (entity, text) in self.hud.into_iter().zip([
            title.to_owned(),
            rule,
            score_text(view.score),
            view.hands.to_string(),
            view.discards.to_string(),
            format!("${}", view.money),
            view.ante.to_string(),
            view.round.to_string(),
        ]) {
            commands.entity(entity).insert(Text::new(text));
        }
        commands
            .entity(self.ante_end)
            .insert(Text::new(if view.endless_mode {
                String::new()
            } else {
                format!(" / {}", view.ante_end)
            }));
        if self.shop_sign != Entity::PLACEHOLDER {
            commands
                .entity(self.shop_sign)
                .insert(shop_sign_node(if view.phase == Phase::Shop {
                    Display::Flex
                } else {
                    Display::None
                }));
        }
        commands
            .entity(self.blind_box)
            .insert(blind_box_node(if in_blind {
                Display::Flex
            } else {
                Display::None
            }));
        if in_blind {
            if let Some(chip) = view.blind_chip.and_then(|id| atlases.blind_chip(id)) {
                commands.entity(self.blind_chip).insert(chip);
            }
            let [name, rule, target, reward] = self.blind_texts;
            commands.entity(name).insert(Text::new(view.blind.clone()));
            commands
                .entity(rule)
                .insert(Text::new(view.blind_rule.clone()));
            // `blind_chip_UI_scale`; only the size changes, the game font stays.
            commands.entity(target).insert((
                Text::new(score_text(view.target)),
                crate::app::presentation::TextSize(text_px(scale_number(
                    view.target as f64,
                    0.7,
                    100_000.0,
                ))),
            ));
            commands
                .entity(reward)
                .insert(Text::new("$".repeat(view.reward)));
        }
        commands.entity(self.hand_title).insert(Text::new(
            view.selected_hand.map_or_else(String::new, |hand| {
                format!("{}  lvl.{}", hand.name(), hand.level.level)
            }),
        ));
        let (chips, mult) = trace
            .and_then(|(_, playback)| playback.score_values())
            .unwrap_or_else(|| {
                view.selected_hand
                    .map_or((0, 0.0), |hand| (hand.level.chips, hand.level.mult as f64))
            });
        commands
            .entity(self.score_tiles[0])
            .insert(Text::new(score_text(chips)));
        commands
            .entity(self.score_tiles[1])
            .insert(Text::new(mult_text(mult)));
        if self.deck_shown != Some(view.deck_cards)
            && let Some(back) = self.deck_back.clone()
        {
            self.deck_shown = Some(view.deck_cards);
            let card = crate::components::tiles::card_px();
            let n = view.deck_cards;
            commands
                .entity(self.deck_pile)
                .despawn_children()
                .with_children(|pile| {
                    for k in drawn_backs(n) {
                        let offset = deck_offset(n, k) * TILE;
                        // Cards under the top one read greyer, like their stacked shadows.
                        let shade = if k == 1 { 1.0 } else { DECK_UNDER_SHADE };
                        pile.spawn((
                            ImageNode {
                                color: Color::srgb(shade, shade, shade),
                                ..back.clone()
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                left: px((DECK_AREA.x - card.x) / 2.0 + offset.x),
                                top: px((DECK_AREA.y - card.y) / 2.0 + offset.y),
                                width: px(card.x),
                                height: px(card.y),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    }
                });
        }
        commands.entity(self.deck_count).insert(Text::new(format!(
            "{}/{}",
            view.deck_cards, view.deck_total
        )));
        for (i, text) in [
            format!("{}/{}", view.held_jokers.len(), view.joker_slots),
            format!("{}/{}", view.consumables.len(), view.consumable_slots),
        ]
        .into_iter()
        .enumerate()
        {
            commands.entity(self.counts[i]).insert(Text::new(text));
            commands
                .entity(self.inventory[i])
                .despawn_children()
                .with_children(|zone| {
                    inventory_cards(
                        zone,
                        view,
                        atlases,
                        trace.is_some(),
                        i == 0,
                        session.inventory_selection,
                    );
                });
        }
        let keys: Vec<_> = view.tags.iter().map(|tag| tag.content.id.0).collect();
        if self.tag_keys != keys {
            self.tag_keys = keys;
            commands
                .entity(self.tags)
                .despawn_children()
                .with_children(|stack| {
                    for tag in &view.tags {
                        tag_icon(stack, atlases, tag);
                    }
                });
        }
        commands
            .entity(self.center)
            .insert(center_layout(view.phase))
            .despawn_children()
            .with_children(|center| match view.phase {
                Phase::BeforeBlind => {
                    for (index, blind) in view.blinds.iter().enumerate() {
                        blind_panel(center, blind, index, atlases, view.revision);
                    }
                    if view.can_reroll_boss
                        || view
                            .vouchers
                            .iter()
                            .any(|v| matches!(v.id.0, "v_directors_cut" | "v_retcon"))
                    {
                        tile_optional(
                            center,
                            "Reroll Boss $10",
                            ORANGE,
                            (2.65, 0.65),
                            0.43,
                            view.can_reroll_boss
                                .then_some(Click::Play(Command::RerollBoss)),
                            false,
                        );
                    }
                }
                Phase::Playing => playing_board(
                    center,
                    view,
                    atlases,
                    trace.is_some(),
                    session.speed,
                    preview,
                ),
                Phase::Shop => {
                    shop_board(center, view, atlases, session.shop_selection, text);
                }
                Phase::Reward => {
                    label(center, "Blind cleared", 38.0, INK);
                    label(
                        center,
                        format!("Round score {}", score_text(view.score)),
                        25.0,
                        INK,
                    );
                    tile_button(
                        center,
                        &format!("Cash Out +${}", view.reward),
                        ORANGE,
                        (3.4, 1.0),
                        0.52,
                        Click::Play(Command::CashOut),
                    );
                }
                Phase::Pack | Phase::Target => pack_board(center, view, atlases),
                Phase::Won | Phase::Lost => {}
            });
    }
}

/// Mult as the original `number_format` writes it: whole numbers as is, otherwise two
/// decimals under 10, one under 100 and none above, with thousands separators.
pub fn mult_text(mult: f64) -> String {
    if mult >= 100_000_000_000.0 {
        return format!("{mult:.3e}");
    }
    let text = if mult == mult.floor() || mult >= 100.0 {
        format!("{mult:.0}")
    } else if mult >= 10.0 {
        format!("{mult:.1}")
    } else {
        format!("{mult:.2}")
    };
    let (whole, fraction) = text
        .split_once('.')
        .map_or((text.as_str(), None), |(w, f)| (w, Some(f)));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    match fraction {
        Some(fraction) => format!("{grouped}.{fraction}"),
        None => grouped,
    }
}

pub fn score_text(score: usize) -> String {
    if score as u128 >= 100_000_000_000 {
        format!("{:.3e}", score as f64)
    } else {
        score.to_string()
    }
}

fn value(ui: &mut ChildSpawnerCommands, size: f32, color: Color) -> Entity {
    ui.spawn((
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        text_shadow(),
        Pickable::IGNORE,
    ))
    .id()
}

/// "View Deck" over the deck while it is hovered.
#[derive(Component)]
pub(crate) struct DeckHoverLabel;

fn deck_hover_label(ui: &mut ChildSpawnerCommands) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(DECK_AREA.x),
            height: px(DECK_AREA.y),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Hidden,
        DeckHoverLabel,
        Pickable::IGNORE,
    ))
    .with_children(|centre| {
        centre
            .spawn((
                Node {
                    padding: UiRect::all(px(0.1 * TILE)),
                    ..column(0.0)
                },
                PixelBox::new(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                Pickable::IGNORE,
            ))
            .with_children(|text| {
                shadowed_label(text, "View", text_px(0.5), INK);
                shadowed_label(text, "Deck", text_px(0.4), INK);
            });
    });
}

fn blind_box_node(display: Display) -> Node {
    Node {
        display,
        min_width: px(4.5 * TILE),
        padding: UiRect::all(px(0.05 * TILE)),
        ..column(0.05 * TILE)
    }
}

/// `scale_number`: text shrinks as the number grows past `max`.
fn scale_number(number: f64, scale: f32, max: f64) -> f32 {
    if number < max {
        scale
    } else {
        (scale as f64 * (max / number).powf(0.25)).max(0.25) as f32
    }
}

/// `create_UIBox_HUD_blind`: the blind name in DYN_UI.MAIN over a DYN_UI.DARK box with the
/// rule, the blind chip and a black box with the target and reward.
fn spawn_blind_box(
    ui: &mut ChildSpawnerCommands,
    stake: Option<ImageNode>,
    chip: &mut Entity,
    texts: &mut [Entity; 4],
) -> Entity {
    ui.spawn((
        blind_box_node(Display::None),
        PixelBox::new(BLACK).emboss(0.05),
    ))
    .with_children(|blind| {
        blind
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(0.7 * TILE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(BLACK).emboss(0.05),
                DynRole::Main,
            ))
            .with_children(|name| {
                texts[0] = name
                    .spawn((
                        Text::new(""),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(text_px(0.64)),
                            ..default()
                        },
                        TextColor(INK),
                        text_shadow(),
                        Floating,
                        UiTransform::IDENTITY,
                        Pickable::IGNORE,
                    ))
                    .id();
            });
        blind
            .spawn((
                Node {
                    width: percent(100),
                    min_height: px(2.74 * TILE),
                    justify_content: JustifyContent::Center,
                    ..column(0.0)
                },
                PixelBox::new(BLACK),
                DynRole::Dark,
            ))
            .with_children(|body| {
                texts[1] = body
                    .spawn((
                        Node {
                            max_width: px(4.2 * TILE),
                            padding: UiRect::all(px(0.05 * TILE)),
                            ..default()
                        },
                        Text::new(""),
                        TextFont {
                            font_size: bevy::text::FontSize::Px(text_px(0.36)),
                            ..default()
                        },
                        TextLayout::justify(Justify::Center),
                        TextColor(INK),
                        Pickable::IGNORE,
                    ))
                    .id();
                body.spawn(Node {
                    padding: UiRect::all(px(0.15 * TILE)),
                    column_gap: px(0.15 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    *chip = row
                        .spawn((
                            Node {
                                width: px(1.5 * TILE),
                                height: px(1.5 * TILE),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .id();
                    row.spawn((
                        Node {
                            min_width: px(2.9 * TILE),
                            padding: UiRect::all(px(0.05 * TILE)),
                            ..column(0.0)
                        },
                        PixelBox::new(BLACK).emboss(0.05),
                    ))
                    .with_children(|target| {
                        target.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: bevy::text::FontSize::Px(text_px(0.3)),
                                ..default()
                            },
                            TextColor(INK),
                            text_shadow(),
                            LocalText::new("ph_blind_score_at_least"),
                            Pickable::IGNORE,
                        ));
                        target
                            .spawn(Node {
                                min_height: px(0.6 * TILE),
                                column_gap: px(0.1 * TILE),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|count| {
                                if let Some(stake) = stake {
                                    atlas_image(count, stake, 0.5 * TILE, 0.5 * TILE);
                                }
                                texts[2] = value(count, text_px(0.7), RED);
                            });
                        target
                            .spawn(Node {
                                min_height: px(0.45 * TILE),
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|reward| {
                                reward.spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: bevy::text::FontSize::Px(text_px(0.3)),
                                        ..default()
                                    },
                                    TextColor(INK),
                                    LocalText::new("ph_blind_reward"),
                                    Pickable::IGNORE,
                                ));
                                texts[3] = value(reward, text_px(0.45), MONEY);
                            });
                    });
                });
            });
    })
    .id()
}

/// Spacing between HUD counters (`spacing` in `create_UIBox_HUD`).
const HUD_SPACING: f32 = 0.13;

/// `G.SHOP_SIGN`: the animated sign and the gold line in a DYN_UI.DARK box framed by MAIN.
fn spawn_shop_sign(ui: &mut ChildSpawnerCommands, image: ImageNode) -> Entity {
    ui.spawn((
        shop_sign_node(Display::None),
        PixelBox::new(BLACK).emboss(0.05),
        DynRole::Main,
        Pickable::IGNORE,
    ))
    .with_children(|frame| {
        frame
            .spawn((
                Node {
                    min_width: px(4.72 * TILE),
                    min_height: px(3.1 * TILE),
                    padding: UiRect::all(px(0.1 * TILE)),
                    justify_content: JustifyContent::Center,
                    ..column(0.1 * TILE)
                },
                PixelBox::new(BLACK),
                DynRole::Dark,
                Pickable::IGNORE,
            ))
            .with_children(|sign| {
                sign.spawn((
                    image,
                    Node {
                        width: px(4.4 * TILE),
                        height: px(2.2 * TILE),
                        ..default()
                    },
                    ShopSignFrames,
                    Pickable::IGNORE,
                ));
                sign.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(text_px(0.5)),
                        ..default()
                    },
                    TextColor(lighten(crate::components::tiles::GOLD, 0.3)),
                    text_shadow(),
                    LocalText::new("ph_improve_run"),
                    Floating,
                    UiTransform::IDENTITY,
                    Pickable::IGNORE,
                ));
            });
    })
    .id()
}

fn shop_sign_node(display: Display) -> Node {
    Node {
        display,
        padding: UiRect::all(px(0.1 * TILE)),
        ..default()
    }
}

/// `contents.dollars_chips`: "Round score" beside a dark box with the stake chip and score.
fn round_score(ui: &mut ChildSpawnerCommands, stake: Option<ImageNode>, score: &mut Entity) {
    ui.spawn((
        Node {
            padding: UiRect::all(px(0.1 * TILE)),
            column_gap: px(0.1 * TILE),
            align_items: AlignItems::Center,
            ..default()
        },
        PixelBox::new(BLACK).emboss(0.05),
        DynRole::BossMain,
    ))
    .with_children(|row| {
        row.spawn(Node {
            min_width: px(1.3 * TILE),
            ..column(0.0)
        })
        .with_children(|title| {
            shadowed_label(title, "Round", text_px(0.42), INK);
            shadowed_label(title, "score", text_px(0.42), INK);
        });
        row.spawn((
            Node {
                min_width: px(3.3 * TILE),
                min_height: px(0.7 * TILE),
                column_gap: px(0.1 * TILE),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            PixelBox::new(BLACK),
            DynRole::BossDark,
        ))
        .with_children(|count| {
            if let Some(chip) = stake {
                atlas_image(count, chip, 0.5 * TILE, 0.5 * TILE);
            }
            *score = value(count, text_px(0.85), INK);
        });
    });
}

/// `contents.hand`: the hand name and level over Chips x Mult.
fn hand_area(ui: &mut ChildSpawnerCommands, title: &mut Entity, tiles: &mut [Entity; 2]) {
    ui.spawn((
        Node {
            padding: UiRect::all(px(0.03 * TILE)),
            ..column(0.0)
        },
        PixelBox::new(darken(BLACK, 0.1)).emboss(0.05),
    ))
    .with_children(|area| {
        area.spawn(Node {
            min_height: px(1.1 * TILE),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|name| {
            *title = value(name, text_px(0.56), INK);
        });
        area.spawn(Node {
            min_height: px(TILE),
            padding: UiRect::all(px(0.1 * TILE)),
            column_gap: px(0.05 * TILE),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            for (index, color) in [BLUE, RED].into_iter().enumerate() {
                if index == 1 {
                    shadowed_label(row, "X", text_px(0.8), RED);
                }
                row.spawn((
                    Node {
                        min_width: px(2.0 * TILE),
                        min_height: px(TILE),
                        padding: UiRect::horizontal(px(0.1 * TILE)),
                        align_items: AlignItems::Center,
                        justify_content: if index == 0 {
                            JustifyContent::End
                        } else {
                            JustifyContent::Start
                        },
                        ..default()
                    },
                    PixelBox::new(color).emboss(0.05),
                ))
                .with_children(|tile| {
                    tiles[index] = value(tile, text_px(0.92), INK);
                });
            }
        });
    });
}

/// `contents.buttons` and `contents.round`.
fn bottom_hud(ui: &mut ChildSpawnerCommands, hud: &mut [Entity; 8], ante_end: &mut Entity) {
    ui.spawn(Node {
        align_items: AlignItems::Center,
        ..default()
    })
    .with_children(|bottom| {
        bottom
            .spawn(Node {
                padding: UiRect::all(px(0.2 * TILE)),
                ..column(0.2 * TILE)
            })
            .with_children(|actions| {
                hud_button(actions, &["Run", "Info"], RED, Click::OpenRunInfo);
                hud_button(actions, &["Options"], ORANGE, Click::OpenPause);
            });
        bottom
            .spawn(column(HUD_SPACING * TILE))
            .with_children(|stats| {
                stats
                    .spawn(Node {
                        column_gap: px(HUD_SPACING * TILE),
                        ..default()
                    })
                    .with_children(|pair| {
                        hud[3] = counter(pair, Some("Hands"), (1.45, 0.0), BLUE, 0.8).0;
                        hud[4] = counter(pair, Some("Discards"), (1.45, 0.0), RED, 0.8).0;
                    });
                hud[5] = counter(stats, None, (1.45 * 2.0 + HUD_SPACING, 1.15), MONEY, 0.88).0;
                stats
                    .spawn(Node {
                        column_gap: px(HUD_SPACING * TILE),
                        ..default()
                    })
                    .with_children(|pair| {
                        let (ante, inner) =
                            counter(pair, Some("Ante"), (1.45, 1.0), IMPORTANT, 0.8);
                        hud[6] = ante;
                        pair.commands_mut().entity(inner).with_children(|box_| {
                            *ante_end = box_
                                .spawn((
                                    Text::new(""),
                                    TextFont {
                                        font_size: bevy::text::FontSize::Px(text_px(0.4)),
                                        ..default()
                                    },
                                    TextColor(INK),
                                    text_shadow(),
                                    Pickable::IGNORE,
                                ))
                                .id();
                        });
                        hud[7] = counter(pair, Some("Round"), (1.45, 1.0), IMPORTANT, 0.8).0;
                    });
            });
    });
}

/// Run Info and Options: 1.5 x 1.75 with a shadow.
fn hud_button(ui: &mut ChildSpawnerCommands, lines: &[&str], color: Color, action: Click) {
    let rows: Vec<[(&str, f32); 1]> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            [(
                *line,
                if index == 0 && lines.len() > 1 {
                    0.48
                } else {
                    0.4
                },
            )]
        })
        .collect();
    let rows: Vec<&[(&str, f32)]> = rows.iter().map(|row| row.as_slice()).collect();
    crate::components::tiles::card_button(
        ui,
        &rows,
        color,
        (1.5, 1.75),
        UiRect::all(px(0.05 * TILE)),
        Some(action),
    );
}

/// A HUD counter: a BOSS_MAIN box with its title over a BOSS_DARK box holding the number.
/// Returns the number and the inner box.
fn counter(
    ui: &mut ChildSpawnerCommands,
    title: Option<&str>,
    (min_w, min_h): (f32, f32),
    color: Color,
    scale: f32,
) -> (Entity, Entity) {
    let mut number = Entity::PLACEHOLDER;
    let mut inner = Entity::PLACEHOLDER;
    ui.spawn((
        Node {
            min_width: px(min_w * TILE),
            min_height: px(min_h * TILE),
            padding: UiRect::all(px(0.05 * TILE)),
            justify_content: JustifyContent::Center,
            ..column(0.0)
        },
        PixelBox::new(BLACK).emboss(0.05),
        DynRole::BossMain,
    ))
    .with_children(|field| {
        if let Some(title) = title {
            field
                .spawn(Node {
                    min_height: px(0.33 * TILE),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| shadowed_label(row, title, text_px(0.34), INK));
        }
        let wide = title.is_none();
        inner = field
            .spawn((
                Node {
                    min_width: px(if wide { 1.28 * 2.0 + HUD_SPACING } else { 1.2 } * TILE),
                    min_height: px(if wide { 1.0 } else { 0.0 } * TILE),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                PixelBox::new(BLACK),
                DynRole::BossDark,
            ))
            .with_children(|number_box| {
                number = value(number_box, text_px(scale), color);
            })
            .id();
    });
    (number, inner)
}

/// The shop sign atlas holds 4 frames of 113 x 57, played at `G.ANIMATION_FPS` = 10.
#[derive(Component)]
pub(crate) struct ShopSignFrames;

/// DynaText `float`: a slow sway and bob.
#[derive(Component)]
pub(crate) struct Floating;

/// Text read from the local localization by key.
#[derive(Component)]
pub(crate) struct LocalText {
    key: &'static str,
    var: Option<String>,
}

impl LocalText {
    pub(crate) fn new(key: &'static str) -> Self {
        Self { key, var: None }
    }

    /// A string with `#1#` replaced by `var`.
    pub(crate) fn with_var(key: &'static str, var: impl ToString) -> Self {
        Self {
            key,
            var: Some(var.to_string()),
        }
    }
}

const SIGN_FRAMES: u32 = 4;
const SIGN_FRAME: Vec2 = Vec2::new(113.0, 57.0);
const ANIMATION_FPS: f32 = 10.0;

pub(crate) fn animate_shop_sign(
    time: Res<Time<Real>>,
    mut signs: Query<&mut ImageNode, With<ShopSignFrames>>,
) {
    let frame = (time.elapsed_secs() * ANIMATION_FPS) as u32 % SIGN_FRAMES;
    let min = Vec2::new(frame as f32 * SIGN_FRAME.x, 0.0);
    for mut image in &mut signs {
        let rect = Rect::from_corners(min, min + SIGN_FRAME);
        if image.rect != Some(rect) {
            image.rect = Some(rect);
        }
    }
}

pub(crate) fn float_text(
    time: Res<Time<Real>>,
    mut texts: Query<&mut UiTransform, With<Floating>>,
) {
    let t = time.elapsed_secs();
    for mut transform in &mut texts {
        transform.rotation = Rot2::radians(0.02 * (2.0 * t).sin());
        transform.translation = Val2::px(0.0, -(1.0 + 0.3 * (0.666 * t).sin()) * 0.05 * TILE);
    }
}

pub(crate) fn fill_local_text(
    text: Option<Res<crate::services::assets::ui_text::UiText>>,
    mut nodes: Query<(&LocalText, &mut Text), Added<LocalText>>,
) {
    let Some(text) = text else {
        return;
    };
    for (local, mut value) in &mut nodes {
        value.0 = match &local.var {
            Some(var) => text.with_var(local.key, var),
            None => text.get(local.key),
        };
    }
}

fn center_layout(phase: Phase) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        left: px(CENTER_LEFT),
        right: px(204),
        top: px(219),
        bottom: px(28),
        justify_content: if phase == Phase::Playing {
            JustifyContent::End
        } else {
            JustifyContent::Center
        },
        ..column(14.0)
    };
    if phase == Phase::Shop {
        // `G.shop` aligns 'tmi' on the hand with offset −5.3; it runs below the window.
        node.top = px((11.5 - 0.95 * CARD_H - 5.3) * TILE);
        node.bottom = Val::Auto;
        node.justify_content = JustifyContent::Start;
        node.align_items = AlignItems::Center;
    }
    if phase == Phase::BeforeBlind {
        node.flex_direction = FlexDirection::Row;
        node.align_items = AlignItems::Stretch;
        node.column_gap = px(15);
    }
    node
}

fn tag_icon(ui: &mut ChildSpawnerCommands, atlases: &Atlases, tag: &ante_core::contract::TagView) {
    let mut icon = ui.spawn(Node {
        width: px(43),
        height: px(43),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });
    crate::components::tooltip::attach(
        &mut icon,
        crate::components::tooltip::Tip::new(&tag.name, &tag.content.description)
            .badge("Tag")
            .info(crate::components::info::references(tag.content.id.0))
            .technical(tag.content.id.0),
    );
    icon.with_children(|art| {
        if let Some(image) = atlases.tag_icon(tag.content.id.0) {
            atlas_image(art, image, 43.0, 43.0);
        } else {
            label(art, "Tag", 19.0, GOLD);
        }
    });
}

fn blind_panel(
    ui: &mut ChildSpawnerCommands,
    blind: &BlindView,
    index: usize,
    atlases: &Atlases,
    revision: u64,
) {
    let color = match index {
        0 => BLUE,
        1 => GOLD,
        _ => Color::srgb(0.63, 0.63, 0.55),
    };
    let muted = crate::components::tiles::emboss(color);
    let offset = if blind.selectable { 0.0 } else { 38.0 };
    let mut panel = ui.spawn((
        Node {
            margin: UiRect::top(px(offset)),
            flex_grow: 1.0,
            flex_basis: px(0),
            max_width: px(260),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(4)),
            border_radius: BorderRadius::all(px(15)),
            ..column(14.0)
        },
        BackgroundColor(if blind.selectable {
            PANEL
        } else {
            Color::srgb(0.16, 0.23, 0.23)
        }),
        BorderColor::all(if blind.selectable { color } else { muted }),
    ));
    let effect = if blind.description.is_empty() {
        format!("Score at least {}", blind.target)
    } else {
        blind.description.clone()
    };
    crate::components::tooltip::attach(
        &mut panel,
        crate::components::tooltip::Tip::new(&blind.name, effect)
            .badge(["Small", "Big", "Boss"][index])
            .technical(format!(
                "{} · target {} · reward {}",
                blind.id, blind.target, blind.reward
            ))
            .technical(format!(
                "Selectable: {} · passed: {} · skipped: {} · can skip: {}",
                blind.selectable, blind.passed, blind.skipped, blind.can_skip
            ))
            .technical(format!(
                "Tag: {}",
                blind.tag.as_ref().map_or("none", |tag| tag.content.id.0)
            )),
    );
    panel.with_children(|card| {
        let selection = if blind.selectable {
            "Select"
        } else if blind.skipped {
            "Skipped"
        } else if blind.passed {
            "Defeated"
        } else {
            "Upcoming"
        };
        tile_optional(
            card,
            selection,
            ORANGE,
            (2.49, 0.64),
            0.42,
            blind.selectable.then_some(Click::Play(Command::StartBlind)),
            false,
        );
        label(
            card,
            &blind.name,
            text_px(0.45),
            if blind.selectable { INK } else { MUTED },
        );
        if let Some(mut chip) = atlases.blind_chip(blind.id) {
            if !blind.selectable {
                chip.color = Color::srgb(0.65, 0.66, 0.62);
            }
            atlas_image(card, chip, 86.0, 86.0);
        } else {
            label(card, ["Small", "Big", "Boss"][index], 28.0, color);
        }
        card.spawn(box_node(
            Node {
                width: percent(100),
                padding: UiRect::all(px(10)),
                ..column(4.0)
            },
            WELL,
        ))
        .with_children(|score| {
            if !blind.description.is_empty() {
                label(score, &blind.description, 21.0, MUTED);
            }
            label(score, "Score at least", 21.0, INK);
            label(score, score_text(blind.target), 36.0, RED);
            label(
                score,
                format!("Reward: {}", "$".repeat(blind.reward)),
                20.0,
                GOLD,
            );
        });
        if let Some(tag) = &blind.tag {
            label(card, "or", 23.0, INK);
            row(card, |skip| {
                tag_icon(skip, atlases, tag);
                tile_optional(
                    skip,
                    "Skip Blind",
                    RED,
                    (1.92, 0.68),
                    0.37,
                    blind
                        .can_skip
                        .then_some(Click::Play(Command::SkipBlind { revision })),
                    false,
                );
            });
            label(card, &tag.name, 20.0, MUTED);
        } else {
            label(card, "Final blind of the ante", 20.0, GOLD);
        }
    });
}

pub fn inspect_deck(
    ui: &mut ChildSpawnerCommands,
    view: &RunView,
    atlases: &Atlases,
    deck: DeckChoice,
    remaining: bool,
) {
    let remaining = remaining && view.phase == Phase::Playing;
    let mut ranks = [0; 13];
    let mut suits = [0; 4];
    for card in view
        .deck_view
        .iter()
        .filter(|card| !remaining || card.remaining)
    {
        if let Some(rank) = RANKS.find(card.rank) {
            ranks[rank] += 1;
        }
        suits[card.suit] += 1;
    }
    modal(ui, 1200.0, 16.0, |panel| {
        row(panel, |tabs| {
            if view.phase == Phase::Playing {
                tile_optional(
                    tabs,
                    "Remaining",
                    RED,
                    (2.8, 0.78),
                    0.5,
                    Some(Click::ShowRemaining(true)),
                    remaining,
                );
            }
            tile_optional(
                tabs,
                "Full Deck",
                RED,
                (2.8, 0.78),
                0.5,
                Some(Click::ShowRemaining(false)),
                !remaining,
            );
        });
        panel
            .spawn(Node {
                width: percent(100),
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|body| {
                body.spawn(box_node(
                    Node {
                        width: px(208),
                        padding: UiRect::all(px(8)),
                        ..column(11.0)
                    },
                    WELL,
                ))
                .with_children(|info| {
                    label(info, deck.name(), 28.0, INK);
                    description(
                        info,
                        match deck {
                            DeckChoice::Red => "+1 discard\neach round",
                            DeckChoice::Blue => "+1 hand\neach round",
                            _ => deck.effect(),
                        },
                        185.0,
                        85.0,
                    );
                    label(
                        info,
                        format!("Base cards  {}", suits.iter().sum::<usize>()),
                        24.0,
                        RED,
                    );
                    for (suit, count) in SUITS.into_iter().zip(suits) {
                        label(info, format!("{suit}  {count}"), 22.0, INK);
                    }
                });
                body.spawn(box_node(
                    Node {
                        width: px(95),
                        padding: UiRect::all(px(8)),
                        ..column(1.0)
                    },
                    WELL,
                ))
                .with_children(|counts| {
                    for (rank, count) in RANKS.chars().zip(ranks) {
                        let rank = if rank == 'T' {
                            "10".to_owned()
                        } else {
                            rank.to_string()
                        };
                        label(counts, format!("{rank}   {count}"), 24.0, INK);
                    }
                });
                body.spawn(Node {
                    width: px(820),
                    ..column(7.0)
                })
                .with_children(|lanes| {
                    for (suit, name) in SUITS.into_iter().enumerate() {
                        let cards = view
                            .deck_view
                            .iter()
                            .filter(|card| card.suit == suit && (!remaining || card.remaining));
                        let count = suits[suit];
                        lanes
                            .spawn(box_node(
                                Node {
                                    width: percent(100),
                                    height: px(101),
                                    ..default()
                                },
                                WELL,
                            ))
                            .with_children(|lane| {
                                if count == 0 {
                                    label(lane, format!("{name} — 0"), 20.0, MUTED);
                                }
                                let width = 64.0;
                                let step = ((820.0 - width - 16.0)
                                    / count.saturating_sub(1).max(1) as f32)
                                    .min(66.0);
                                for (index, card) in cards.enumerate() {
                                    let mut entity = lane.spawn(box_node(
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: px(8.0 + index as f32 * step),
                                            top: px(6),
                                            width: px(width),
                                            height: px(85.6),
                                            align_items: AlignItems::Center,
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        CREAM,
                                    ));
                                    card_hover(&mut entity);
                                    crate::components::tooltip::attach_anchored(
                                &mut entity,
                                crate::components::tooltip::playing_card(
                                    card.rank,
                                    name,
                                    card.enhancement.unwrap_or_default(),
                                    card.edition,
                                    card.seal.unwrap_or_default(),
                                    card.debuffed,
                                )
                                .technical(deck_card_effect(card))
                                .technical(format!("Deck slot #{index}"))
                                .technical(format!(
                                    "Debuffed: {} · remaining: {} · effective suits: {}",
                                    card.debuffed, card.remaining, card.effective_suits
                                )),
                                crate::components::tooltip::TipAnchor::Left,
                            );
                                    entity.with_children(|art| {
                                        let face = crate::components::widgets::PlayingFace {
                                            rank: card.rank,
                                            suit: name,
                                            enhancement: card.enhancement.unwrap_or_default(),
                                            edition: card.edition,
                                            seal: card.seal.unwrap_or_default(),
                                            debuffed: card.debuffed,
                                            seed: index,
                                        };
                                        if !crate::components::widgets::playing_card_face(
                                            art, atlases, &face, width,
                                        ) {
                                            let rank = if card.rank == 'T' {
                                                "10".to_owned()
                                            } else {
                                                card.rank.to_string()
                                            };
                                            label(
                                                art,
                                                rank,
                                                30.0,
                                                if suit % 2 == 1 { RED } else { WELL },
                                            );
                                        }
                                    });
                                }
                            });
                    }
                });
            });
        tile_button(panel, "Back", ORANGE, (15.0, 0.66), 0.49, Click::Back);
    });
}

pub fn result(ui: &mut ChildSpawnerCommands, view: &RunView) {
    modal(ui, 590.0, 14.0, |panel| {
        let (heading, color) = match (view.phase, view.endless_mode) {
            (Phase::Won, true) => ("ENDLESS COMPLETE", GOLD),
            (Phase::Won, false) => ("YOU WIN!", GOLD),
            _ => ("GAME OVER", RED),
        };
        label(panel, heading, 52.0, color);
        if view.endless_mode && view.campaign_won {
            label(panel, "Ante 8 victory recorded", 24.0, GOLD);
        }
        if view.phase == Phase::Won && view.endless_mode {
            label(panel, "Next ante exceeds the score limit", 21.0, MUTED);
        }
        for (name, data) in [
            ("Best hand", score_text(view.stats.best_hand)),
            ("Round score", score_text(view.score)),
            ("Cards played", view.stats.played.to_string()),
            ("Cards discarded", view.stats.discarded.to_string()),
            ("Purchases", view.stats.purchased.to_string()),
            ("Rerolls", view.stats.rerolls.to_string()),
            ("Ante / Round", format!("{} / {}", view.ante, view.round)),
            ("Seed", view.seed.to_string()),
        ] {
            panel
                .spawn(box_node(
                    Node {
                        width: percent(100),
                        height: px(34),
                        padding: UiRect::horizontal(px(10)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    WELL,
                ))
                .with_children(|line| {
                    label(line, name, 21.0, MUTED);
                    label(line, data, 24.0, INK);
                });
        }
        row(panel, |actions| {
            tile_button(
                actions,
                "New Run",
                RED,
                (3.05, 0.82),
                0.49,
                Click::OpenDecks,
            );
            tile_button(
                actions,
                "Main Menu",
                RED,
                (3.05, 0.82),
                0.49,
                Click::MainMenu,
            );
        });
        if view.can_continue_endless {
            tile_button(
                panel,
                "Endless Mode",
                BLUE,
                (6.25, 0.94),
                0.55,
                Click::Play(Command::ContinueEndless),
            );
        }
    });
}

pub fn deck_preview(ui: &mut ChildSpawnerCommands, view: &RunView) {
    let mut counts = [[0usize; 13]; 4];
    let mut stones = 0;
    let mut effective = [0usize; 4];
    for card in view.deck_view.iter().filter(|card| card.remaining) {
        if card.enhancement == Some("Stone") {
            stones += 1;
        } else if let Some(rank) = RANKS.find(card.rank) {
            counts[card.suit][rank] += 1;
            for (suit, count) in effective.iter_mut().enumerate() {
                *count += usize::from(card.effective_suits & (1 << suit) != 0);
            }
        }
    }
    ui.spawn((
        box_node(
            Node {
                padding: UiRect::all(px(8)),
                align_items: AlignItems::Center,
                ..column(3.0)
            },
            WELL,
        ),
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        let totals = std::array::from_fn::<_, 13, _>(|rank| {
            counts.iter().map(|suit| suit[rank]).sum::<usize>()
        });
        let colors = [Color::srgb(0.28, 0.20, 0.48), RED, BLUE, ORANGE];
        for (suit, rank_counts) in std::iter::once((None, totals)).chain(
            counts
                .into_iter()
                .enumerate()
                .map(|(i, row)| (Some(i), row)),
        ) {
            panel
                .spawn((
                    Node {
                        column_gap: px(2),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    let title = suit.map_or_else(
                        || "Rank".into(),
                        |i| {
                            let base = rank_counts.iter().sum::<usize>();
                            if effective[i] == base {
                                format!("{} {base}", SUITS[i])
                            } else {
                                format!("{} {base} ({})", SUITS[i], effective[i])
                            }
                        },
                    );
                    row.spawn((
                        Node {
                            width: px(114),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|cell| label(cell, title, 17.0, INK));
                    for (index, count) in rank_counts.into_iter().enumerate() {
                        row.spawn((
                            box_node(
                                Node {
                                    width: px(33),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                suit.map_or(PANEL, |i| colors[i]),
                            ),
                            Pickable::IGNORE,
                        ))
                        .with_children(|cell| {
                            let text = if suit.is_none() {
                                let rank = RANKS.as_bytes()[index] as char;
                                format!(
                                    "{}\n{count}",
                                    if rank == 'T' {
                                        "10".into()
                                    } else {
                                        rank.to_string()
                                    }
                                )
                            } else {
                                count.to_string()
                            };
                            label(cell, text, 18.0, if count == 0 { MUTED } else { INK });
                        });
                    }
                });
        }
        if stones > 0 {
            label(panel, format!("Stone: {stones}"), 17.0, INK);
        }
    });
}

/// Deck view lines: only the attributes the card actually has, like the original.
fn deck_card_effect(card: &ante_core::contract::DeckCardView) -> String {
    let mut lines = Vec::new();
    if let Some(enhancement) = card.enhancement {
        lines.push(format!("Enhancement: {enhancement}"));
    }
    if !matches!(card.edition, "" | "Base" | "None") {
        lines.push(format!("Edition: {}", card.edition));
    }
    if let Some(seal) = card.seal {
        lines.push(format!("Seal: {seal}"));
    }
    if card.debuffed {
        lines.push("Debuffed".to_owned());
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests;
