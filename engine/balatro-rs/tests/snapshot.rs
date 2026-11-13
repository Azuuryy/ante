use balatro_rs::{
    action::Action,
    config::{Config, RngMode},
    game::Game,
    stage::Blind,
};

fn started(seed: u64) -> Game {
    let mut game = Game::new(Config {
        seed: Some(seed),
        rng_mode: RngMode::Fast,
        ..Config::default()
    });
    game.handle_action(Action::SelectBlind(Blind::Small))
        .unwrap();
    game
}

#[test]
fn a_loaded_snapshot_deals_the_same_cards_after_a_discard() {
    let mut saved = started(12345);
    let mut loaded = Game::from_json(&saved.to_json().unwrap()).unwrap();
    assert_eq!(
        saved.available.not_selected(),
        loaded.available.not_selected()
    );
    let card = saved.available.not_selected()[0];
    for game in [&mut saved, &mut loaded] {
        game.handle_action(Action::SelectCard(card)).unwrap();
        game.handle_action(Action::Discard()).unwrap();
    }
    assert_eq!(
        saved.available.not_selected(),
        loaded.available.not_selected()
    );
}
