use crate::card::Card;
use crate::consumable::Consumable;
use crate::deck::Deck;
use crate::joker::Jokers;
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::{Planetarium, Planets};
use crate::shop::{
    gen_random_playing_card, gen_shop_playing_card, ConsumableGenerator, JokerGenerator,
    PackGenerator, ShopContext,
};
use crate::tag::Tag;
use crate::tarot::Tarot;
use crate::voucher::{Voucher, Vouchers};
use balatro_seed::Instance;
use balatro_types::{BossBlind, Edition, Rarity, Suit, Value};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use strum::IntoEnumIterator;

mod backend;
mod real;

pub(crate) use backend::*;
pub(crate) use real::*;

/// What a single shop-item generation call produced. The category split
/// has to happen inside the backend, since `Real` mode's category roll is
/// bundled into one `Instance::next_shop_item` call.
pub(crate) enum GeneratedItem {
    Joker(Jokers),
    Consumable(Consumable),
    /// Only ever produced with the Magic Trick voucher redeemed.
    PlayingCard(Card),
}

pub(crate) trait RngBackend {
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem;

    fn gen_pack(&mut self, ctx: &ShopContext, exclude: Option<(&PackCategory, &PackSize)>) -> Pack;

    /// This ante's voucher offer, drawn from the vouchers still
    /// offerable (see `Vouchers::offerable`). `None` once all 32 are
    /// redeemed.
    fn gen_voucher(&mut self, ante: i32, vouchers: &Vouchers) -> Option<Voucher>;

    /// Owned-joker dedup hook: called on buy/sell so `Real` mode's lock
    /// table stays accurate. No-op for `Fast` mode.
    fn on_joker_bought(&mut self, joker: &Jokers);
    fn on_joker_sold(&mut self, joker: &Jokers);
    /// Redeemed-voucher hook: `Real` mode's draw rates read straight off
    /// its own active-voucher list, so it has to be told. No-op for
    /// `Fast` mode, which reads `ShopContext::vouchers` per call instead.
    fn on_voucher_bought(&mut self, voucher: &Voucher);
    /// Jokers::Showman's real effect. No call site yet — see `jokers.md`.
    #[allow(dead_code)]
    fn set_showman(&mut self, owned: bool);
    fn set_gros_michel_extinct(&mut self, extinct: bool);
    /// A uniform index below `len` (`pseudorandom_element` over a list).
    fn roll_index(&mut self, len: usize) -> usize;
    /// Joker ids the generators leave out.
    fn set_locked(&mut self, ids: Vec<String>);
    /// A uniform number in [0, 1) (`pseudorandom`).
    fn roll_unit(&mut self) -> f64;

