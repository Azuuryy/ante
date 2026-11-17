//! Joker effects tied to moments outside the scoring pass: the blind being set
//! (`context.setting_blind`), a card sold (`selling_card`), playing cards added
//! (`playing_card_added`), the hand played (`after`), and Vampire's `before`.

use super::*;
use crate::event::JokerNote;

impl Game {
    /// Removes a held Joker for good (sliced, eaten, extinct), giving back the slot a
    /// Negative edition added.
    pub(crate) fn destroy_joker(&mut self, id: usize) {
        let Some(index) = self.jokers.iter().position(|j| j.instance_id() == id) else {
            return;
        };
        let removed = self.jokers.remove(index);
        self.shop.astronomer = self
            .jokers
            .iter()
            .any(|j| matches!(j, Jokers::Astronomer(_)));
        if matches!(removed, Jokers::ChaosTheClown(_)) {
            self.chaos_changed(false);
        }
        if removed.edition() == Edition::Negative {
            self.config.joker_slots = self.config.joker_slots.saturating_sub(1);
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
    }

    /// `setting_blind`: Ceremonial Dagger slices the Joker to its right (not Eternal) and
    /// gains twice its sell value as Mult; Madness, on a Small or Big Blind, gains X0.5
    /// and destroys a random other Joker that is not Eternal.
    pub(crate) fn jokers_set_blind(&mut self, blind: Blind) {
        self.card_makers_set_blind();
        // Burglar: +3 hands and no discards this round.
        for id in self.held_ids(|j| matches!(j, Jokers::Burglar(_))) {
            self.plays += 3;
            self.discards_remaining = 0;
            self.joker_message(id, JokerNote::HandsPlus(3));
        }
        for id in self.held_ids(|j| matches!(j, Jokers::CeremonialDagger(_))) {
            let Some(position) = self.jokers.iter().position(|j| j.instance_id() == id) else {
                continue;
            };
            let Some(right) = self.jokers.get(position + 1).cloned() else {
                continue;
            };
            if right.stickers().eternal {
                continue;
            }
            let gain = 2 * self.joker_sell_value(&right);
            self.destroy_joker(right.instance_id());
            if let Some(state) = self.joker_state_mut(id) {
                state.counter += gain as f32;
                let total = state.counter as usize;
                self.joker_message(id, JokerNote::MultPlus(total));
            }
        }
        if blind == Blind::Boss {
            return;
        }
        for id in self.held_ids(|j| matches!(j, Jokers::Madness(_))) {
            if !self.jokers.iter().any(|j| j.instance_id() == id) {
                continue;
            }
            let Some(state) = self.joker_state_mut(id) else {
                continue;
            };
            state.counter += 0.5;
            let factor = 100 + (state.counter * 100.0).round() as usize;
            let victims: Vec<Jokers> = self
                .jokers
                .iter()
                .filter(|j| j.instance_id() != id && !j.stickers().eternal)
                .cloned()
                .collect();
            if !victims.is_empty() {
                let victim = self.backend.pick_random_joker(victims);
                self.destroy_joker(victim.instance_id());
            }
            self.joker_message(id, JokerNote::XMult(factor));
        }
    }

    /// `selling_card`: Campfire gains X0.25 per card sold.
    pub(super) fn jokers_card_sold(&mut self) {
        self.grow_jokers(
            |j| matches!(j, Jokers::Campfire(_)),
            0.25,
            |_| JokerNote::UpgradePlain,
        );
    }

    /// `playing_card_added`: Hologram gains X0.25 per playing card added to the deck.
    pub(crate) fn playing_cards_added(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.grow_jokers(
            |j| matches!(j, Jokers::Hologram(_)),
            0.25 * count as f32,
            |value| JokerNote::XMult(100 + (value * 100.0).round() as usize),
        );
    }

    /// `after` the hand: each Seltzer counts down from 10 and is drunk at the last hand.
    pub(crate) fn wear_seltzer(&mut self) {
        self.count_loyalty();
        for id in self.held_ids(|j| matches!(j, Jokers::Seltzer(_))) {
            let used = self.joker_state_mut(id).map_or(0.0, |state| state.counter) as usize;
            let left = 10usize.saturating_sub(used);
            if left <= 1 {
                self.joker_message(id, JokerNote::Drank);
                self.destroy_joker(id);
            } else {
                if let Some(state) = self.joker_state_mut(id) {
                    state.counter += 1.0;
                }
                self.joker_message(id, JokerNote::Remaining(left - 1));
            }
        }
    }

    /// `end_of_round`: Egg gains $3 of sell value ("Value Up!"); after a Boss Blind,
    /// Rocket's payout grows by $2 ("Upgrade!").
    pub(super) fn jokers_end_of_round(&mut self) {
        for id in self.held_ids(|j| matches!(j, Jokers::Egg(_))) {
            if let Some(state) = self.joker_state_mut(id) {
                state.extra_value += 3;
                self.joker_message(id, JokerNote::ValueUp);
            }
        }
        if self.blind == Some(Blind::Boss) {
            self.grow_jokers(
                |j| matches!(j, Jokers::Rocket(_)),
                2.0,
                |_| JokerNote::UpgradeMoney,
            );
        }
    }

    /// End of a Boss Blind: Campfire goes back to X1 ("Reset") when it had grown.
    pub(super) fn reset_campfires(&mut self) {
        if self.blind != Some(Blind::Boss) {
            return;
        }
        for id in self.held_ids(|j| matches!(j, Jokers::Campfire(_))) {
            if let Some(state) = self.joker_state_mut(id) {
                if state.counter > 0.0 {
                    state.counter = 0.0;
                    self.joker_message(id, JokerNote::Reset);
                }
            }
        }
    }

    /// Vampire's `before`: each scoring card with an enhancement, not debuffed, loses it
    /// before the hand scores, and Vampire gains X0.1 per card.
    pub(super) fn jokers_before_scoring(&mut self, hand: &mut MadeHand) {
        self.space_jokers(hand.rank);
        self.dna(hand);
        let vampires = self.held_ids(|j| matches!(j, Jokers::Vampire(_)));
        if vampires.is_empty() {
            return;
        }
        let drained: Vec<usize> = hand
            .hand
            .cards()
            .iter()
            .filter(|c| c.enhancement.is_some() && !self.is_card_debuffed(c))
            .map(|c| c.id)
            .collect();
        if drained.is_empty() {
            return;
        }
        for id in &drained {
            self.mutate_card(*id, |c| c.enhancement = None);
        }
        let strip = |cards: Vec<Card>| -> Vec<Card> {
            cards
                .into_iter()
                .map(|mut c| {
                    if drained.contains(&c.id) {
                        c.enhancement = None;
                    }
                    c
                })
                .collect()
        };
        hand.hand = SelectHand::new(strip(hand.hand.cards()));
        hand.all = strip(hand.all.clone());
        for id in vampires {
            if let Some(state) = self.joker_state_mut(id) {
                state.counter += 0.1 * drained.len() as f32;
            }
        }
    }

    /// Mr. Bones on a lost round: with at least a quarter of the blind scored, the run
    /// goes on and Mr. Bones is destroyed ("Saved!").
    pub(super) fn mr_bones_saves(&mut self, required: usize) -> bool {
        if self.score.saturating_mul(4) < required {
            return false;
        }
        let Some(id) = self
            .held_ids(|j| matches!(j, Jokers::MrBones(_)))
            .first()
            .copied()
        else {
            return false;
        };
        self.joker_message(id, JokerNote::Saved);
        self.destroy_joker(id);
        true
    }

    /// A Joker joining or leaving the held ones outside the shop's cash out: Chaos the
    /// Clown's free reroll comes and goes with it (`add_to_deck`, `remove_from_deck`).
    pub(crate) fn chaos_changed(&mut self, joined: bool) {
        if joined {
            self.free_rerolls += 1;
        } else {
            self.free_rerolls = self.free_rerolls.saturating_sub(1);
        }
    }

    fn held_ids(&self, kind: fn(&Jokers) -> bool) -> Vec<usize> {
        self.jokers
            .iter()
            .filter(|j| kind(j))
            .map(|j| j.instance_id())
            .collect()
    }
}
