//! Jokers that create cards or level up hands (Lot 3, batch 5a): the Tarot, Spectral,
//! Joker and Stone card they make, and the moments they act on that the scoring pass
//! does not cover: the blind set, a Booster Pack opened, the first discard, the hand's
//! `before` and `after`.

use super::*;
use crate::event::JokerNote;
use crate::spectral::Spectral;

impl Game {
    /// Room in the consumable slots (`#G.consumeables.cards + buffer < limit`).
    pub(crate) fn has_consumable_room(&self) -> bool {
        self.consumables.len() < self.consumable_slots()
    }

    /// A random Tarot card in the consumable slots, said "+1 Tarot" on Joker `id`.
    pub(crate) fn create_tarot(&mut self, id: usize) {
        let held: Vec<Tarot> = self
            .consumables
            .iter()
            .filter_map(|c| match c {
                Consumable::Tarot(t) => Some(*t),
                _ => None,
            })
            .collect();
        let tarot = self.backend.roll_random_tarot(&held);
        self.consumables.push(tarot);
        self.joker_message(id, JokerNote::PlusTarot);
    }

    /// A random Spectral card (not The Soul or Black Hole), said "+1 Spectral".
    pub(crate) fn create_spectral(&mut self, id: usize) {
        let pool: Vec<Spectral> = Spectral::iter().filter(|s| !s.is_rare()).collect();
        let spectral = pool[self.backend.roll_index(pool.len())];
        self.consumables.push(Consumable::Spectral(spectral));
        self.joker_message(id, JokerNote::PlusSpectral);
    }

    /// Diet Cola sold: a Double Tag.
    pub(crate) fn acquire_double_tag(&mut self) {
        self.acquire_tag(Tag::Double, None);
    }

    /// `setting_blind` for the card makers: Cartomancer a Tarot card, Riff-raff up to two
    /// Common Jokers, Marble Joker a Stone card in the deck.
    pub(super) fn card_makers_set_blind(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::Cartomancer(_))) {
            if self.has_consumable_room() {
                self.create_tarot(id);
            }
        }
        for id in self.joker_ids(|j| matches!(j, Jokers::RiffRaff(_))) {
            let room = self.joker_slots().saturating_sub(self.jokers.len()).min(2);
            if room == 0 {
                continue;
            }
            for _ in 0..room {
                let mut joker = self.backend.gen_joker_of_rarity(
                    self.ante_current.number() as i32,
                    self.prob_mult,
                    &self.jokers,
                    Rarity::Common,
                );
                joker.set_edition(Edition::Base);
                self.add_acquired_joker(joker);
            }
            self.joker_message(id, JokerNote::PlusJoker);
        }
        for id in self.joker_ids(|j| matches!(j, Jokers::MarbleJoker(_))) {
            let mut stone = self.backend.gen_random_card(self.prob_mult, false, None);
            stone.enhancement = Some(Enhancement::Stone);
            self.deck.push(stone);
            self.playing_cards_added(1);
            self.joker_message(id, JokerNote::PlusStone);
        }
    }

    /// `open_booster`: Hallucination has 1 in 2 to create a Tarot card.
    pub(super) fn card_makers_open_booster(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::Hallucination(_))) {
            if self.has_consumable_room() && self.prob_roll(1, 2) {
                self.create_tarot(id);
            }
        }
    }

    /// `pre_discard`: on the round's first discard, Burnt Joker levels up the discarded
    /// poker hand.
    pub(super) fn card_makers_discard(&mut self, discarded: &[Card], first: bool) {
        if !first || discarded.is_empty() {
            return;
        }
        let burnt = self.joker_ids(|j| matches!(j, Jokers::BurntJoker(_)));
        if burnt.is_empty() {
            return;
        }
        let Ok(made) = SelectHand::new(discarded.to_vec()).best_hand_with(self.hand_rules()) else {
            return;
        };
        for id in burnt {
            self.planetarium.level_up(made.rank);
            self.joker_message(id, JokerNote::UpgradePlain);
        }
    }

    /// `before`: Space Joker has 1 in 4 to level up the hand played.
    pub(super) fn space_jokers(&mut self, rank: HandRank) {
        for id in self.joker_ids(|j| matches!(j, Jokers::SpaceJoker(_))) {
            if self.prob_roll(1, 4) {
                self.planetarium.level_up(rank);
                self.joker_message(id, JokerNote::LevelUp);
            }
        }
    }

    /// `after`: Loyalty Card counts the hands played since it was acquired.
    pub(super) fn count_loyalty(&mut self) {
        for id in self.joker_ids(|j| matches!(j, Jokers::LoyaltyCard(_))) {
            if let Some(state) = self.joker_state_mut(id) {
                state.counter += 1.0;
            }
        }
    }

    fn joker_ids(&self, kind: fn(&Jokers) -> bool) -> Vec<usize> {
        self.jokers
            .iter()
            .filter(|j| kind(j))
            .map(|j| j.instance_id())
            .collect()
    }
}
