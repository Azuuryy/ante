use super::*;

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
