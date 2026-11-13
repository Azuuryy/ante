// Debug entry points used by Ante's dev panel.

use crate::action::Action;
use crate::game::Game;
use crate::stage::{Blind, Stage};

fn blind_game() -> Game {
    let mut g = Game::default();
    g.start();
    g.handle_action(Action::SelectBlind(Blind::Small)).unwrap();
    g
}

#[test]
fn debug_add_score_clears_the_blind_like_a_winning_hand() {
    let mut g = blind_game();
    let missing = g.required_score() - g.score;
    g.debug_add_score(missing).unwrap();
    assert_eq!(g.stage, Stage::PostBlind());
    assert!(g.reward > 0);
    assert_eq!(g.available.cards().len(), 0);
}

#[test]
fn debug_add_score_below_target_keeps_the_blind() {
    let mut g = blind_game();
    g.debug_add_score(1).unwrap();
    assert!(g.stage.is_blind());
}

#[test]
fn debug_add_score_outside_a_blind_is_rejected() {
    let mut g = Game::default();
    g.start();
    assert!(g.debug_add_score(1).is_err());
}

#[test]
fn debug_next_ante_advances_one_ante() {
    let mut g = Game::default();
    g.start();
    let before = g.ante_current.number();
    g.debug_next_ante().unwrap();
    assert_eq!(g.ante_current.number(), before + 1);
}
