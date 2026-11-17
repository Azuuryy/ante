//! What a held consumable carries besides its kind: sell value gained (Gift Card) and the
//! Negative edition (Perkeo's copies, one more slot each). `Game.consumable_meta` follows
//! `Game.consumables` by position; a missing entry is a plain card.

use super::*;
use crate::event::JokerNote;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConsumableMeta {
    /// `ability.extra_value`.
    pub extra_value: usize,
    /// A Negative edition, which adds a consumable slot while held.
    pub negative: bool,
}

impl Game {
    /// The state of held consumable `index`.
    pub fn consumable_meta(&self, index: usize) -> ConsumableMeta {
        self.consumable_meta.get(index).copied().unwrap_or_default()
    }

    fn meta_mut(&mut self, index: usize) -> &mut ConsumableMeta {
        if self.consumable_meta.len() <= index {
            self.consumable_meta
                .resize(index + 1, ConsumableMeta::default());
        }
        &mut self.consumable_meta[index]
    }

    /// Takes held consumable `index` out, with its state.
    pub(crate) fn remove_consumable_at(&mut self, index: usize) -> Consumable {
        if index < self.consumable_meta.len() {
            self.consumable_meta.remove(index);
        }
        self.consumables.remove(index)
    }

    /// Moves a held consumable, with its state.
    pub fn move_consumable(&mut self, from: usize, to: usize) {
        if from >= self.consumables.len() || to >= self.consumables.len() {
            return;
        }
        self.meta_mut(self.consumables.len() - 1);
        let card = self.consumables.remove(from);
        let meta = self.consumable_meta.remove(from);
        self.consumables.insert(to, card);
        self.consumable_meta.insert(to, meta);
    }

    /// Negative consumables held, each worth a slot.
    pub(crate) fn negative_consumables(&self) -> usize {
        (0..self.consumables.len())
            .filter(|&index| self.consumable_meta(index).negative)
            .count()
    }

    /// What selling held consumable `index` pays: half its cost (+$5 when Negative), at
    /// least $1, plus the value it gained.
    pub fn consumable_sell_value(&self, index: usize) -> usize {
        let Some(card) = self.consumables.get(index) else {
            return 0;
        };
        let meta = self.consumable_meta(index);
        let cost = card.cost() + if meta.negative { 5 } else { 0 };
        (cost / 2).max(1) + meta.extra_value
    }

    /// End of round: each Gift Card adds $1 of sell value to every Joker and consumable
    /// ("Value Up!").
    pub(super) fn gift_cards(&mut self) {
        let gifts: Vec<usize> = self
            .jokers
            .iter()
            .filter(|j| !j.is_disabled() && matches!(j, Jokers::GiftCard(_)))
            .map(|j| j.instance_id())
            .collect();
        for id in gifts {
            for joker in &mut self.jokers {
                let mut state = joker.state();
                state.extra_value += 1;
                joker.set_state(state);
            }
            for index in 0..self.consumables.len() {
                self.meta_mut(index).extra_value += 1;
            }
            self.joker_message(id, JokerNote::ValueUp);
        }
    }

    /// `ending_shop`: each Perkeo adds a Negative copy of a random held consumable.
    pub(crate) fn perkeo(&mut self) {
        let perkeos: Vec<usize> = self
            .jokers
            .iter()
            .filter(|j| !j.is_disabled() && matches!(j, Jokers::Perkeo(_)))
            .map(|j| j.instance_id())
            .collect();
        for id in perkeos {
            if self.consumables.is_empty() {
                continue;
            }
            let index = self.backend.roll_index(self.consumables.len());
            let copy = self.consumables[index];
            self.consumables.push(copy);
            let last = self.consumables.len() - 1;
            *self.meta_mut(last) = ConsumableMeta {
                extra_value: 0,
                negative: true,
            };
            self.joker_message(id, JokerNote::Duplicated);
        }
    }
}
