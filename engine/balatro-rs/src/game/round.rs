use super::*;
use crate::card::Value;
use crate::event::{GameEvent, JokerNote};
use crate::planet::Planets;
use balatro_types::joker::SelectorValue;

impl Game {
    // Fires when the round that just ended cleared the blind, before played
    // cards are swapped out and replacements drawn in. `self.available` still
    // reflects what was actually held in hand at the moment the round ended.
    pub(crate) fn handle_round_end(&mut self) {
        for e in self.effect_registry.on_round_end.clone() {
            if let Effects::OnRoundEnd(f) = e {
                f.lock().unwrap()(self)
            }
        }
        self.roll_bananas();
        self.wear_out_jokers();
        self.reset_campfires();
        self.jokers_end_of_round();
        self.age_invisible();
        self.stickers_end_of_round();
        self.gift_cards();
        // Hit the Road goes back to X1 with the round's discards: "Reset" when it grew.
        let jacks = self
            .non_debuffed(self.discarded_this_round.iter())
            .iter()
            .any(|c| c.value == crate::card::Value::Jack);
        if jacks {
            let roads: Vec<usize> = self
                .jokers
                .iter()
                .filter(|j| matches!(j, Jokers::HitTheRoad(_)))
                .map(|j| j.instance_id())
                .collect();
            for id in roads {
                self.joker_message(id, JokerNote::Reset);
            }
        }

        // Then each held card's `get_end_of_round_effect`, repeated by a Red Seal or Mime
        // when it has one: a Gold card pays $3, a Blue Seal makes the Planet of the last
        // hand played if there is room. A debuffed card has no effect.
        for card in self.available.not_selected() {
            let gold = card.enhancement == Some(Enhancement::Gold);
            let blue = card.seal == Some(Seal::Blue);
            if !(gold || blue) || self.is_card_debuffed(&card) {
                continue;
            }
            for _ in 0..self.trigger_count_held(&card) {
                if gold {
                    self.money += 3;
                    self.emit(GameEvent::CardPaid { card, amount: 3 });
                }
                if blue && self.consumables.len() < self.consumable_slots() {
                    if let Some(planet) = self.last_hand_planet() {
                        let consumable = Consumable::Planet(planet);
                        self.consumables.push(consumable);
                        self.emit(GameEvent::SealCreated { card, consumable });
                    }
                }
            }
        }
    }

    /// The Planet of `last_hand_played`, as the Blue Seal looks it up.
    fn last_hand_planet(&self) -> Option<Planets> {
        let rank = self.last_hand_played?;
        Planets::iter().find(|planet| planet.hand_rank() == rank)
    }

    pub(crate) fn clear_blind(&mut self) {
        self.temporary_hand_size = 0;
        self.score = self.config.base_score;
        self.plays = self.plays_per_round();
        self.discards_remaining = self.discards_per_round();
        self.hand_ranks_played_this_round.clear();
        self.mouth_hand = None;
        self.discarded_this_round.clear();
        self.boss_disabled_by_luchador = false;
        self.deck.append(&mut self.discarded);
        self.deck.extend(self.available.cards());
        self.available.empty();
        self.backend.shuffle_deck(&mut self.deck);
        self.roll_discard_selectors();
    }

    /// The boss currently in effect, or `None` if we're not in the middle of
    /// that boss's own round, or its effect is disabled (Chicot held, or
    /// Luchador sold this Boss Blind).
    pub(crate) fn active_boss(&self) -> Option<BossBlind> {
        if self.blind != Some(Blind::Boss) {
            return None;
        }
        if self.jokers.iter().any(|j| matches!(j, Jokers::Chicot(_))) {
            return None;
        }
        if self.boss_disabled_by_luchador {
            return None;
        }
        self.current_boss
    }

    /// Discards actually usable right now
    pub fn discards(&self) -> usize {
        if self.active_boss() == Some(BossBlind::Water) {
            0
        } else {
            self.discards_remaining
        }
    }

    // Castle/MailInRebate lock onto a fresh random suit/rank each round
    // ("changes every round") - unconditional, overwrites whatever was
    // already there.
    pub(super) fn roll_discard_selectors(&mut self) {
        for j in self.jokers.iter_mut() {
            self.backend.roll_discard_selector(j);
        }
        self.roll_card_targets(true);
    }

