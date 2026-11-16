use crate::card::{Enhancement, Suit};
use crate::error::GameError;
use crate::game::Game;
use crate::rng::RngBackend;
use rand::Rng;
use strum::IntoEnumIterator;

pub use balatro_types::Tarot;

/// Engine behavior for `Tarot`
pub trait TarotEffect {
    fn apply(&self, game: &mut Game) -> Result<(), GameError>;
}

impl TarotEffect for Tarot {
    fn apply(&self, game: &mut Game) -> Result<(), GameError> {
        match self {
            Self::Magician => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Lucky));
                }
            }
            Self::Empress => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Mult));
                }
            }
            Self::Hierophant => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Bonus));
                }
            }
            Self::Lovers => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Wild));
                }
            }
            Self::Chariot => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Steel));
                }
            }
            Self::Justice => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Glass));
                }
            }
            Self::Devil => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Gold));
                }
            }
            Self::Tower => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.enhancement = Some(Enhancement::Stone));
                }
            }
            Self::Star => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Diamond);
                }
            }
            Self::Moon => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Club);
                }
            }
            Self::Sun => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Heart);
                }
            }
            Self::World => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.suit = Suit::Spade);
                }
            }
            Self::Strength => {
                let selected = game.available.selected();
                for card in selected {
                    game.mutate_card(card.id, |c| c.value = c.value.next());
                }
            }
            Self::HangedMan => {
                let selected = game.available.selected();
                for card in selected {
                    game.destroy_card(card.id);
                }
            }
            Self::Death => {
                let selected = game.available.selected();
                if selected.len() < 2 {
                    return Err(GameError::InvalidAction);
                }
                let left_id = selected[0].id;
                let right = selected[1];
                game.mutate_card(left_id, |c| {
                    c.value = right.value;
                    c.suit = right.suit;
                    c.enhancement = right.enhancement;
                    c.edition = right.edition;
                    c.seal = right.seal;
                });
            }
            Self::Hermit => {
                let gain = game.money.min(20);
                game.money += gain;
            }
            Self::Temperance => {
                let total: usize = game.jokers.iter().map(|j| game.joker_sell_value(j)).sum();
                game.money += total.min(50);
            }
            Self::WheelOfFortune => {
                if !game.jokers.is_empty() && game.prob_roll(1, 4) {
                    let picked = game.backend.pick_random_joker(game.jokers.clone());
                    let edition = game.backend.roll_random_edition();
                    if let Some(j) = game
                        .jokers
                        .iter_mut()
                        .find(|j| j.instance_id() == picked.instance_id())
                    {
                        j.set_edition(edition);
                    }
                }
            }
            Self::HighPriestess => {
                let slots = game.consumable_slots();
                for _ in 0..2 {
                    if game.consumables.len() >= slots {
                        break;
                    }
                    let planetarium = game.planetarium.clone();
                    let planet = game.backend.roll_random_planet(&planetarium, &[]);
                    game.consumables.push(planet);
                }
            }
            Self::Emperor => {
                let slots = game.consumable_slots();
                let mut excl: Vec<Tarot> = game
                    .consumables
                    .iter()
                    .filter_map(|c| {
                        if let crate::consumable::Consumable::Tarot(t) = c {
                            Some(*t)
                        } else {
                            None
                        }
                    })
                    .collect();
                for _ in 0..2 {
                    if game.consumables.len() >= slots {
                        break;
                    }
                    let tarot = game.backend.roll_random_tarot(&excl);
                    if let crate::consumable::Consumable::Tarot(t) = &tarot {
                        excl.push(*t);
                    }
                    game.consumables.push(tarot);
                }
            }
            Self::Judgement => {
                if game.jokers.len() < game.joker_slots() {
                    let prob_mult = game.prob_mult;
                    let exclude = game.jokers.clone();
                    let ante = game.ante_current.number() as i32;
                    let joker = game.backend.gen_joker(ante, prob_mult, &exclude);
                    game.jokers.push(joker);
                }
            }
            Self::Fool => {
                if let Some(last) = game.last_consumable_used {
                    if game.consumables.len() < game.consumable_slots() {
                        game.consumables.push(last);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Picks a uniformly random Tarot
pub fn random_tarot(rng: &mut impl Rng) -> Tarot {
    let all: Vec<Tarot> = Tarot::iter().collect();
    let i = rng.gen_range(0..all.len());
    all[i]
}

#[cfg(test)]
mod tests;
