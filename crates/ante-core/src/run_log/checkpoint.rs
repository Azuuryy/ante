use serde::{Deserialize, Serialize};

use crate::contract::{Phase, RunView};

/// The confirmed state after a command, without runtime ids, so two runs of the same
/// log compare equal. Cards read as rank, suit and their modifiers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub phase: String,
    pub ante: usize,
    pub round: usize,
    pub money: usize,
    pub score: usize,
    pub hands: usize,
    pub discards: usize,
    pub reward: usize,
    pub hand: Vec<String>,
    pub jokers: Vec<String>,
    pub consumables: Vec<String>,
    pub deck_cards: usize,
}

impl Checkpoint {
    pub fn of(view: &RunView) -> Self {
        Self {
            phase: phase_name(view.phase).to_owned(),
            ante: view.ante,
            round: view.round,
            money: view.money,
            score: view.score,
            hands: view.hands,
            discards: view.discards,
            reward: view.reward,
            hand: view
                .cards
                .iter()
                .map(|card| {
                    let mut label = format!("{}{}", card.rank, card.suit);
                    for part in [&card.enhancement, &card.edition, &card.seal] {
                        if !part.is_empty() && part != "Base" && part != "None" {
                            label.push(' ');
                            label.push_str(part);
                        }
                    }
                    if card.selected {
                        label.push_str(" *");
                    }
                    label
                })
                .collect(),
            jokers: view
                .held_jokers
                .iter()
                .map(|joker| {
                    format!("{} {}", joker.name, joker.edition)
                        .trim()
                        .to_owned()
                })
                .collect(),
            consumables: view.consumables.iter().map(|c| c.name.clone()).collect(),
            deck_cards: view.deck_cards,
        }
    }
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::BeforeBlind => "before_blind",
        Phase::Playing => "playing",
        Phase::Reward => "reward",
        Phase::Shop => "shop",
        Phase::Pack => "pack",
        Phase::Won => "won",
        Phase::Lost => "lost",
    }
}