    /// The Idol's card (`reset_idol_card`: a random non-Stone card of the full deck) and the
    /// Ancient Joker's suit (`reset_ancient_card`: a suit other than the last). `every`
    /// rolls them all, as a new round starts; otherwise only those not rolled yet, for a
    /// Joker bought in the shop before its first round.
    pub(crate) fn roll_card_targets(&mut self, every: bool) {
        let deck: Vec<Card> = self
            .full_deck()
            .into_iter()
            .filter(|card| card.enhancement != Some(Enhancement::Stone))
            .collect();
        for index in 0..self.jokers.len() {
            let current = self.jokers[index].state().selector;
            if !every && current.is_some() {
                continue;
            }
            let target = match self.jokers[index] {
                Jokers::TheIdol(_) if !deck.is_empty() => {
                    let card = self.backend.pick_random_card(deck.clone());
                    SelectorValue::Card(card.value, card.suit)
                }
                Jokers::TheIdol(_) => SelectorValue::Card(Value::Ace, Suit::Spade),
                Jokers::ToDoList(_) => {
                    // `to_do_poker_hand`: a visible hand other than the current one.
                    let current = match current {
                        Some(SelectorValue::Hand(rank)) => Some(rank),
                        _ => None,
                    };
                    let hands: Vec<HandRank> = HandRank::iter()
                        .filter(|rank| *rank != HandRank::RoyalFlush && Some(*rank) != current)
                        .filter(|rank| {
                            !matches!(
                                rank,
                                HandRank::FiveOfAKind | HandRank::FlushHouse | HandRank::FlushFive
                            ) || self.planetarium.level(*rank).plays > 0
                        })
                        .collect();
                    let pick = hands[self.backend.roll_index(hands.len())];
                    SelectorValue::Hand(pick)
                }
                Jokers::AncientJoker(_) => {
                    let last = match current {
                        Some(SelectorValue::Suit(suit)) => Some(suit),
                        _ => None,
                    };
                    let suits: Vec<Suit> = [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond]
                        .into_iter()
                        .filter(|suit| Some(*suit) != last)
                        .collect();
                    let pick = self.backend.pick_random_card(
                        suits
                            .iter()
                            .map(|suit| Card::new(Value::Two, *suit))
                            .collect(),
                    );
                    SelectorValue::Suit(pick.suit)
                }
                _ => continue,
            };
            self.jokers[index].state_mut().selector = Some(target);
        }
    }

