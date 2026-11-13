use crate::action::{Action, MoveDirection};
use crate::ante::Ante;
use crate::available::Available;
use crate::card::{card_display, Card, Edition, Enhancement, Seal, Suit};
use crate::config::{Config, RngMode};
use crate::consumable::Consumable;
use crate::deck::Deck;
use crate::effect::{EffectRegistry, Effects, RuleFlag};
use crate::error::GameError;
use crate::hand::{MadeHand, SelectHand};
use crate::joker::{joker_display, JokerEffects, JokerState, Jokers};
use crate::pack::{OpenPackState, Pack, PackCategory, PackContent, PackSize};
use crate::planet::Planetarium;
use crate::rank::HandRank;
use crate::rng::{Backend, FastBackend, RealBackend, RngBackend};
use crate::score::{ScoreSource, ScoreStep, ScoreTrace};
use crate::shop::{Shop, ShopContext};
use crate::spectral::SpectralEffect;
use crate::stage::{Blind, BlindExt, End, Stage};
use crate::tag::Tag;
use crate::tarot::{Tarot, TarotEffect};
use crate::voucher::{Voucher, Vouchers};

use balatro_types::{BossBlind, Rarity};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;
use std::fmt;
use strum::IntoEnumIterator;

#[cfg(feature = "serde")]
fn default_backend() -> Backend {
    Backend::Fast(FastBackend::new(ChaCha8Rng::from_entropy()))
}

fn default_reroll_cost() -> usize {
    5
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Game {
    pub config: Config,
    pub shop: Shop,
    pub planetarium: Planetarium,
    pub deck: Deck,
    pub available: Available,
    pub discarded: Vec<Card>,
    pub tags: Vec<Tag>,
    pub small_blind_tag: Tag,
    pub big_blind_tag: Tag,
    #[cfg_attr(feature = "serde", serde(default))]
    pub orbital_targets: [Option<HandRank>; 2],
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) pack_returns_to_blinds: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) boss_rerolled: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) skipped_count: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) unused_discards: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) temporary_hand_size: usize,
    pub blind: Option<Blind>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub current_boss: Option<BossBlind>,
    pub stage: Stage,
    pub ante_start: Ante,
    pub ante_end: Ante,
    pub ante_current: Ante,
    #[cfg_attr(feature = "serde", serde(default))]
    pub campaign_won: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub endless_mode: bool,
    pub action_history: Vec<Action>,
    pub round: usize,

    // vouchers redeemed this run, and the ante whose voucher offer
    // currently sits in the shop (the offer survives rerolls and every
    // shop visit of that ante, so it can't be regenerated per refresh)
    #[cfg_attr(feature = "serde", serde(default))]
    pub vouchers: Vouchers,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) voucher_ante: Option<Ante>,

    // jokers and their effects
    pub jokers: Vec<Jokers>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub effect_registry: EffectRegistry,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) trace_xmult: Option<f64>,

    // held consumables (planets, tarots, etc.)
    pub consumables: Vec<Consumable>,

    // playing
    pub plays: usize,
    #[cfg_attr(feature = "serde", serde(rename = "discards"))] // shouldn't access directly
    pub discards_remaining: usize,
    pub reward: usize,
    pub money: usize,

    // for scoring
    pub chips: usize,
    /// Mult is fractional, like the original; the score floors chips x mult.
    pub mult: f64,
    pub score: usize,
    pub prob_mult: u32,

    // shared game-level history some jokers key off (RideTheBus, Obelisk, etc)
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) hand_ranks_played_this_round: HashSet<HandRank>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub consecutive_hands_without_face_card: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub consecutive_hands_not_most_played_type: usize,
    // cards removed via the Discard action only (not played cards)
    #[cfg_attr(feature = "serde", serde(default))]
    pub discarded_this_round: Vec<Card>,
    // run total of cards discarded (Discard action only), never reset.
    #[cfg_attr(feature = "serde", serde(default))]
    pub total_cards_discarded: usize,
    // set when Luchador is sold, disables the current Boss Blind's effect
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) boss_disabled_by_luchador: bool,
    // set when a boss's ability actually triggered (Matador's trigger condition).
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) boss_triggered_this_hand: bool,

    pub last_consumable_used: Option<Consumable>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub last_score: usize,
    #[cfg_attr(feature = "serde", serde(default = "default_reroll_cost"))]
    pub reroll_cost: usize,

    #[cfg_attr(feature = "serde", serde(default))]
    pub seed: u64,
    #[cfg_attr(feature = "serde", serde(default))]
    pub seed_str: Option<String>,
    // The one source of randomness for everything. shop/pack generation,
    // deck shuffling, prob_roll, tag draws, seal-triggered consumables,
    // discard-selector rolls, etc.
    #[cfg_attr(feature = "serde", serde(default = "default_backend"))]
    pub(crate) backend: Backend,
    // track stage so we can come back to it after temp tarot stage
    pub(crate) tarot_prev_stage: Option<Stage>,
    // track stage so we can come back to it after temp spectral stage
    pub(crate) spectral_prev_stage: Option<Stage>,

    // open pack state (set when a pack is purchased and being opened)
    #[cfg_attr(feature = "serde", serde(default))]
    pub open_pack: Option<OpenPackState>,
}

#[derive(Default)]
struct TraceRecorder {
    trace: ScoreTrace,
    money: Option<usize>,
    profile: bool,
    elapsed: std::time::Duration,
}

impl Game {
    pub fn new(config: Config) -> Self {
        let ante_start = Ante::try_from(config.ante_start).unwrap_or(Ante::One);
        let (seed, seed_str) = match (config.seed_str.clone(), config.seed) {
            (Some(s), _) => {
                let u = crate::seed_from_str(&s);
                (u, Some(s))
            }
            (None, Some(u)) => (u, None),
            (None, None) => {
                let u: u64 = thread_rng().gen();
                (u, None)
            }
        };
        // We always need a fast backend (for now)
        // TODO: until real can do everything
        let fast = FastBackend::new(ChaCha8Rng::seed_from_u64(seed));
        let backend = match config.rng_mode {
            RngMode::Fast => Backend::Fast(fast),
            RngMode::Real => {
                // Real mode needs a Balatro-format seed *string*; fall back
                // to the numeric seed's decimal representation if only
                // `config.seed` (no `seed_str`) was given.
                let seed_string = seed_str.clone().unwrap_or_else(|| seed.to_string());
                Backend::Real(RealBackend::new(&seed_string, fast))
            }
        };
        let mut game = Self {
            shop: Shop::new(),
            planetarium: Planetarium::new(),
            deck: Deck::default(),
            available: Available::default(),
            discarded: Vec::new(),
            tags: Vec::new(),
            small_blind_tag: Tag::Uncommon,
            big_blind_tag: Tag::Uncommon,
            orbital_targets: [None; 2],
            pack_returns_to_blinds: false,
            boss_rerolled: false,
            skipped_count: 0,
            unused_discards: 0,
            temporary_hand_size: 0,
            action_history: Vec::new(),
            vouchers: Vouchers::new(),
            voucher_ante: None,
            jokers: Vec::new(),
            effect_registry: EffectRegistry::new(),
            trace_xmult: None,
            consumables: Vec::new(),
            blind: None,
            current_boss: None,
            stage: Stage::PreBlind(),
            ante_start,
            ante_end: Ante::try_from(config.ante_end).unwrap_or(Ante::Eight),
            ante_current: ante_start,
            campaign_won: false,
            endless_mode: false,
            round: config.round_start,
            plays: config.plays,
            discards_remaining: config.discards,
            reward: config.reward_base,
            money: config.money_start,
            chips: config.base_chips,
            mult: config.base_mult as f64,
            score: config.base_score,
            prob_mult: 1,
            hand_ranks_played_this_round: HashSet::new(),
            consecutive_hands_without_face_card: 0,
            consecutive_hands_not_most_played_type: 0,
            discarded_this_round: Vec::new(),
            total_cards_discarded: 0,
            boss_disabled_by_luchador: false,
            boss_triggered_this_hand: false,
            last_consumable_used: None,
            last_score: 0,
            reroll_cost: default_reroll_cost(),
            tarot_prev_stage: None,
            spectral_prev_stage: None,
            open_pack: None,
            seed,
            seed_str,
            backend,
            config,
        };
        game.draw_ante_tags();
        game.draw_ante_boss();
        game
    }

    pub fn start(&mut self) {
        self.stage = Stage::PreBlind();
    }

    pub fn result(&self) -> Option<End> {
        match self.stage {
            Stage::End(end) => Some(end),
            _ => None,
        }
    }

    pub fn is_over(&self) -> bool {
        self.result().is_some()
    }

