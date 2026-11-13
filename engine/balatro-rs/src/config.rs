const DEFAULT_ROUND_START: usize = 0;
const DEFAULT_PLAYS: usize = 4;
const DEFAULT_DISCARDS: usize = 4;
const DEFAULT_MONEY_START: usize = 0;
const DEFAULT_MONEY_MAX: usize = 500;
const DEFAULT_REWARD_BASE: usize = 0;
const DEFAULT_MONEY_PER_HAND: usize = 1;
const DEFAULT_INTEREST_RATE: f32 = 0.2;
const DEFAULT_INTEREST_MAX: usize = 5;
const DEFAULT_BASE_MULT: usize = 0;
const DEFAULT_BASE_CHIPS: usize = 0;
const DEFAULT_BASE_SCORE: usize = 0;
const DEFAULT_ANTE_START: usize = 1;
const DEFAULT_ANTE_END: usize = 8;
const DEFAULT_JOKER_SLOTS: usize = 5;
const DEFAULT_JOKER_SLOTS_MAX: usize = 10;
const DEFAULT_AVAILABLE: usize = 8;
const DEFAULT_AVAILABLE_MAX: usize = 24; // arbitrary
const DEFAULT_STORE_CONSUMABLE_SLOTS_MAX: usize = 4;
const DEFAULT_DECK_MAX: usize = 100;
const DEFAULT_DISCARDED_MAX: usize = 100;
const DEFAULT_SELECTED_MAX: usize = 5;
const DEFAULT_CONSUMABLE_SLOTS: usize = 2;

/// Which RNG backend `Shop` generation uses. `Fast` is `core`'s original
/// `rand_chacha`-based generation (unchanged). `Real` is a byte-accurate
/// port of the real Balatro seed algorithm (`balatro-seed`), producing the
/// same shop/pack contents the real game would for a given seed. Only
/// affects shop-item/pack generation — deck shuffling, probability rolls,
/// and the skip-blind tag draw are unaffected either way.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RngMode {
    #[default]
    Fast,
    Real,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Config {
    pub round_start: usize,
    pub plays: usize,
    pub discards: usize,
    pub money_start: usize,
    pub money_max: usize,
    pub reward_base: usize,
    pub money_per_hand: usize,
    pub interest_rate: f32,
    pub interest_max: usize,
    pub base_mult: usize,
    pub base_chips: usize,
    pub base_score: usize,
    pub ante_start: usize,
    pub ante_end: usize,
    pub joker_slots: usize,
    pub joker_slots_max: usize,
    pub selected_max: usize,
    pub available: usize,
    pub available_max: usize,
    pub store_consumable_slots_max: usize,
    pub consumable_slots: usize,
    pub deck_max: usize,
    pub discarded_max: usize,
    pub seed: Option<u64>,
    pub seed_str: Option<String>,
    pub rng_mode: RngMode,
}

impl Config {
    pub fn new() -> Self {
        Config {
            round_start: DEFAULT_ROUND_START,
            plays: DEFAULT_PLAYS,
            discards: DEFAULT_DISCARDS,
            money_start: DEFAULT_MONEY_START,
            money_max: DEFAULT_MONEY_MAX,
            reward_base: DEFAULT_REWARD_BASE,
            money_per_hand: DEFAULT_MONEY_PER_HAND,
            interest_rate: DEFAULT_INTEREST_RATE,
            interest_max: DEFAULT_INTEREST_MAX,
            base_mult: DEFAULT_BASE_MULT,
            base_chips: DEFAULT_BASE_CHIPS,
            base_score: DEFAULT_BASE_SCORE,
            ante_start: DEFAULT_ANTE_START,
            ante_end: DEFAULT_ANTE_END,
            joker_slots: DEFAULT_JOKER_SLOTS,
            joker_slots_max: DEFAULT_JOKER_SLOTS_MAX,
            selected_max: DEFAULT_SELECTED_MAX,
            available: DEFAULT_AVAILABLE,
            available_max: DEFAULT_AVAILABLE_MAX,
            store_consumable_slots_max: DEFAULT_STORE_CONSUMABLE_SLOTS_MAX,
            consumable_slots: DEFAULT_CONSUMABLE_SLOTS,
            deck_max: DEFAULT_DECK_MAX,
            discarded_max: DEFAULT_DISCARDED_MAX,
            seed: None,
            seed_str: None,
            rng_mode: RngMode::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
