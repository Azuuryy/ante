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

mod actions;
mod boss_hand;
mod consumables;
mod display;
mod progression;
mod purchases;
mod round;
mod scoring;
#[cfg(feature = "serde")]
mod snapshot;
mod tags;

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
    /// Rows of `reward`, kept from the cleared Blind until cash out.
    #[cfg_attr(feature = "serde", serde(default))]
    pub round_reward: crate::reward::RoundReward,
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
    // The Mouth: the only hand type allowed this round, the first one played.
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) mouth_hand: Option<HandRank>,
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
            round_reward: crate::reward::RoundReward::default(),
            money: config.money_start,
            chips: config.base_chips,
            mult: config.base_mult as f64,
            score: config.base_score,
            prob_mult: 1,
            hand_ranks_played_this_round: HashSet::new(),
            mouth_hand: None,
            consecutive_hands_without_face_card: 0,
            consecutive_hands_not_most_played_type: 0,
            discarded_this_round: Vec::new(),
            total_cards_discarded: 0,
            boss_disabled_by_luchador: false,
            boss_triggered_this_hand: false,
            last_consumable_used: None,
            last_score: 0,
            reroll_cost: default_reroll_cost(),
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
}

impl Default for Game {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests;
