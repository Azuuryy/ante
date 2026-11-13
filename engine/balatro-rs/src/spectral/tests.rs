use super::*;
use crate::card::{Card, Enhancement, Suit, Value};
use crate::joker::{jokers_by_rarity, mint_joker_id, Jokers};
use crate::stage::{Blind, Stage};
use std::collections::HashSet;

fn game_in_blind() -> Game {
    Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    }
}

fn minted_joker(rarity: Rarity, index: usize) -> Jokers {
    let mut j = jokers_by_rarity(rarity)[index].clone();
    j.set_instance_id(mint_joker_id());
    j
}

mod joker_spectrals;
mod seals_and_cards;