    // --- voucher-adjusted limits ---
    //
    // Vouchers are never applied to `Config` in place; every limit below
    // is `Config`'s baseline plus what the redeemed set implies. Derived
    // rather than mutated so the effects survive save/load and can't be
    // double-applied. (`config.joker_slots` is the exception, mutated by
    // Negative-edition jokers, which predates vouchers.)

    /// Hand size dealt each round — Paint Brush / Palette raise the
    /// baseline; Manacle (while active) then subtracts one from that.
    pub fn hand_size(&self) -> usize {
        let base =
            (self.config.available + self.vouchers.hand_size_bonus() + self.temporary_hand_size)
                .min(self.config.available_max);
        if self.active_boss() == Some(BossBlind::Manacle) {
            base.saturating_sub(1)
        } else {
            base
        }
    }

    /// Joker slots — Antimatter adds one on top of any Negative-edition
    /// jokers already counted in `config.joker_slots`.
    pub fn joker_slots(&self) -> usize {
        (self.config.joker_slots + self.vouchers.joker_slot_bonus())
            .min(self.config.joker_slots_max)
    }

    /// Consumable slots — Crystal Ball adds one.
    pub fn consumable_slots(&self) -> usize {
        self.config.consumable_slots + self.vouchers.consumable_slot_bonus()
    }

    /// Hands per round — Grabber/Nacho Tong add, Hieroglyph subtracts.
    /// Never drops below one; a run with no hands is unplayable.
    pub fn plays_per_round(&self) -> usize {
        Self::apply_delta(self.config.plays, self.vouchers.hands_delta()).max(1)
    }

    /// Discards per round — Wasteful/Recyclomancy add, Petroglyph
    /// subtracts. May legitimately reach zero.
    pub fn discards_per_round(&self) -> usize {
        Self::apply_delta(self.config.discards, self.vouchers.discards_delta())
    }

    /// Interest cap at cash out — Seed Money / Money Tree raise it.
    pub fn interest_max(&self) -> usize {
        self.config.interest_max + self.vouchers.interest_cap_bonus()
    }

    /// What the next reroll in a fresh shop costs, before the per-reroll
    /// escalation. Reroll Surplus / Reroll Glut take $2 off each.
    pub fn base_reroll_cost(&self) -> usize {
        default_reroll_cost().saturating_sub(self.vouchers.reroll_discount())
    }

    /// Price of a shop item after Clearance Sale / Liquidation. Vouchers
    /// themselves are never discounted.
    pub fn price(&self, cost: usize) -> usize {
        self.vouchers.discounted(cost)
    }

    fn apply_delta(base: usize, delta: isize) -> usize {
        if delta >= 0 {
            base + delta as usize
        } else {
            base.saturating_sub(delta.unsigned_abs())
        }
    }

