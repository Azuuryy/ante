use crate::card::{Card, Enhancement, Suit, Value};
use crate::hand::SelectHand;
use crate::stage::{Blind, Stage};
use balatro_types::BossBlind;

use super::*;

fn score_before_after_joker(joker: Jokers, hand: SelectHand, before: usize, after: usize) {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };

    // First score without joker
    let score = g.calc_score(hand.best_hand().unwrap());
    assert_eq!(score, before);

    // Buy (and apply) the joker
    g.money += 1000; // Give adequate money to buy
    g.stage = Stage::Shop();
    g.shop.jokers.push(joker.clone());
    g.buy_joker(joker).unwrap();
    g.stage = Stage::Blind(Blind::Small);
    // Second score with joker applied
    let score = g.calc_score(hand.best_hand().unwrap());
    assert_eq!(score, after);
}

mod boss_interactions;
mod card_parity_jokers;
mod catalog;
mod conditional_jokers;
mod discards_and_debuffs;
mod face_card_jokers;
mod hand_type_jokers;
mod held_card_jokers;
mod scaling_jokers;
mod wild_and_retriggers;