    /// A single freshly generated joker, outside the shop/pack context —
    /// currently only Judgement. Distinct from `gen_shop_item`'s joker arm
    /// since a shop slot's category (joker vs. consumable) is its own roll.
    fn gen_joker(&mut self, ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers;
    fn gen_joker_of_rarity(
        &mut self,
        _ante: i32,
        prob_mult: u32,
        exclude: &[Jokers],
        rarity: Rarity,
    ) -> Jokers;
    fn pick_random_joker(&mut self, available: Vec<Jokers>) -> Jokers;
    fn clone_joker(&mut self, j: Jokers) -> Jokers;
    fn shuffle_deck(&mut self, deck: &mut Deck);
    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool;
    /// Returns `(small_blind_tag, big_blind_tag)` for a fresh ante.
    fn draw_ante_tags(&mut self) -> (Tag, Tag);
    /// Returns the Boss Blind for a fresh ante.
    fn draw_boss(&mut self, ante: i32) -> BossBlind;
    /// A single random Tarot consumable, respecting `exclude` — used by
    /// the purple-seal discard trigger and the Emperor/High Priestess
    /// tarot effects.
    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable;
    /// A single random Planet consumable, respecting `exclude` — used by
    /// the blue-seal round-end trigger and the High Priestess tarot effect.
    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable;
    /// Castle/MailInRebate's per-round reroll (`Game::clear_blind`). Mint-time
    /// rolling for freshly generated jokers already happens inside
    /// `gen_joker`/`gen_shop_item`/`gen_pack`, each already using the right
    /// stream per backend — this is only for rerolling an *existing* joker.
    fn roll_discard_selector(&mut self, j: &mut Jokers);
    fn roll_random_edition(&mut self) -> Edition;
    fn roll_random_suit(&mut self) -> Suit;
    fn roll_random_value(&mut self) -> Value;
    fn pick_random_card(&mut self, available: Vec<Card>) -> Card;
    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card;
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct FastBackend {
    rng: ChaCha8Rng,
    joker_gen: JokerGenerator,
    consumable_gen: ConsumableGenerator,
    pack_gen: PackGenerator,
}

impl FastBackend {
    pub(crate) fn new(rng: ChaCha8Rng) -> Self {
        FastBackend {
            rng,
            joker_gen: JokerGenerator::new(),
            consumable_gen: ConsumableGenerator::new(),
            pack_gen: PackGenerator {
                gros_michel_extinct: false,
                locked: Vec::new(),
            },
        }
    }
}

impl RngBackend for FastBackend {
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem {
        // Joker=20, Tarot/Planet=4 each before the Merchant/Tycoon
        // vouchers raise them; playing cards weigh 0 until Magic Trick.
        let weights = [
            20.0,
            ctx.vouchers.tarot_rate(),
            ctx.vouchers.planet_rate(),
            ctx.vouchers.playing_card_rate(),
            ctx.spectral_rate,
        ];
        let dist = WeightedIndex::new(weights).expect("shop item weights are finite and nonzero");
        match dist.sample(&mut self.rng) {
            0 => GeneratedItem::Joker(self.joker_gen.gen_joker(
                ctx.edition_prob_mult(),
                exclude_jokers,
                &mut self.rng,
            )),
            1 => GeneratedItem::Consumable(
                self.consumable_gen
                    .gen_tarot_consumable(exclude_tarots, &mut self.rng),
            ),
            2 => GeneratedItem::Consumable(self.consumable_gen.gen_planet_consumable(
                ctx.planetarium,
                exclude_planets,
                &mut self.rng,
            )),
            4 => GeneratedItem::Consumable(Consumable::Spectral(crate::spectral::random_spectral(
                &mut self.rng,
            ))),
            _ => GeneratedItem::PlayingCard(gen_shop_playing_card(
                ctx.edition_prob_mult(),
                ctx.vouchers.shop_cards_are_modified(),
                &mut self.rng,
            )),
        }
    }

    fn gen_pack(&mut self, ctx: &ShopContext, exclude: Option<(&PackCategory, &PackSize)>) -> Pack {
        self.pack_gen.gen_pack(
            ctx.planetarium,
            ctx.edition_prob_mult(),
            exclude,
            ctx.held_jokers,
            &mut self.rng,
        )
    }

    fn gen_voucher(&mut self, _ante: i32, vouchers: &Vouchers) -> Option<Voucher> {
        let offerable = vouchers.offerable();
        if offerable.is_empty() {
            return None;
        }
        Some(offerable[self.rng.gen_range(0..offerable.len())])
    }

    fn on_joker_bought(&mut self, _joker: &Jokers) {}
    fn on_joker_sold(&mut self, _joker: &Jokers) {}
    fn on_voucher_bought(&mut self, _voucher: &Voucher) {}
    fn set_showman(&mut self, _owned: bool) {}

    fn roll_index(&mut self, len: usize) -> usize {
        self.rng.gen_range(0..len.max(1))
    }

    fn roll_unit(&mut self) -> f64 {
        self.rng.gen::<f64>()
    }

    fn set_locked(&mut self, ids: Vec<String>) {
        self.joker_gen.locked = ids.clone();
        self.pack_gen.locked = ids;
    }

    fn set_gros_michel_extinct(&mut self, extinct: bool) {
        self.joker_gen.gros_michel_extinct = extinct;
        self.pack_gen.gros_michel_extinct = extinct;
    }

    fn gen_joker(&mut self, _ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers {
        self.joker_gen.gen_joker(prob_mult, exclude, &mut self.rng)
    }

    fn gen_joker_of_rarity(
        &mut self,
        _ante: i32,
        prob_mult: u32,
        exclude: &[Jokers],
        rarity: Rarity,
    ) -> Jokers {
        self.joker_gen
            .gen_joker_of_rarity(prob_mult, exclude, rarity, &mut self.rng)
    }

    fn pick_random_joker(&mut self, available: Vec<Jokers>) -> Jokers {
        let idx = self.rng.gen_range(0..available.len());
        available[idx].clone()
    }

    fn clone_joker(&mut self, j: Jokers) -> Jokers {
        self.joker_gen.clone_joker(j, &mut self.rng)
    }

    fn shuffle_deck(&mut self, deck: &mut Deck) {
        deck.shuffle(&mut self.rng);
    }

    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        self.rng.gen_ratio(numerator.min(denominator), denominator)
    }

    fn draw_ante_tags(&mut self) -> (Tag, Tag) {
        let tags: Vec<Tag> = Tag::iter().collect();
        let small = *tags.choose(&mut self.rng).unwrap();
        let big = *tags.choose(&mut self.rng).unwrap();
        (small, big)
    }

    fn draw_boss(&mut self, ante: i32) -> BossBlind {
        let pool: Vec<BossBlind> = BossBlind::iter()
            .filter(|b| b.is_finisher() == (ante % 8 == 0))
            .filter(|b| ante >= b.min_ante())
            .collect();
        *pool.choose(&mut self.rng).unwrap()
    }

    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable {
        self.consumable_gen
            .gen_tarot_consumable(exclude, &mut self.rng)
    }

    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable {
        self.consumable_gen
            .gen_planet_consumable(planetarium, exclude, &mut self.rng)
    }

    fn roll_discard_selector(&mut self, j: &mut Jokers) {
        crate::joker::roll_discard_selector(&mut self.rng, j);
    }

    fn roll_random_edition(&mut self) -> Edition {
        const EDITIONS: [Edition; 3] = [Edition::Foil, Edition::Holographic, Edition::Polychrome];
        EDITIONS[self.rng.gen_range(0..EDITIONS.len())]
    }

    fn roll_random_suit(&mut self) -> Suit {
        let suits: Vec<Suit> = Suit::iter().collect();
        suits[self.rng.gen_range(0..suits.len())]
    }

    fn roll_random_value(&mut self) -> Value {
        let values: Vec<Value> = Value::iter().collect();
        values[self.rng.gen_range(0..values.len())]
    }

    fn pick_random_card(&mut self, available: Vec<Card>) -> Card {
        available[self.rng.gen_range(0..available.len())]
    }

    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card {
        gen_random_playing_card(prob_mult, &mut self.rng, force_enhance, force_values)
    }
}

#[cfg(test)]
mod tests;