    /// Bundles what shop generation reads off the game. Borrowing rules
    /// mean callers clone the pieces first, then pass them in — see
    /// `cashout`.
    fn shop_context<'a>(
        &self,
        planetarium: &'a Planetarium,
        held_consumables: &'a [Consumable],
        held_jokers: &'a [Jokers],
        vouchers: &'a Vouchers,
    ) -> ShopContext<'a> {
        ShopContext {
            planetarium,
            held_consumables,
            held_jokers,
            vouchers,
            prob_mult: self.prob_mult,
            ante: self.ante_current.number() as i32,
            most_played: self.most_played_hand_rank(),
        }
    }

    // Fires when the round that just ended cleared the blind, before played
    // cards are swapped out and replacements drawn in. `self.available` still
    // reflects what was actually held in hand at the moment the round ended.
    fn handle_round_end(&mut self) {
        for e in self.effect_registry.on_round_end.clone() {
            if let Effects::OnRoundEnd(f) = e {
                f.lock().unwrap()(self)
            }
        }

        let planetarium = self.planetarium.clone();
        for card in self.available.not_selected() {
            if card.seal == Some(Seal::Blue) && self.consumables.len() < self.consumable_slots() {
                let planet = self.backend.roll_random_planet(&planetarium, &[]);
                self.consumables.push(planet);
            }
        }
    }

    pub(crate) fn clear_blind(&mut self) {
        self.temporary_hand_size = 0;
        self.score = self.config.base_score;
        self.plays = self.plays_per_round();
        self.discards_remaining = self.discards_per_round();
        self.hand_ranks_played_this_round.clear();
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
    fn roll_discard_selectors(&mut self) {
        for j in self.jokers.iter_mut() {
            self.backend.roll_discard_selector(j);
        }
    }

    // draw from deck to available
    fn draw(&mut self, count: usize) {
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

    fn play_selected_inner(&mut self, trace: Option<&mut TraceRecorder>) -> Result<(), GameError> {
        if self.plays == 0 {
            return Err(GameError::NoRemainingPlays);
        }
        self.plays -= 1;
        let selected = SelectHand::new(self.available.selected());
        let best = selected.best_hand()?;
        let scored = best.clone();
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
        let removed = self.available.remove_selected();
        let n = if self.active_boss() == Some(BossBlind::Serpent) {
            3
        } else {
            removed
        };
        self.draw(n);
        for card in scored.hand.cards() {
            if card.enhancement == Some(Enhancement::Glass) && self.prob_roll(1, 4) {
                self.destroy_card(card.id);
            }
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
        self.discards_remaining -= 1;
        let discarded = self.available.selected();
        self.discarded.extend(discarded.iter().copied());
        self.discarded_this_round.extend(discarded.iter().copied());
        self.total_cards_discarded += discarded.len();
        let removed = self.available.remove_selected();
        let n = if self.active_boss() == Some(BossBlind::Serpent) {
            3
        } else {
            removed
        };
        self.draw(n);

        // Check for purple seals
        for card in &discarded {
            if card.seal == Some(Seal::Purple) && self.consumables.len() < self.consumable_slots() {
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
        Ok(())
    }

    pub(crate) fn destroy_card(&mut self, id: usize) {
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
            Some(BossBlind::Club) => card.matches_suit(Suit::Club),
            Some(BossBlind::Goad) => card.matches_suit(Suit::Spade),
            Some(BossBlind::Head) => card.matches_suit(Suit::Heart),
            Some(BossBlind::Window) => card.matches_suit(Suit::Diamond),
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
        self.backend
            .prob_roll(numerator.saturating_mul(self.prob_mult), denominator)
    }

    #[cfg(feature = "ante-extra-jokers")]
    pub(crate) fn roll_misprint_mult(&mut self) -> usize {
        self.backend.roll_misprint_mult()
    }

    #[cfg(feature = "ante-extra-jokers")]
    fn age_ice_cream(&mut self) {
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

    pub fn calc_score(&mut self, hand: MadeHand) -> usize {
        self.calc_score_inner(hand, None)
    }

    /// Same scoring pass as `calc_score`, plus an ordered `ScoreTrace` of
    /// every chips/mult contribution along the way.
    pub fn calc_score_traced(&mut self, hand: MadeHand) -> (usize, ScoreTrace) {
        let mut recorder = TraceRecorder {
            money: Some(self.money),
            ..TraceRecorder::default()
        };
        let score = self.calc_score_inner(hand, Some(&mut recorder));
        (score, recorder.trace)
    }

    // Multiplicative steps note their factor so the trace can tell XMult from +Mult.
    pub(crate) fn note_xmult(&mut self, factor: f64) {
        self.trace_xmult = Some(self.trace_xmult.unwrap_or(1.0) * factor);
    }

    fn record_step(
        &mut self,
        trace: &mut Option<&mut TraceRecorder>,
        source: ScoreSource,
        chips_before: usize,
        mult_before: f64,
        retrigger: bool,
    ) {
        let xmult = self.trace_xmult.take();
        let Some(t) = trace else {
            return;
        };
        let money_before = t.money.replace(self.money).unwrap_or(self.money);
        let money = self.money as i64 - money_before as i64;
        // skip no-op checks (e.g. a Base-edition joker, a conditional
        // effect that didn't fire)
        if chips_before == self.chips && mult_before == self.mult && money == 0 {
            return;
        }
        let xmult = xmult.filter(|_| mult_before != self.mult);
        let start = t.profile.then(std::time::Instant::now);
        t.trace.0.push(ScoreStep {
            source,
            chips_before,
            chips_after: self.chips,
            mult_before,
            mult_after: self.mult,
            retrigger,
            xmult,
            money,
        });
        if let Some(start) = start {
            t.elapsed += start.elapsed();
        }
    }

    // Extension point for retrigger (jokers + red seal)
    fn trigger_count_played(&mut self, card: &Card, is_first: bool) -> usize {
        let mut n = if card.seal == Some(Seal::Red) { 2 } else { 1 };
        for e in self.effect_registry.trigger_count_played.clone() {
            if let Effects::TriggerCountPlayed(f) = e {
                n += f.lock().unwrap()(self, *card, is_first);
            }
        }
        n
    }

    // Extension point for retrigger (jokers + red seal)
    fn trigger_count_held(&mut self, card: &Card) -> usize {
        let mut n = if card.seal == Some(Seal::Red) { 2 } else { 1 };
        for e in self.effect_registry.trigger_count_held.clone() {
            if let Effects::TriggerCountHeld(f) = e {
                n += f.lock().unwrap()(self, *card, false);
            }
        }
        n
    }

    fn calc_score_inner(
        &mut self,
        mut hand: MadeHand,
        mut trace: Option<&mut TraceRecorder>,
    ) -> usize {
        self.boss_triggered_this_hand = false;

        // compute chips and mult from hand level
        let chips_before = self.chips;
        let mult_before = self.mult;
        if self.active_boss() == Some(BossBlind::Arm) && self.planetarium.level(hand.rank).level > 1
        {
            self.planetarium.level_down(hand.rank);
            self.boss_triggered_this_hand = true;
        }
        let level = self.planetarium.play(hand.rank);
        let (level_chips, level_mult) = if self.active_boss() == Some(BossBlind::Flint) {
            self.boss_triggered_this_hand = true;
            // Halved rounding to nearest, Mult at least 1, like `Blind:modify_hand`.
            (level.chips.div_ceil(2), level.mult.div_ceil(2).max(1))
        } else {
            (level.chips, level.mult)
        };
        self.chips += level_chips;
        self.mult += level_mult as f64;
        self.record_step(
            &mut trace,
            ScoreSource::HandLevel(hand.rank),
            chips_before,
            mult_before,
            false,
        );

        // Observatory: every held Planet card whose own hand is the one
        // being scored multiplies Mult by 1.5. Applied straight after the
        // hand level, before any card or joker contribution.
        if self.vouchers.observatory() {
            let matching = self
                .consumables
                .iter()
                .filter(|c| matches!(c, Consumable::Planet(p) if p.hand_rank() == hand.rank))
                .count();
            for _ in 0..matching {
                let chips_before = self.chips;
                let mult_before = self.mult;
                self.mult *= 1.5;
                self.note_xmult(1.5);
                self.record_step(
                    &mut trace,
                    ScoreSource::Voucher(Voucher::Observatory),
                    chips_before,
                    mult_before,
                    false,
                );
            }
        }

        // first card in original play order that's actually used in scoring:
        // either part of the made hand, or a stone kicker (which always scores).
        // ordinary kickers that never score don't count, even if played first.
        let hand_ids: Vec<usize> = hand.hand.cards().iter().map(|c| c.id).collect();
        let first_played_id = hand
            .all
            .iter()
            .find(|c| hand_ids.contains(&c.id) || c.enhancement == Some(Enhancement::Stone))
            .map(|c| c.id);

        // per-scored-card, rank chips, enhancements, editions
        for card in hand.hand.cards() {
            if self.is_card_debuffed(&card) {
                self.boss_triggered_this_hand = true;
                continue;
            }
            let is_first = Some(card.id) == first_played_id;
            for trig in 0..self.trigger_count_played(&card, is_first) {
                let chips_before = self.chips;
                let mult_before = self.mult;

                // stone has no rank, skip normal chip value
                if card.enhancement != Some(Enhancement::Stone) {
                    self.chips += card.chips();
                }
                match card.enhancement {
                    Some(Enhancement::Bonus) => self.chips += 30,
                    Some(Enhancement::Mult) => self.mult += 4.0,
                    Some(Enhancement::Glass) => {
                        self.mult *= 2.0;
                        self.note_xmult(2.0);
                    }
                    Some(Enhancement::Lucky) => {
                        if self.prob_roll(1, 5) {
                            self.mult += 20.0;
                        }
                        if self.prob_roll(1, 15) {
                            self.money += 20;
                        }
                    }
                    _ => {}
                }
                match card.edition {
                    Edition::Foil => self.chips += 50,
                    Edition::Holographic => self.mult += 10.0,
                    Edition::Polychrome => {
                        self.mult *= 1.5;
                        self.note_xmult(1.5);
                    }
                    _ => {}
                }
                if card.seal == Some(Seal::Gold) {
                    self.money += 3;
                }

                self.record_step(
                    &mut trace,
                    ScoreSource::PlayedCard(card),
                    chips_before,
                    mult_before,
                    trig > 0,
                );
            }
        }

        // stone cards always score +50 chips, even as kickers outside the played
        // hand's own scoring subset.
        // still subject to the same retrigger rules as any other played card.
        for card in &hand.all {
            if card.enhancement == Some(Enhancement::Stone) {
                if self.is_card_debuffed(card) {
                    self.boss_triggered_this_hand = true;
                    continue;
                }
                let is_first = Some(card.id) == first_played_id;
                for trig in 0..self.trigger_count_played(card, is_first) {
                    let chips_before = self.chips;
                    let mult_before = self.mult;
                    self.chips += 50;
                    self.record_step(
                        &mut trace,
                        ScoreSource::StoneKicker(*card),
                        chips_before,
                        mult_before,
                        trig > 0,
                    );
                }
            }
        }

        // held in hand effects (cards not played this hand)
        for card in self.available.not_selected() {
            if self.is_card_debuffed(&card) {
                // Held-card Matador trigger unconfirmed, real-game check needed.
                if matches!(
                    card.enhancement,
                    Some(Enhancement::Steel) | Some(Enhancement::Gold)
                ) {
                    self.boss_triggered_this_hand = true;
                }
                continue;
            }
            for trig in 0..self.trigger_count_held(&card) {
                let chips_before = self.chips;
                let mult_before = self.mult;
                match card.enhancement {
                    Some(Enhancement::Steel) => {
                        self.mult *= 1.5;
                        self.note_xmult(1.5);
                    }
                    Some(Enhancement::Gold) => self.money += 3,
                    _ => {}
                }
                self.record_step(
                    &mut trace,
                    ScoreSource::HeldCard(card),
                    chips_before,
                    mult_before,
                    trig > 0,
                );
            }
        }

        // run hand modifiers (e.g. Pareidolia) before joker scoring effects
        for e in self.effect_registry.on_modify_hand.clone() {
            if let Effects::OnModifyHand(f) = e {
                f.lock().unwrap()(self, &mut hand)
            }
        }

        // apply joker effects with per-joker edition ordering.
        // effects recomputed per joker, not read from EffectRegistry. Each joker's edition
        // bonus must sandwich its own OnScore closures in self.jokers order.
        for joker in self.jokers.clone() {
            let chips_before = self.chips;
            let mult_before = self.mult;
            match joker.edition() {
                Edition::Foil => self.chips += 50,
                Edition::Holographic => self.mult += 10.0,
                _ => {}
            }
            self.record_step(
                &mut trace,
                ScoreSource::Joker(joker.clone()),
                chips_before,
                mult_before,
                false,
            );

            for e in joker.effects(self) {
                if let Effects::OnScore(f) = e {
                    let chips_before = self.chips;
                    let mult_before = self.mult;
                    f.lock().unwrap()(self, hand.clone());
                    self.record_step(
                        &mut trace,
                        ScoreSource::Joker(joker.clone()),
                        chips_before,
                        mult_before,
                        false,
                    );
                }
            }

            if joker.edition() == Edition::Polychrome {
                let chips_before = self.chips;
                let mult_before = self.mult;
                self.mult *= 1.5;
                self.note_xmult(1.5);
                self.record_step(
                    &mut trace,
                    ScoreSource::Joker(joker.clone()),
                    chips_before,
                    mult_before,
                    false,
                );
            }
        }

        // record after joker loop, not before (for CardSharp)
        self.hand_ranks_played_this_round.insert(hand.rank);

        // compute score
        let score = (self.chips as f64 * self.mult).floor() as usize;

        // reset chips and mult
        self.mult = self.config.base_mult as f64;
        self.chips = self.config.base_chips;

        score
    }

    pub fn required_score(&self) -> usize {
        self.required_score_for(self.blind)
    }

    pub fn required_score_for(&self, blind: Option<Blind>) -> usize {
        let base = self.ante_current.base();
        match blind {
            None => base,
            Some(Blind::Small) => base,
            Some(Blind::Big) => base + base / 2,
            Some(Blind::Boss) => {
                let boss = if self.blind == blind {
                    self.active_boss()
                } else {
                    self.current_boss
                };
                base * Self::boss_score_multiplier(boss)
            }
        }
    }

    /// The score the upcoming Boss Blind will require, based on
    /// `current_boss`, valid before boss blind is active.
    pub fn boss_required_score(&self) -> usize {
        self.ante_current.base() * Self::boss_score_multiplier(self.current_boss)
    }

    fn boss_score_multiplier(boss: Option<BossBlind>) -> usize {
        match boss {
            Some(BossBlind::Wall) => 4,
            Some(BossBlind::VioletVessel) => 6,
            Some(BossBlind::Needle) => 1,
            _ => 2,
        }
    }

    fn calc_reward(&mut self, blind: Blind) -> Result<usize, GameError> {
        let mut interest = (self.money as f32 * self.config.interest_rate).floor() as usize;
        let interest_max = self.interest_max();
        if interest > interest_max {
            interest = interest_max
        }
        let base = blind.reward();
        let hand_bonus = self.plays * self.config.money_per_hand;
        let reward = base + interest + hand_bonus;
        Ok(reward)
    }

    fn take_tag(&mut self, tag: Tag) -> bool {
        if let Some(index) = self.tags.iter().position(|candidate| *candidate == tag) {
            self.tags.remove(index);
            true
        } else {
            false
        }
    }

    fn acquire_tag(&mut self, tag: Tag, orbital: Option<HandRank>) {
        let copies = if tag == Tag::Double {
            1
        } else {
            let before = self.tags.len();
            self.tags.retain(|tag| *tag != Tag::Double);
            1 + before - self.tags.len()
        };
        for _ in 0..copies {
            match tag {
                Tag::Speed => self.money += self.skipped_count * 5,
                Tag::Handy => {
                    self.money += HandRank::iter()
                        .filter(|rank| *rank != HandRank::RoyalFlush)
                        .map(|rank| self.planetarium.level(rank).plays)
                        .sum::<usize>()
                }
                Tag::Garbage => self.money += self.unused_discards,
                Tag::Economy => self.money += self.money.min(40),
                Tag::Orbital => {
                    let rank = orbital.unwrap_or(HandRank::HighCard);
                    for _ in 0..3 {
                        self.planetarium.level_up(rank);
                    }
                }
                Tag::TopUp => {
                    for _ in 0..2 {
                        if self.jokers.len() >= self.joker_slots() {
                            break;
                        }
                        let mut joker = self.backend.gen_joker_of_rarity(
                            self.ante_current.number() as i32,
                            self.prob_mult,
                            &self.jokers,
                            Rarity::Common,
                        );
                        joker.set_edition(Edition::Base);
                        self.backend.on_joker_bought(&joker);
                        self.jokers.push(joker);
                    }
                    self.effect_registry
                        .register_jokers(self.jokers.clone(), &self.clone());
                }
                _ => self.tags.push(tag),
            }
        }
    }

    fn resolve_blind_choice_tags(&mut self) {
        while let Some(index) = self.tags.iter().position(|tag| {
            matches!(
                tag,
                Tag::Boss | Tag::Standard | Tag::Charm | Tag::Meteor | Tag::Ethereal | Tag::Buffoon
            )
        }) {
            let (category, size) = match self.tags.remove(index) {
                Tag::Boss => {
                    self.replace_boss();
                    continue;
                }
                Tag::Standard => (PackCategory::Standard, PackSize::Mega),
                Tag::Charm => (PackCategory::Arcana, PackSize::Mega),
                Tag::Meteor => (PackCategory::Celestial, PackSize::Mega),
                Tag::Ethereal => (PackCategory::Spectral, PackSize::Normal),
                Tag::Buffoon => (PackCategory::Buffoon, PackSize::Mega),
                _ => unreachable!(),
            };
            let planetarium = self.planetarium.clone();
            let consumables = self.consumables.clone();
            let jokers = self.jokers.clone();
            let vouchers = self.vouchers.clone();
            let ctx = self.shop_context(&planetarium, &consumables, &jokers, &vouchers);
            let pack = self.backend.tag_pack(category, size, &ctx);
            self.pack_returns_to_blinds = true;
            self.open_booster(pack);
            break;
        }
    }

    fn resolve_shop_tags(&mut self, opening: bool) {
        let mut tags: Vec<_> = std::mem::take(&mut self.tags)
            .into_iter()
            .enumerate()
            .collect();
        tags.sort_by_key(|(_, tag)| !matches!(tag, Tag::Uncommon | Tag::Rare));
        let mut pending = Vec::new();
        let mut edited = HashSet::new();
        let mut coupon = false;
        for (index, tag) in tags {
            match tag {
                Tag::Uncommon | Tag::Rare => {
                    if self.shop.jokers.len() == edited.len()
                        && self.shop.consumables.is_empty()
                        && self.shop.cards.is_empty()
                    {
                        pending.push((index, tag));
                        continue;
                    }
                    let rarity = if tag == Tag::Rare {
                        Rarity::Rare
                    } else {
                        Rarity::Uncommon
                    };
                    let mut excluded = self.jokers.clone();
                    excluded.extend(self.shop.jokers.iter().cloned());
                    let mut joker = self.backend.gen_joker_of_rarity(
                        self.ante_current.number() as i32,
                        self.prob_mult,
                        &excluded,
                        rarity,
                    );
                    joker.set_edition(Edition::Base);
                    if let Some(index) = self
                        .shop
                        .jokers
                        .iter()
                        .rposition(|offer| !edited.contains(&offer.instance_id()))
                    {
                        self.shop.jokers.remove(index);
                    } else if self.shop.consumables.pop().is_none() {
                        self.shop.cards.pop();
                    }
                    self.shop.free_jokers.push(joker.instance_id());
                    edited.insert(joker.instance_id());
                    self.shop.jokers.push(joker);
                }
                Tag::Foil | Tag::Holographic | Tag::Polychrome | Tag::Negative => {
                    let edition = match tag {
                        Tag::Foil => Edition::Foil,
                        Tag::Holographic => Edition::Holographic,
                        Tag::Polychrome => Edition::Polychrome,
                        _ => Edition::Negative,
                    };
                    if let Some(joker) = self.shop.jokers.iter_mut().find(|joker| {
                        joker.edition() == Edition::Base && !edited.contains(&joker.instance_id())
                    }) {
                        joker.set_edition(edition);
                        edited.insert(joker.instance_id());
                        self.shop.free_jokers.push(joker.instance_id());
                    } else {
                        pending.push((index, tag));
                    }
                }
                Tag::Coupon if opening && !coupon => coupon = true,
                Tag::D6 if opening && self.reroll_cost != 0 => self.reroll_cost = 0,
                Tag::Voucher if opening => {
                    let mut unavailable = self.vouchers.clone();
                    for offer in self.shop.voucher_offers() {
                        unavailable.redeem(offer);
                    }
                    let mut extra = None;
                    while let Some(voucher) = self
                        .backend
                        .gen_voucher(self.ante_current.number() as i32, &unavailable)
                    {
                        if self.vouchers.is_offerable(voucher) {
                            extra = Some(voucher);
                            break;
                        }
                        unavailable.redeem(voucher);
                    }
                    if let Some(voucher) = extra {
                        self.shop.extra_vouchers.push(voucher);
                    } else {
                        pending.push((index, tag));
                    }
                }
                _ => pending.push((index, tag)),
            }
        }
        pending.sort_unstable_by_key(|(index, _)| *index);
        self.tags = pending.into_iter().map(|(_, tag)| tag).collect();
        if coupon {
            self.shop.coupon();
        }
    }

    pub fn boss_reroll_available(&self) -> bool {
        self.stage == Stage::PreBlind()
            && self.current_boss.is_some()
            && self.money >= 10
            && (self.vouchers.has(Voucher::Retcon)
                || (self.vouchers.has(Voucher::DirectorsCut) && !self.boss_rerolled))
    }

    fn replace_boss(&mut self) {
        let previous = self.current_boss;
        for _ in 0..32 {
            self.draw_ante_boss();
            if self.current_boss != previous {
                break;
            }
        }
        self.boss_rerolled = true;
    }

    fn cashout(&mut self) -> Result<(), GameError> {
        self.money += self.reward;
        self.reward = 0;
        self.reroll_cost = self.base_reroll_cost();
        self.stage = Stage::Shop();
        self.refresh_voucher_offer();
        let planetarium = self.planetarium.clone();
        let held_consumables = self.consumables.clone();
        let held_jokers = self.jokers.clone();
        let vouchers = self.vouchers.clone();
        let ctx = self.shop_context(&planetarium, &held_consumables, &held_jokers, &vouchers);
        self.shop.refresh(&ctx, &mut self.backend);
        self.shop.extra_vouchers.clear();
        self.resolve_shop_tags(true);
        Ok(())
    }

    /// Draws this ante's voucher the first time its shop opens. Within an
    /// ante the offer is left alone: it survives rerolls, and once bought
    /// the slot stays empty until the next ante.
    fn refresh_voucher_offer(&mut self) {
        // Strictly forward: Hieroglyph/Petroglyph walk the ante backwards,
        // and an ante already drawn for must not offer a second voucher.
        if self.voucher_ante.is_some_and(|a| self.ante_current <= a) {
            return;
        }
        self.voucher_ante = Some(self.ante_current);
        let vouchers = self.vouchers.clone();
        self.shop.voucher = self
            .backend
            .gen_voucher(self.ante_current.number() as i32, &vouchers);
    }

    pub(crate) fn reroll(&mut self) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.money < self.reroll_cost {
            return Err(GameError::InvalidBalance);
        }
        self.money -= self.reroll_cost;
        self.reroll_cost += 1;
        let planetarium = self.planetarium.clone();
        let mut held = self.consumables.clone();
        held.extend(self.shop.consumables.clone());
        let mut held_jokers = self.jokers.clone();
        held_jokers.extend(self.shop.jokers.clone());
        let vouchers = self.vouchers.clone();
        // Note: `refresh_cards`, not `refresh` — rerolls leave the packs
        // and the voucher offer untouched.
        let ctx = self.shop_context(&planetarium, &held, &held_jokers, &vouchers);
        self.shop.refresh_cards(&ctx, &mut self.backend);
        self.resolve_shop_tags(false);
        Ok(())
    }

    pub(crate) fn sell_joker(&mut self, idx: usize) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        if idx >= self.jokers.len() {
            return Err(GameError::InvalidAction);
        }
        self.money += self.jokers[idx].sell_value();
        let sold = self.jokers.remove(idx);
        for e in sold.effects(self) {
            if let Effects::OnSell(f) = e {
                f.lock().unwrap()(self);
            }
        }
        if sold.edition() == Edition::Negative {
            self.config.joker_slots = self.config.joker_slots.saturating_sub(1);
        }
        // Only unlock if this was the last owned copy (Showman is the only
        // way to own more than one) — a discriminant compare, matching
        // shop.rs's own exclude-list comparisons, since Jokers' derived
        // Eq also compares edition/stickers, not just "which joker".
        let still_owned = self
            .jokers
            .iter()
            .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&sold));
        if !still_owned {
            self.backend.on_joker_sold(&sold);
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
        Ok(())
    }

    pub(crate) fn sell_consumable(&mut self, idx: usize) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        if idx >= self.consumables.len() {
            return Err(GameError::InvalidAction);
        }
        self.money += self.consumables[idx].sell_value();
        self.consumables.remove(idx);
        Ok(())
    }

    pub(crate) fn can_add_joker(&self, joker: &Jokers) -> bool {
        let slots = self.joker_slots() + usize::from(joker.edition() == Edition::Negative);
        self.jokers.len() < slots.min(self.config.joker_slots_max)
    }

    pub(crate) fn buy_joker(&mut self, joker: Jokers) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if !self.can_add_joker(&joker) {
            return Err(GameError::NoAvailableSlot);
        }
        let price = self.shop.joker_price(&joker, &self.vouchers);
        if price > self.money {
            return Err(GameError::InvalidBalance);
        }
        self.shop.buy_joker(&joker)?;
        self.money -= price;
        self.add_acquired_joker(joker);
        Ok(())
    }

    fn add_acquired_joker(&mut self, joker: Jokers) {
        if joker.edition() == Edition::Negative {
            self.config.joker_slots =
                (self.config.joker_slots + 1).min(self.config.joker_slots_max);
        }
        self.backend.on_joker_bought(&joker);
        self.jokers.push(joker);
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
    }

    pub(crate) fn buy_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.consumables.len() >= self.consumable_slots() {
            return Err(GameError::NoAvailableSlot);
        }
        if self.shop.consumable_price(&consumable, &self.vouchers) > self.money {
            return Err(GameError::InvalidBalance);
        }
        let price = self.shop.consumable_price(&consumable, &self.vouchers);
        self.shop.buy_consumable(&consumable)?;
        self.shop
            .free_consumables
            .retain(|offer| *offer != consumable);
        self.money -= price;
        self.consumables.push(consumable);
        Ok(())
    }

    /// Redeems the shop's voucher offer. Permanent for the run — there is
    /// no sell action, and the slot stays empty until the next ante.
    pub(crate) fn buy_voucher(&mut self, voucher: Voucher) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.vouchers.has(voucher) {
            return Err(GameError::VoucherAlreadyRedeemed);
        }
        // Vouchers are exempt from Clearance Sale / Liquidation.
        if voucher.cost() > self.money {
            return Err(GameError::InvalidBalance);
        }
        if !self.vouchers.is_offerable(voucher) {
            return Err(GameError::InvalidAction);
        }
        self.shop.buy_voucher(&voucher)?;
        self.money -= voucher.cost();
        self.vouchers.redeem(voucher);
        self.backend.on_voucher_bought(&voucher);
        self.apply_voucher(voucher);
        Ok(())
    }

    /// The parts of a voucher that aren't derived from the redeemed set on
    /// demand (see `voucher.rs`) — one-shot changes to live game state.
    fn apply_voucher(&mut self, voucher: Voucher) {
        // "-1 Ante" (Hieroglyph/Petroglyph): the run steps back an ante,
        // making every remaining blind easier but adding a round.
        if Vouchers::lowers_ante(voucher) {
            if let Some(prev) = self.ante_current.prev() {
                self.ante_current = prev;
            }
        }
        // Reroll discounts apply to the shop the player is standing in,
        // not just the next one.
        if matches!(voucher, Voucher::RerollSurplus | Voucher::RerollGlut) {
            self.reroll_cost = self.reroll_cost.saturating_sub(2);
        }
        // Overstock widens the current shop immediately: top up the new
        // slots rather than making the player wait for the next round.
        if matches!(voucher, Voucher::Overstock | Voucher::OverstockPlus) {
            let planetarium = self.planetarium.clone();
            let mut held = self.consumables.clone();
            held.extend(self.shop.consumables.clone());
            let mut held_jokers = self.jokers.clone();
            held_jokers.extend(self.shop.jokers.clone());
            let vouchers = self.vouchers.clone();
            let ctx = self.shop_context(&planetarium, &held, &held_jokers, &vouchers);
            self.shop.top_up_cards(&ctx, &mut self.backend);
        }
    }

    pub(crate) fn buy_playing_card(&mut self, card: Card) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.deck.len() + self.available.cards().len() + self.discarded.len()
            >= self.config.deck_max
        {
            return Err(GameError::NoAvailableSlot);
        }
        let cost = self.shop.card_price(&card, &self.vouchers);
        if cost > self.money {
            return Err(GameError::InvalidBalance);
        }
        let bought = self.shop.buy_card(&card)?;
        self.money -= cost;
        self.deck.push(bought);
        Ok(())
    }

    /// Règles d'utilisation d'un consommable, paraphrasées de Card:can_use_consumeable.
    /// Source unique : appelée par le générateur et par use_consumable, avant tout retrait.
    pub(crate) fn consumable_usable(&self, c: &Consumable) -> bool {
        use crate::spectral::Spectral;
        use crate::tarot::Tarot;
        if matches!(
            self.stage,
            Stage::End(_) | Stage::TarotHand(_) | Stage::SpectralHand(_) | Stage::PackOpen()
        ) {
            return false;
        }
        let selected = self.available.selected();
        let n = selected.len();
        let hand = self.available.cards().len();
        let blind = self.stage.is_blind();
        let joker_room = self.jokers.len() < self.joker_slots();
        let has_editionless_joker = self.jokers.iter().any(|j| j.edition() == Edition::Base);
        match c {
            Consumable::Planet(_) => true,
            // Sans condition (la carte est dans la zone de consommables, donc la place est acquise).
            Consumable::Tarot(
                Tarot::Hermit | Tarot::Temperance | Tarot::Emperor | Tarot::HighPriestess,
            ) => true,
            Consumable::Spectral(Spectral::BlackHole) => true,
            // Copie le dernier Tarot ou Planète utilisé, jamais The Fool lui-même.
            Consumable::Tarot(Tarot::Fool) => self
                .last_consumable_used
                .is_some_and(|last| last != Consumable::Tarot(Tarot::Fool)),
            Consumable::Tarot(Tarot::WheelOfFortune) => !self.jokers.is_empty(),
            // Création de joker : une place libre est requise.
            Consumable::Tarot(Tarot::Judgement)
            | Consumable::Spectral(Spectral::Wraith | Spectral::Soul) => joker_room,
            Consumable::Spectral(Spectral::Ankh) => {
                !self.jokers.is_empty() && self.joker_slots() > 1
            }
            Consumable::Spectral(Spectral::Hex | Spectral::Ectoplasm) => has_editionless_joker,
            // Zéro cible mais touche la main : Blind, main de plus d'une carte, sélection ignorée.
            Consumable::Spectral(
                Spectral::Familiar
                | Spectral::Grim
                | Spectral::Incantation
                | Spectral::Sigil
                | Spectral::Ouija
                | Spectral::Immolate,
            ) => blind && hand > 1,
            // Aura : une seule carte sélectionnée, sans édition.
            Consumable::Spectral(Spectral::Aura) => {
                blind && n == 1 && matches!(selected[0].edition, Edition::Base)
            }
            // Cibles réelles : sélection dans la plage min..=max.
            Consumable::Tarot(t) if t.requires_targets() => {
                blind && n >= t.min_targets() && n <= t.max_targets()
            }
            Consumable::Spectral(s) if s.requires_targets() => {
                blind && n >= s.min_targets() && n <= s.max_targets()
            }
            _ => false,
        }
    }

    pub(crate) fn use_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        if matches!(
            self.stage,
            Stage::End(_) | Stage::TarotHand(_) | Stage::SpectralHand(_) | Stage::PackOpen()
        ) {
            return Err(GameError::InvalidAction);
        }
        // Refus avant tout retrait : la carte ne doit jamais être perdue.
        if !self.consumable_usable(&consumable) {
            return Err(GameError::InvalidAction);
        }
        let i = self
            .consumables
            .iter()
            .position(|c| c == &consumable)
            .ok_or(GameError::InvalidAction)?;
        self.consumables.remove(i);
        self.apply_consumable(consumable)
    }

    // Applies a consumable that has already left its slot or the shop.
    fn apply_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        match consumable {
            Consumable::Planet(planet) => {
                self.planetarium.level_up(planet.hand_rank());
                self.last_consumable_used = Some(Consumable::Planet(planet));
            }
            Consumable::Tarot(t) => {
                t.apply(self)?;
                if t != Tarot::Fool {
                    self.last_consumable_used = Some(Consumable::Tarot(t));
                }
            }
            Consumable::Spectral(s) => {
                // Fool only ever copies the last Tarot or Planet used, never a Spectral.
                s.apply(self)?;
            }
        }
        Ok(())
    }

    // Room for cards a consumable creates when it is not already in a consumable slot,
    // like `can_use_consumeable` for a card in a pack or the shop.
    fn creation_room(&self, c: &Consumable) -> bool {
        use crate::spectral::Spectral;
        let room = self.consumables.len() < self.consumable_slots();
        match c {
            Consumable::Tarot(Tarot::Emperor | Tarot::HighPriestess) => room,
            Consumable::Tarot(Tarot::Fool) => {
                room && self
                    .last_consumable_used
                    .is_some_and(|last| last != Consumable::Tarot(Tarot::Fool))
            }
            Consumable::Tarot(Tarot::Judgement)
            | Consumable::Spectral(Spectral::Wraith | Spectral::Soul) => {
                self.jokers.len() < self.joker_slots()
            }
            _ => true,
        }
    }

    /// Whether a pack card can be taken now: a Joker needs a slot, and a card that
    /// creates consumables or Jokers needs room for them.
    pub fn pack_card_pickable(&self, content: &PackContent) -> bool {
        match content {
            PackContent::Joker(joker) => self.can_add_joker(joker),
            PackContent::Tarot(t) => self.creation_room(&Consumable::Tarot(*t)),
            PackContent::Spectral(s) => self.creation_room(&Consumable::Spectral(*s)),
            _ => true,
        }
    }

    /// Whether a shop consumable can be bought and used at once, like `can_buy_and_use`:
    /// affordable, needing no target, and usable from the shop.
    pub fn can_buy_and_use(&self, c: &Consumable) -> bool {
        let targets = match c {
            Consumable::Tarot(t) => t.requires_targets(),
            Consumable::Spectral(s) => s.requires_targets(),
            Consumable::Planet(_) => false,
        };
        self.stage == Stage::Shop()
            && self.shop.consumables.contains(c)
            && self.shop.consumable_price(c, &self.vouchers) <= self.money
            && !targets
            && self.creation_room(c)
            && self.consumable_usable(c)
    }

    /// Buys a shop consumable and uses it at once, without taking a consumable slot.
    pub fn buy_and_use_consumable(&mut self, c: Consumable) -> Result<(), GameError> {
        if !self.can_buy_and_use(&c) {
            return Err(GameError::InvalidAction);
        }
        let price = self.shop.consumable_price(&c, &self.vouchers);
        self.shop.buy_consumable(&c)?;
        self.shop.free_consumables.retain(|offer| *offer != c);
        self.money -= price;
        self.apply_consumable(c)
    }

    pub(crate) fn apply_tarot(&mut self) -> Result<(), GameError> {
        let Stage::TarotHand(t) = self.stage else {
            return Err(GameError::InvalidStage);
        };
        let prev = self
            .tarot_prev_stage
            .take()
            .ok_or(GameError::InvalidStage)?;
        let selected_count = self.available.selected().len();
        if selected_count < t.min_targets() || selected_count > t.max_targets() {
            self.tarot_prev_stage = Some(prev);
            return Err(GameError::InvalidAction);
        }
        t.apply(self)?;
        if t != Tarot::Fool {
            self.last_consumable_used = Some(Consumable::Tarot(t));
        }
        self.stage = prev;
        // Only reachable via a pack pick now, finish_pack handles hand cleanup
        if prev.is_pack_open() {
            if let Some(ref mut state) = self.open_pack {
                state.picks_remaining = state.picks_remaining.saturating_sub(1);
            }
            let done = self
                .open_pack
                .as_ref()
                .is_some_and(|s| s.picks_remaining == 0);
            if done {
                self.finish_pack();
            }
        }
        Ok(())
    }

    pub(crate) fn apply_spectral(&mut self) -> Result<(), GameError> {
        let Stage::SpectralHand(s) = self.stage else {
            return Err(GameError::InvalidStage);
        };
        let prev = self
            .spectral_prev_stage
            .take()
            .ok_or(GameError::InvalidStage)?;
        let selected_count = self.available.selected().len();
        if selected_count < s.min_targets() || selected_count > s.max_targets() {
            self.spectral_prev_stage = Some(prev);
            return Err(GameError::InvalidAction);
        }
        // Fool only ever copies the last Tarot or Planet used, never a Spectral.
        s.apply(self)?;
        self.stage = prev;
        // Only reachable via a pack pick now; finish_pack handles hand cleanup
        if prev.is_pack_open() {
            if let Some(ref mut state) = self.open_pack {
                state.picks_remaining = state.picks_remaining.saturating_sub(1);
            }
            let done = self
                .open_pack
                .as_ref()
                .is_some_and(|s| s.picks_remaining == 0);
            if done {
                self.finish_pack();
            }
        }
        Ok(())
    }

    pub(crate) fn buy_pack(&mut self, pack: Pack) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        let price = self.shop.pack_price(&pack, &self.vouchers);
        if price > self.money {
            return Err(GameError::InvalidBalance);
        }
        self.shop.buy_pack(&pack)?;
        self.money -= price;
        self.open_booster(pack);
        Ok(())
    }

    fn open_booster(&mut self, mut pack: Pack) {
        if self.vouchers.omen_globe() {
            self.backend
                .omen_globe_contents(self.ante_current.number() as i32, &mut pack);
        }
        // Arcana/Spectral packs draw a hand for the player to apply targeted
        // tarots/spectrals against
        if matches!(pack.category, PackCategory::Arcana | PackCategory::Spectral) {
            self.draw(self.hand_size());
        }

        self.open_pack = Some(OpenPackState {
            picks_remaining: pack.picks_allowed(),
            description: pack.description(),
            contents: pack.contents,
        });
        self.stage = Stage::PackOpen();
    }

    pub(crate) fn pick_pack_card(&mut self, content: PackContent) -> Result<(), GameError> {
        if self.stage != Stage::PackOpen() {
            return Err(GameError::InvalidStage);
        }

        if !self.pack_card_pickable(&content) {
            return Err(GameError::NoAvailableSlot);
        }
        // Remove the chosen content from the pack
        {
            let state = self.open_pack.as_mut().ok_or(GameError::InvalidAction)?;
            let pos = state
                .contents
                .iter()
                .position(|c| c == &content)
                .ok_or(GameError::InvalidAction)?;
            state.contents.remove(pos);
        }

        // Targeting tarots enter TarotHand; picks_remaining decremented later in apply_tarot
        if let PackContent::Tarot(t) = &content {
            if t.requires_targets() {
                self.tarot_prev_stage = Some(Stage::PackOpen());
                self.stage = Stage::TarotHand(*t);
                return Ok(());
            }
        }
        // Targeting spectrals enter SpectralHand; picks_remaining decremented later in apply_spectral
        if let PackContent::Spectral(s) = &content {
            if s.requires_targets() {
                self.spectral_prev_stage = Some(Stage::PackOpen());
                self.stage = Stage::SpectralHand(*s);
                return Ok(());
            }
        }

        match content {
            PackContent::Tarot(t) => {
                t.apply(self)?;
            }
            PackContent::Planet(p) => {
                self.planetarium.level_up(p.hand_rank());
            }
            PackContent::Joker(j) => self.add_acquired_joker(j),
            PackContent::PlayingCard(c) => {
                self.deck.push(c);
            }
            PackContent::Spectral(s) => {
                s.apply(self)?;
            }
        }

        if let Some(ref mut state) = self.open_pack {
            state.picks_remaining = state.picks_remaining.saturating_sub(1);
        }

        let done = self
            .open_pack
            .as_ref()
            .is_some_and(|s| s.picks_remaining == 0);
        if done {
            self.finish_pack();
        }
        Ok(())
    }

    pub(crate) fn targeting_pack(&self) -> bool {
        match self.stage {
            Stage::TarotHand(_) => self.tarot_prev_stage == Some(Stage::PackOpen()),
            Stage::SpectralHand(_) => self.spectral_prev_stage == Some(Stage::PackOpen()),
            _ => false,
        }
    }

    pub(crate) fn skip_pack(&mut self) -> Result<(), GameError> {
        if self.stage != Stage::PackOpen() && !self.targeting_pack() {
            return Err(GameError::InvalidStage);
        }
        self.tarot_prev_stage = None;
        self.spectral_prev_stage = None;
        self.finish_pack();
        Ok(())
    }

    fn finish_pack(&mut self) {
        // Return any drawn hand (Arcana packs draw cards for tarot targeting)
        if !self.available.cards().is_empty() {
            let cards = self.available.cards();
            self.available.empty();
            self.deck.extend(cards);
            self.backend.shuffle_deck(&mut self.deck);
        }
        self.open_pack = None;
        if std::mem::take(&mut self.pack_returns_to_blinds) {
            self.stage = Stage::PreBlind();
            self.resolve_blind_choice_tags();
        } else {
            self.stage = Stage::Shop();
        }
    }

    fn select_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        // can only set blind if stage is pre blind
        if self.stage != Stage::PreBlind() {
            return Err(GameError::InvalidStage);
        }
        // provided blind must be expected next blind
        if let Some(current) = self.blind {
            if blind != current.next() {
                return Err(GameError::InvalidBlind);
            }
        } else {
            // if game just started, blind will be None, in which case
            // we can only set it to small.
            if blind != Blind::Small {
                return Err(GameError::InvalidBlind);
            }
        }
        self.blind = Some(blind);
        self.stage = Stage::Blind(blind);
        // Per-round resources are set here, at the start of the round, not
        // only in `clear_blind` at the end of the previous one — anything
        // bought in the shop in between (Grabber, Wasteful, Hieroglyph...)
        // has to count for the round it was bought for.
        self.plays = self.plays_per_round();
        self.discards_remaining = self.discards_per_round();
        while self.take_tag(Tag::Juggle) {
            self.temporary_hand_size += 3;
        }
        self.deal();
        Ok(())
    }

    fn skip_blind(&mut self, blind: Blind) -> Result<(), GameError> {
        if self.stage != Stage::PreBlind() {
            return Err(GameError::InvalidStage);
        }
        if blind == Blind::Boss {
            return Err(GameError::InvalidBlind);
        }
        if let Some(current) = self.blind {
            if blind != current.next() {
                return Err(GameError::InvalidBlind);
            }
        } else if blind != Blind::Small {
            return Err(GameError::InvalidBlind);
        }
        self.blind = Some(blind);
        self.stage = Stage::PreBlind();
        let tag = self
            .skip_tag(blind)
            .expect("blind is not Boss, checked above");
        self.skipped_count += 1;
        let orbital = self.orbital_targets[usize::from(blind == Blind::Big)];
        self.acquire_tag(tag, orbital);
        self.resolve_blind_choice_tags();
        Ok(())
    }

    fn draw_ante_tags(&mut self) {
        let (small, big) = self.backend.draw_ante_tags();
        self.small_blind_tag = small;
        self.big_blind_tag = big;
        self.orbital_targets = [small, big].map(|tag| {
            if tag != Tag::Orbital {
                return None;
            }
            match self.backend.roll_random_planet(&self.planetarium, &[]) {
                Consumable::Planet(planet) => Some(planet.hand_rank()),
                _ => None,
            }
        });
    }

    fn draw_ante_boss(&mut self) {
        self.current_boss = Some(self.backend.draw_boss(self.ante_current.number() as i32));
    }

    pub fn skip_tag(&self, blind: Blind) -> Option<Tag> {
        match blind {
            Blind::Small => Some(self.small_blind_tag),
            Blind::Big => Some(self.big_blind_tag),
            Blind::Boss => None,
        }
    }

    fn next_round(&mut self) -> Result<(), GameError> {
        self.stage = Stage::PreBlind();
        self.round += 1;
        self.resolve_blind_choice_tags();
        Ok(())
    }

    // Returns true if should clear blind after, false if not.
    fn handle_score(&mut self, score: usize) -> Result<bool, GameError> {
        // can only handle score if stage is blind
        if !self.stage.is_blind() {
            return Err(GameError::InvalidStage);
        }

        self.score += score;
        let required = self.required_score();

        // blind not passed
        if self.score < required {
            // no more hands to play -> lose
            if self.plays == 0 {
                self.stage = Stage::End(End::Lose);
                return Ok(false);
            } else {
                // more hands to play, carry on
                return Ok(false);
            }
        }

        let blind = self.blind.expect("stage is blind");
        self.last_score = self.score;

        // score exceeds blind (blind passed).
        // handle reward then progress to next stage.
        let reward = self.calc_reward(blind)?;
        self.unused_discards += self.discards_remaining;
        self.reward = reward;
        if blind == Blind::Boss {
            while self.take_tag(Tag::Investment) {
                self.reward += 25;
            }
        }

        if blind == Blind::Boss {
            let next = if self.endless_mode {
                self.ante_current.next_supported()
            } else {
                self.ante_current.next(self.ante_end)
            };
            if let Some(ante) = next {
                self.advance_ante(ante);
            } else {
                self.campaign_won |= self.ante_current == Ante::Eight;
                self.stage = Stage::End(End::Win);
                return Ok(true);
            }
        }

        // finish blind, proceed to post blind
        self.stage = Stage::PostBlind();
        Ok(true)
    }

    fn advance_ante(&mut self, ante: Ante) {
        self.ante_current = ante;
        self.boss_rerolled = false;
        self.blind = None;
        self.draw_ante_tags();
        self.draw_ante_boss();
    }

    pub fn can_continue_endless(&self) -> bool {
        self.stage == Stage::End(End::Win)
            && self.campaign_won
            && !self.endless_mode
            && self.ante_current == Ante::Eight
            && self.ante_current.next_supported().is_some()
    }

    /// Debug: scores `amount` as a played hand would, clearing the blind when reached.
    pub fn debug_add_score(&mut self, amount: usize) -> Result<(), GameError> {
        if self.handle_score(amount)? {
            self.handle_round_end();
            self.clear_blind();
        }
        Ok(())
    }

    /// Debug: moves to the next ante and draws its tags and boss.
    pub fn debug_next_ante(&mut self) -> Result<(), GameError> {
        let next = self
            .ante_current
            .next_supported()
            .ok_or(GameError::InvalidAction)?;
        self.advance_ante(next);
        Ok(())
    }
    pub fn continue_endless(&mut self) -> Result<(), GameError> {
        if !self.can_continue_endless() {
            return Err(GameError::InvalidAction);
        }
        let next = self
            .ante_current
            .next_supported()
            .ok_or(GameError::InvalidAction)?;
        self.endless_mode = true;
        self.advance_ante(next);
        self.stage = Stage::PostBlind();
        Ok(())
    }

    pub fn handle_action(&mut self, action: Action) -> Result<(), GameError> {
        self.handle_action_inner(action, None)
    }

    pub fn handle_action_traced(
        &mut self,
        action: Action,
        profile_trace: bool,
    ) -> Result<(ScoreTrace, std::time::Duration), GameError> {
        let mut recorder = TraceRecorder {
            profile: profile_trace,
            ..TraceRecorder::default()
        };
        self.handle_action_inner(action, Some(&mut recorder))?;
        Ok((recorder.trace, recorder.elapsed))
    }

    fn handle_action_inner(
        &mut self,
        action: Action,
        trace: Option<&mut TraceRecorder>,
    ) -> Result<(), GameError> {
        self.action_history.push(action.clone());
        match action {
            Action::SelectCard(card) => {
                if self.stage.is_blind() {
                    self.select_card(card)
                } else if let Stage::TarotHand(t) = self.stage {
                    if self.available.selected().len() >= t.max_targets() {
                        return Err(GameError::InvalidAction);
                    }
                    self.select_card(card)
                } else if let Stage::SpectralHand(s) = self.stage {
                    if self.available.selected().len() >= s.max_targets() {
                        return Err(GameError::InvalidAction);
                    }
                    self.select_card(card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::DeselectCard(card) => {
                if self.stage.is_blind()
                    || matches!(self.stage, Stage::TarotHand(_) | Stage::SpectralHand(_))
                {
                    self.available.deselect_card(card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::Play() => match self.stage.is_blind() {
                true => self.play_selected_inner(trace),
                false => Err(GameError::InvalidAction),
            },
            Action::Discard() => match self.stage.is_blind() {
                true => self.discard_selected(),
                false => Err(GameError::InvalidAction),
            },
            Action::MoveCard(dir, card) => {
                if self.stage.is_blind()
                    || matches!(self.stage, Stage::TarotHand(_) | Stage::SpectralHand(_))
                {
                    self.move_card(dir, card)
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::CashOut(_reward) => match self.stage {
                Stage::PostBlind() => self.cashout(),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyJoker(joker) => match self.stage {
                Stage::Shop() => self.buy_joker(joker),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyConsumable(consumable) => match self.stage {
                Stage::Shop() => self.buy_consumable(consumable),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyVoucher(voucher) => match self.stage {
                Stage::Shop() => self.buy_voucher(voucher),
                _ => Err(GameError::InvalidAction),
            },
            Action::BuyPlayingCard(card) => match self.stage {
                Stage::Shop() => self.buy_playing_card(card),
                _ => Err(GameError::InvalidAction),
            },
            Action::UseConsumable(consumable) => self.use_consumable(consumable),
            Action::NextRound() => match self.stage {
                Stage::Shop() => self.next_round(),
                _ => Err(GameError::InvalidAction),
            },
            Action::SelectBlind(blind) => match self.stage {
                Stage::PreBlind() => self.select_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
            Action::SkipBlind(blind) => match self.stage {
                Stage::PreBlind() => self.skip_blind(blind),
                _ => Err(GameError::InvalidAction),
            },
            Action::ApplyTarot() => self.apply_tarot(),
            Action::ApplySpectral() => self.apply_spectral(),
            Action::SellJoker(idx) => self.sell_joker(idx),
            Action::SellConsumable(idx) => self.sell_consumable(idx),
            Action::BuyPack(pack) => match self.stage {
                Stage::Shop() => self.buy_pack(pack),
                _ => Err(GameError::InvalidStage),
            },
            Action::PickPackCard(content) => match self.stage {
                Stage::PackOpen() => self.pick_pack_card(content),
                _ => Err(GameError::InvalidStage),
            },
            Action::SortHand(sort_by) => {
                if self.stage.is_blind() {
                    self.available.sort(sort_by);
                    Ok(())
                } else {
                    Err(GameError::InvalidAction)
                }
            }
            Action::SkipPack() => self.skip_pack(),
            Action::RerollBoss() => {
                if !self.boss_reroll_available() {
                    return Err(GameError::InvalidAction);
                }
                self.money -= 10;
                self.replace_boss();
                Ok(())
            }
            Action::Reroll() => match self.stage {
                Stage::Shop() => self.reroll(),
                _ => Err(GameError::InvalidAction),
            },
        }
    }

    pub fn handle_action_index(&mut self, index: usize) -> Result<(), GameError> {
        let space = self.gen_action_space();
        let action = space.to_action(index, self)?;
        self.handle_action(action)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hand_str: String = self
            .available
            .cards_and_selected()
            .iter()
            .map(|(c, sel)| {
                if *sel {
                    format!("[{}]", card_display(c))
                } else {
                    card_display(c)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(f, "hand: {}", hand_str)?;
        writeln!(f, "discard pile: {}", self.discarded.len())?;
        let deck_cards = self.deck.cards();
        let modified: Vec<String> = deck_cards
            .iter()
            .filter(|c| c.enhancement.is_some() || c.edition != Edition::Base || c.seal.is_some())
            .map(card_display)
            .collect();
        if modified.is_empty() {
            writeln!(f, "deck: {}", deck_cards.len())?;
        } else {
            writeln!(
                f,
                "deck: {} ({} modified: {})",
                deck_cards.len(),
                modified.len(),
                modified.join(" ")
            )?;
        }
        if self.jokers.is_empty() {
            writeln!(f, "jokers: (none)")?;
        } else {
            writeln!(f, "jokers:")?;
            for j in self.jokers.clone() {
                writeln!(f, "  {}", joker_display(&j))?;
            }
        }
        if self.consumables.is_empty() {
            writeln!(f, "consumables: (none)")?;
        } else {
            let parts: Vec<String> = self
                .consumables
                .iter()
                .map(|c| format!("[{}] {}", c.type_label(), c.name()))
                .collect();
            writeln!(f, "consumables: {}", parts.join(", "))?;
        }
        if self.vouchers.is_empty() {
            writeln!(f, "vouchers: (none)")?;
        } else {
            let names: Vec<&str> = self.vouchers.owned().iter().map(|v| v.name()).collect();
            writeln!(f, "vouchers: {}", names.join(", "))?;
        }
        writeln!(f, "planetarium: {}", self.planetarium)?;
        writeln!(f, "stage: {:?}", self.stage)?;
        writeln!(f, "ante: {:?}", self.ante_current)?;
        writeln!(f, "blind: {:?}", self.blind)?;
        writeln!(f, "round: {}", self.round)?;
        writeln!(f, "hands remaining: {}", self.plays)?;
        writeln!(f, "discards remaining: {}", self.discards())?;
        writeln!(f, "money: {}", self.money)?;
        if matches!(self.stage, Stage::Blind(_)) {
            writeln!(
                f,
                "score: {}  target: {}",
                self.score,
                self.required_score()
            )
        } else {
            writeln!(f, "score: {}", self.score)
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(feature = "serde")]
impl Game {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        #[derive(serde::Deserialize)]
        struct TagFields {
            skipped_count: Option<usize>,
        }
        let legacy_tags = serde_json::from_str::<TagFields>(s)?
            .skipped_count
            .is_none();
        let mut game: Self = serde_json::from_str(s)?;
        for ante in [game.ante_start, game.ante_end, game.ante_current] {
            if Ante::try_from(ante.number()) != Ok(ante) {
                return Err(serde::de::Error::custom("invalid ante in snapshot"));
            }
        }
        let jokers = game.jokers.clone();
        game.effect_registry.register_jokers(jokers, &game.clone());

        // Deserializing doesn't allocate ids, so Card::new() calls after this
        // point would otherwise collide with ids already present in `game`.
        let pack_contents = game
            .shop
            .packs
            .iter()
            .flat_map(|pack| &pack.contents)
            .chain(game.open_pack.iter().flat_map(|pack| &pack.contents));
        let max_id = game
            .deck
            .cards()
            .iter()
            .chain(game.available.cards().iter())
            .chain(game.discarded.iter())
            .chain(game.shop.cards.iter())
            .chain(pack_contents.clone().filter_map(|content| match content {
                PackContent::PlayingCard(card) => Some(card),
                _ => None,
            }))
            .map(|c| c.id)
            .max();
        if let Some(max_id) = max_id {
            crate::card::ensure_id_counter_past(max_id);
        }

        // Same fixup for joker instance ids, minted ones sit in `game.jokers`
        // and `game.shop.jokers`, and 0 means "never minted" so it's excluded.
        let max_joker_id = game
            .jokers
            .iter()
            .chain(game.shop.jokers.iter())
            .chain(pack_contents.filter_map(|content| match content {
                PackContent::Joker(joker) => Some(joker),
                _ => None,
            }))
            .map(|j| j.instance_id())
            .filter(|&id| id > 0)
            .max();
        if game.current_boss.is_none() {
            game.draw_ante_boss();
        }

        if let Some(max_joker_id) = max_joker_id {
            crate::joker::ensure_joker_id_counter_past(max_joker_id);
        }

        if legacy_tags {
            game.skipped_count = game
                .action_history
                .iter()
                .filter(|action| matches!(action, Action::SkipBlind(_)))
                .count();
            for tag in std::mem::take(&mut game.tags) {
                game.acquire_tag(tag, None);
            }
            if game.stage == Stage::PreBlind() {
                game.resolve_blind_choice_tags();
            }
        }

        Ok(game)
    }
}

#[cfg(test)]
mod tests;