    /// End of round for food Jokers: Popcorn loses 4 Mult and Turtle Bean 1 hand size;
    /// at nothing left they are eaten.
    fn wear_out_jokers(&mut self) {
        let foods: Vec<(usize, bool, f32)> = self
            .jokers
            .iter()
            .filter_map(|joker| match joker {
                Jokers::Popcorn(_) => Some((joker.instance_id(), true, joker.state().counter)),
                Jokers::TurtleBean(_) => Some((joker.instance_id(), false, joker.state().counter)),
                _ => None,
            })
            .collect();
        for (id, popcorn, worn) in foods {
            let (step, start) = if popcorn { (4.0, 20.0) } else { (1.0, 5.0) };
            if start - worn - step <= 0.0 {
                self.joker_message(id, JokerNote::Eaten);
                self.destroy_joker(id);
                continue;
            }
            if let Some(state) = self.joker_state_mut(id) {
                state.counter += step;
            }
            let note = if popcorn {
                JokerNote::MultMinus(4)
            } else {
                JokerNote::HandSizeMinus(1)
            };
            self.joker_message(id, note);
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
    }

    /// Gros Michel (1 in 6) and Cavendish (1 in 1000) may go extinct at the end of the
    /// round; Gros Michel's extinction lets Cavendish into the pool.
    fn roll_bananas(&mut self) {
        let bananas: Vec<(usize, u32, bool)> = self
            .jokers
            .iter()
            .filter_map(|joker| match joker {
                Jokers::GrosMichel(_) => Some((joker.instance_id(), 6, true)),
                Jokers::Cavendish(_) => Some((joker.instance_id(), 1000, false)),
                _ => None,
            })
            .collect();
        for (id, odds, gros_michel) in bananas {
            if self.prob_roll(1, odds) {
                self.joker_message(id, JokerNote::Extinct);
                self.destroy_joker(id);
                if gros_michel {
                    self.gros_michel_extinct = true;
                    self.backend.set_gros_michel_extinct(true);
                }
            } else {
                self.joker_message(id, JokerNote::Safe);
            }
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
    }

    // draw from deck to available
    pub(super) fn draw(&mut self, count: usize) {
        self.available.extend(self.deck.draw(count));
    }

    // shuffle and deal new cards to available
    pub(crate) fn deal(&mut self) {
        // add discarded back to deck, emptying in process
        self.deck.append(&mut self.discarded);
        // add available back to deck and empty
        self.deck.extend(self.available.cards());
        self.available.empty();
        self.backend.shuffle_deck(&mut self.deck);
        self.draw(self.hand_size());
    }

    /// Reshuffles discarded/held cards back into the deck and redraws a
    /// fresh hand, regardless of stage. Just a helper.
    pub fn redeal(&mut self) {
        self.deal();
    }

    pub(crate) fn select_card(&mut self, card: Card) -> Result<(), GameError> {
        if self.available.selected().len() >= self.config.selected_max {
            return Err(GameError::InvalidSelectCard);
        }
        self.available.select_card(card)
    }

    pub(crate) fn move_card(
        &mut self,
        direction: MoveDirection,
        card: Card,
    ) -> Result<(), GameError> {
        self.available.move_card(direction, card)
    }

    #[cfg(test)]
    pub(crate) fn play_selected(&mut self) -> Result<(), GameError> {
        self.play_selected_inner(None)
    }

    pub(super) fn play_selected_inner(
        &mut self,
        trace: Option<&mut TraceRecorder>,
    ) -> Result<(), GameError> {
        if self.plays == 0 {
            return Err(GameError::NoRemainingPlays);
        }
        self.plays -= 1;
        let selected = SelectHand::new(self.available.selected());
        let first_hand = self.hand_ranks_played_this_round.is_empty();
        let best = selected.best_hand_with(self.hand_rules())?;
        let scored = best.clone();
        self.press_play(&best);
        let score = self.calc_score_inner(best, trace);
        let clear_blind = self.handle_score(score)?;
        // must run before the discard/redraw below: self.available still reflects
        // the cards actually held when the round ended, not their replacements.
        if clear_blind {
            self.handle_round_end();
        }
        #[cfg(feature = "ante-extra-jokers")]
        self.age_ice_cream();
        self.discarded.extend(self.available.selected());
        self.available.remove_selected();
        // The hand refills to its size: the cards played, those The Hook discarded and
        // any lost earlier to a Joker or consumable are replaced (`draw_from_deck_to_hand`).
        let n = if self.active_boss() == Some(BossBlind::Serpent) {
            3
        } else {
            self.refill_count()
        };
        self.draw(n);
        let mut shattered = 0;
        for card in scored.hand.cards() {
            if card.enhancement == Some(Enhancement::Glass) && self.prob_roll(1, 4) {
                self.destroy_card(card.id);
                shattered += 1;
            }
        }
        self.wear_seltzer();
        self.sixth_sense(&scored.all, first_hand);
        // Glass Joker: X0.75 more per Glass card shattered.
        if shattered > 0 {
            self.grow_jokers(
                |j| matches!(j, Jokers::GlassJoker(_)),
                0.75 * shattered as f32,
                |value| JokerNote::XMult(100 + (value * 100.0).round() as usize),
            );
        }
        if clear_blind {
            self.clear_blind();
        }
        Ok(())
    }

    // discard selected cards from available and draw equal number back to available
    pub(crate) fn discard_selected(&mut self) -> Result<(), GameError> {
        if self.discards() == 0 {
            return Err(GameError::NoRemainingDiscards);
        }
        // `pre_discard`: the round's first discard, The Hook's not counted.
        let first = self.discards_used_this_round() == 0;
        self.discards_remaining -= 1;
        let discarded = self.available.selected();
        self.card_makers_discard(&discarded, first);
        self.available.remove_selected();
        let n = if self.active_boss() == Some(BossBlind::Serpent) {
            3
        } else {
            self.refill_count()
        };
        self.draw(n);
        self.record_discard(discarded);
        Ok(())
    }

    /// Cards that refill the hand to `hand_size()` after cards left it.
    fn refill_count(&self) -> usize {
        self.hand_size()
            .saturating_sub(self.available.cards().len())
    }

    /// The discard of `discarded` cards already out of the hand: they go to the discard
    /// pile, Purple seals create a Tarot and discard Jokers act, like the `discard`
    /// contexts of `G.FUNCS.discard_cards_from_highlighted`. The Discard action and The
    /// Hook share it.
    pub(super) fn record_discard(&mut self, discarded: Vec<Card>) {
        self.discarded.extend(discarded.iter().copied());
        self.discarded_this_round.extend(discarded.iter().copied());
        self.total_cards_discarded += discarded.len();

        // Ramen loses X0.01 per card discarded and is eaten at X1.
        let ramens: Vec<usize> = self
            .jokers
            .iter()
            .filter(|joker| matches!(joker, Jokers::Ramen(_)))
            .map(|joker| joker.instance_id())
            .collect();
        for id in ramens {
            for _ in 0..discarded.len() {
                let worn = self.joker_state_mut(id).map_or(0.0, |state| state.counter);
                if 2.0 - worn - 0.01 <= 1.0 {
                    self.joker_message(id, JokerNote::Eaten);
                    self.destroy_joker(id);
                    break;
                }
                if let Some(state) = self.joker_state_mut(id) {
                    state.counter += 0.01;
                }
                self.joker_message(id, JokerNote::XMultMinus(1));
            }
        }
        // Purple seals: `Card:calculate_seal` does nothing for a debuffed card.
        for card in &discarded {
            if card.seal == Some(Seal::Purple)
                && !self.is_card_debuffed(card)
                && self.consumables.len() < self.consumable_slots()
            {
                let excl: Vec<Tarot> = self
                    .consumables
                    .iter()
                    .filter_map(|c| match c {
                        Consumable::Tarot(t) => Some(*t),
                        _ => None,
                    })
                    .collect();
                let tarot = self.backend.roll_random_tarot(&excl);
                self.consumables.push(tarot);
                self.emit(GameEvent::SealCreated {
                    card: *card,
                    consumable: tarot,
                });
            }
        }

        let hand = MadeHand {
            hand: SelectHand::new(discarded.clone()),
            rank: HandRank::HighCard,
            all: discarded,
        };
        for e in self.effect_registry.on_discard.clone() {
            if let Effects::OnDiscard(f) = e {
                f.lock().unwrap()(self, hand.clone())
            }
        }
    }

    pub(crate) fn destroy_card(&mut self, id: usize) {
        let face = self
            .full_deck()
            .iter()
            .find(|card| card.id == id)
            .is_some_and(|card| self.is_face_card(card));
        if face {
            self.face_card_destroyed();
        }
        self.available.remove_by_id(id);
        self.deck.remove_by_id(id);
        self.discarded.retain(|c| c.id != id);
    }

    pub(crate) fn mutate_card<F: Fn(&mut Card) + Copy>(&mut self, id: usize, f: F) {
        self.available.mutate_card(id, f);
        self.deck.mutate_card(id, f);
    }

    pub(crate) fn joker_state_mut(&mut self, id: usize) -> Option<&mut JokerState> {
        self.jokers
            .iter_mut()
            .find(|j| j.instance_id() == id)
            .map(|j| j.state_mut())
    }

    pub(crate) fn is_face_card(&self, card: &Card) -> bool {
        card.is_face_card()
            || self
                .effect_registry
                .rule_flags
                .contains(&RuleFlag::AllCardsAreFace)
    }

    pub(crate) fn is_even(&self, card: &Card) -> bool {
        card.is_even_impl(self.is_face_card(card))
    }

    pub(crate) fn is_odd(&self, card: &Card) -> bool {
        card.is_odd_impl(self.is_face_card(card))
    }

    /// Whether the active boss (if any) currently debuffs card
    pub fn is_card_debuffed(&self, card: &Card) -> bool {
        match self.active_boss() {
            Some(BossBlind::Club) => self.is_suit(card, Suit::Club),
            Some(BossBlind::Goad) => self.is_suit(card, Suit::Spade),
            Some(BossBlind::Head) => self.is_suit(card, Suit::Heart),
            Some(BossBlind::Window) => self.is_suit(card, Suit::Diamond),
            Some(BossBlind::Plant) => self.is_face_card(card),
            _ => false,
        }
    }

    // Filters debuffed cards out of a collection
    pub fn non_debuffed<'a>(&self, cards: impl IntoIterator<Item = &'a Card>) -> Vec<Card> {
        cards
            .into_iter()
            .filter(|c| !self.is_card_debuffed(c))
            .copied()
            .collect()
    }

    pub(crate) fn most_played_hand_rank(&self) -> HandRank {
        HandRank::iter()
            // royal flush has no storage slot of its own, its counted as straight flush
            .filter(|r| *r != HandRank::RoyalFlush)
            .max_by_key(|r| self.planetarium.level(*r).plays)
            .unwrap_or(HandRank::HighCard)
    }

    // Every card the player owns this run, regardless of whether it's
    // currently undrawn, in hand, or already discarded this round.
    // `self.deck` alone is only the undrawn remainder mid-round.
    pub fn full_deck(&self) -> Vec<Card> {
        let mut all = self.deck.cards();
        all.extend(self.available.cards());
        all.extend(self.discarded.clone());
        all
    }

    pub fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        // Oops! All 6s doubles `G.GAME.probabilities.normal` once per copy.
        let oops = self.count_jokers(|j| matches!(j, Jokers::OopsAllSixes(_))) as u32;
        let numerator = numerator
            .saturating_mul(self.prob_mult)
            .saturating_mul(1u32 << oops.min(16));
        self.backend.prob_roll(numerator, denominator)
    }

    #[cfg(feature = "ante-extra-jokers")]
    pub(crate) fn roll_misprint_mult(&mut self) -> usize {
        self.backend.roll_misprint_mult()
    }

    #[cfg(feature = "ante-extra-jokers")]
    pub(super) fn age_ice_cream(&mut self) {
        let before = self.jokers.len();
        self.jokers.retain_mut(|joker| {
            if matches!(joker, Jokers::IceCream(_)) {
                let state = joker.state_mut();
                state.counter += 1.0;
                return state.counter < 20.0;
            }
            true
        });
        if self.jokers.len() != before {
            self.effect_registry
                .register_jokers(self.jokers.clone(), &self.clone());
        }
    }
}
