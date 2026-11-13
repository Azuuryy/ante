use super::*;

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
