use crate::action::Action;
use crate::card::{Card, Edition, Enhancement, Seal, Suit, Value};
use crate::consumable::Consumable;
use crate::error::GameError;
use crate::joker::{jokers_by_rarity, Jokers, Rarity};
use crate::pack::{Pack, PackCategory, PackContent, PackSize};
use crate::planet::{Planetarium, Planets};
use crate::rank::HandRank;
use crate::rng::{Backend, GeneratedItem, RngBackend};
use crate::tarot::Tarot;
use crate::voucher::{Voucher, Vouchers};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use strum::IntoEnumIterator;

mod generators;

pub(crate) use generators::*;

/// Everything shop generation reads off the `Game`. Bundled into one
/// struct because vouchers made the parameter list unreadable — every
/// field here was already being threaded through positionally.
pub(crate) struct ShopContext<'a> {
    pub planetarium: &'a Planetarium,
    /// Consumables the player already holds — excluded from the roll so
    /// the shop doesn't offer a duplicate.
    pub held_consumables: &'a [Consumable],
    pub held_jokers: &'a [Jokers],
    pub vouchers: &'a Vouchers,
    pub prob_mult: u32,
    pub ante: i32,
    /// Which hand Telescope should guarantee a Planet card for.
    pub most_played: HandRank,
}

impl ShopContext<'_> {
    /// Card slots the shop fills, before packs: 2 by default, more with
    /// Overstock.
    pub(crate) fn card_slots(&self) -> usize {
        DEFAULT_SHOP_CARD_SLOTS + self.vouchers.shop_card_slots_bonus()
    }

    /// Edition probabilities are scaled by both the game-wide probability
    /// multiplier and Hone/Glow Up.
    pub(crate) fn edition_prob_mult(&self) -> u32 {
        self.prob_mult
            .saturating_mul(self.vouchers.edition_rate_mult())
    }
}

#[cfg(test)]
impl<'a> ShopContext<'a> {
    /// Bare context for tests: nothing held, no vouchers redeemed.
    pub(crate) fn for_test(
        planetarium: &'a Planetarium,
        vouchers: &'a Vouchers,
        ante: i32,
    ) -> Self {
        ShopContext {
            planetarium,
            held_consumables: &[],
            held_jokers: &[],
            vouchers,
            prob_mult: 1,
            ante,
            most_played: HandRank::HighCard,
        }
    }
}

const DEFAULT_SHOP_CARD_SLOTS: usize = 2;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct Shop {
    pub jokers: Vec<Jokers>,
    pub consumables: Vec<Consumable>,
    /// Playing cards for sale. Always empty until Magic Trick is redeemed.
    #[cfg_attr(feature = "serde", serde(default))]
    pub cards: Vec<Card>,
    pub packs: Vec<Pack>,
    /// This ante's voucher offer, if it hasn't been bought yet. Unlike
    /// everything else in the shop it survives rerolls and persists
    /// across every shop visit of the ante.
    #[cfg_attr(feature = "serde", serde(default))]
    pub voucher: Option<Voucher>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra_vouchers: Vec<Voucher>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) free_jokers: Vec<usize>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) free_cards: Vec<usize>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) free_consumables: Vec<Consumable>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) free_packs: bool,
}

impl Shop {
    pub fn voucher_offers(&self) -> impl Iterator<Item = Voucher> + '_ {
        self.voucher
            .into_iter()
            .chain(self.extra_vouchers.iter().copied())
    }

    pub fn joker_price(&self, joker: &Jokers, vouchers: &Vouchers) -> usize {
        if self.free_jokers.contains(&joker.instance_id()) {
            0
        } else {
            vouchers.discounted(joker.cost())
        }
    }

    pub fn consumable_price(&self, card: &Consumable, vouchers: &Vouchers) -> usize {
        if self.free_consumables.contains(card) {
            0
        } else {
            vouchers.discounted(card.cost())
        }
    }

    pub fn card_price(&self, card: &Card, vouchers: &Vouchers) -> usize {
        if self.free_cards.contains(&card.id) {
            0
        } else {
            vouchers.discounted(card.shop_cost())
        }
    }

    pub fn pack_price(&self, pack: &Pack, vouchers: &Vouchers) -> usize {
        if self.free_packs {
            0
        } else {
            vouchers.discounted(pack.cost())
        }
    }

    pub(crate) fn coupon(&mut self) {
        self.free_jokers = self.jokers.iter().map(Jokers::instance_id).collect();
        self.free_cards = self.cards.iter().map(|card| card.id).collect();
        self.free_consumables.clone_from(&self.consumables);
        self.free_packs = true;
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl Shop {
    pub(crate) fn refresh_cards(&mut self, ctx: &ShopContext, backend: &mut Backend) {
        self.free_jokers.clear();
        self.free_cards.clear();
        self.free_consumables.clear();
        self.jokers.clear();
        self.consumables.clear();
        self.cards.clear();

        let mut excl_jokers: Vec<Jokers> = ctx.held_jokers.to_vec();
        let mut excl_tarots: Vec<Tarot> = ctx
            .held_consumables
            .iter()
            .filter_map(|c| {
                if let Consumable::Tarot(t) = c {
                    Some(*t)
                } else {
                    None
                }
            })
            .collect();
        let mut excl_planets: Vec<Planets> = ctx
            .held_consumables
            .iter()
            .filter_map(|c| {
                if let Consumable::Planet(p) = c {
                    Some(*p)
                } else {
                    None
                }
            })
            .collect();

        for _ in 0..ctx.card_slots() {
            match backend.gen_shop_item(ctx, &excl_jokers, &excl_tarots, &excl_planets) {
                GeneratedItem::Joker(joker) => {
                    excl_jokers.push(joker.clone());
                    self.jokers.push(joker);
                }
                GeneratedItem::Consumable(c) => {
                    match &c {
                        Consumable::Tarot(t) => excl_tarots.push(*t),
                        Consumable::Planet(p) => excl_planets.push(*p),
                        Consumable::Spectral(_) => {}
                    }
                    self.consumables.push(c);
                }
                GeneratedItem::PlayingCard(card) => self.cards.push(card),
            }
        }
    }

    /// Fills any card slots the shop has gained since it was stocked,
    /// leaving what's already on the shelf alone. Redeeming Overstock
    /// mid-shop is the only thing that widens a live shop.
    pub(crate) fn top_up_cards(&mut self, ctx: &ShopContext, backend: &mut Backend) {
        let filled = self.jokers.len() + self.consumables.len() + self.cards.len();
        let mut excl_jokers: Vec<Jokers> = ctx.held_jokers.to_vec();
        excl_jokers.extend(self.jokers.clone());
        let mut excl_tarots: Vec<Tarot> = Vec::new();
        let mut excl_planets: Vec<Planets> = Vec::new();
        for c in ctx.held_consumables.iter().chain(self.consumables.iter()) {
            match c {
                Consumable::Tarot(t) => excl_tarots.push(*t),
                Consumable::Planet(p) => excl_planets.push(*p),
                Consumable::Spectral(_) => {}
            }
        }

        for _ in filled..ctx.card_slots() {
            match backend.gen_shop_item(ctx, &excl_jokers, &excl_tarots, &excl_planets) {
                GeneratedItem::Joker(joker) => {
                    excl_jokers.push(joker.clone());
                    self.jokers.push(joker);
                }
                GeneratedItem::Consumable(c) => {
                    match &c {
                        Consumable::Tarot(t) => excl_tarots.push(*t),
                        Consumable::Planet(p) => excl_planets.push(*p),
                        Consumable::Spectral(_) => {}
                    }
                    self.consumables.push(c);
                }
                GeneratedItem::PlayingCard(card) => self.cards.push(card),
            }
        }
    }

    pub(crate) fn refresh(&mut self, ctx: &ShopContext, backend: &mut Backend) {
        self.free_packs = false;
        self.refresh_cards(ctx, backend);

        let p1 = backend.gen_pack(ctx, None);
        let exclude = (&p1.category, &p1.size);
        let p2 = backend.gen_pack(ctx, Some(exclude));
        self.packs = vec![p1, p2];
        for pack in &mut self.packs {
            apply_telescope(pack, ctx);
        }
    }

    pub(crate) fn joker_from_index(&self, i: usize) -> Option<Jokers> {
        Some(self.jokers[i].clone())
    }

    pub(crate) fn consumable_from_index(&self, i: usize) -> Option<Consumable> {
        self.consumables.get(i).cloned()
    }

    pub(crate) fn pack_from_index(&self, i: usize) -> Option<Pack> {
        self.packs.get(i).cloned()
    }

    pub(crate) fn card_from_index(&self, i: usize) -> Option<Card> {
        self.cards.get(i).copied()
    }

    pub(crate) fn buy_joker(&mut self, joker: &Jokers) -> Result<Jokers, GameError> {
        let i = self
            .jokers
            .iter()
            .position(|j| j == joker)
            .ok_or(GameError::NoJokerMatch)?;
        Ok(self.jokers.remove(i))
    }

    pub(crate) fn buy_consumable(
        &mut self,
        consumable: &Consumable,
    ) -> Result<Consumable, GameError> {
        let i = self
            .consumables
            .iter()
            .position(|c| c == consumable)
            .ok_or(GameError::NoConsumableMatch)?;
        Ok(self.consumables.remove(i))
    }

    pub(crate) fn buy_pack(&mut self, pack: &Pack) -> Result<Pack, GameError> {
        let i = self
            .packs
            .iter()
            .position(|p| p == pack)
            .ok_or(GameError::InvalidAction)?;
        Ok(self.packs.remove(i))
    }

    pub(crate) fn buy_card(&mut self, card: &Card) -> Result<Card, GameError> {
        let i = self
            .cards
            .iter()
            .position(|c| c.id == card.id)
            .ok_or(GameError::NoCardMatch)?;
        Ok(self.cards.remove(i))
    }

    /// Takes the offered voucher off the shelf. Unlike the other buys it
    /// clears the slot rather than removing from a list — there is only
    /// ever one voucher on offer per ante.
    pub(crate) fn buy_voucher(&mut self, voucher: &Voucher) -> Result<Voucher, GameError> {
        match self.voucher {
            Some(v) if v == *voucher => {
                self.voucher = None;
                Ok(v)
            }
            _ => {
                let index = self
                    .extra_vouchers
                    .iter()
                    .position(|v| v == voucher)
                    .ok_or(GameError::NoVoucherMatch)?;
                Ok(self.extra_vouchers.remove(index))
            }
        }
    }

    pub(crate) fn gen_moves_buy_joker(
        &self,
        balance: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.jokers.is_empty() {
            return None;
        }
        let buys = self
            .jokers
            .clone()
            .into_iter()
            .filter(move |j| self.joker_price(j, vouchers) <= balance)
            .map(Action::BuyJoker)
            .collect::<Vec<_>>();
        Some(buys.into_iter())
    }

    pub(crate) fn gen_moves_buy_consumable(
        &self,
        balance: usize,
        consumable_slots: usize,
        held: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.consumables.is_empty() || held >= consumable_slots {
            return None;
        }
        let buys = self
            .consumables
            .clone()
            .into_iter()
            .filter(move |c| self.consumable_price(c, vouchers) <= balance)
            .map(Action::BuyConsumable)
            .collect::<Vec<_>>();
        Some(buys.into_iter())
    }

    pub(crate) fn gen_moves_buy_pack(
        &self,
        balance: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.packs.is_empty() {
            return None;
        }
        let buys = self
            .packs
            .clone()
            .into_iter()
            .filter(move |p| self.pack_price(p, vouchers) <= balance)
            .map(Action::BuyPack)
            .collect::<Vec<_>>();
        Some(buys.into_iter())
    }

    pub(crate) fn gen_moves_buy_card(
        &self,
        balance: usize,
        vouchers: &Vouchers,
    ) -> Option<impl Iterator<Item = Action>> {
        if self.cards.is_empty() {
            return None;
        }
        let buys = self
            .cards
            .clone()
            .into_iter()
            .filter(move |c| self.card_price(c, vouchers) <= balance)
            .map(Action::BuyPlayingCard)
            .collect::<Vec<_>>();
        Some(buys.into_iter())
    }

    pub(crate) fn gen_moves_buy_voucher(
        &self,
        balance: usize,
    ) -> Option<impl Iterator<Item = Action>> {
        // Vouchers are never discounted, so no `Vouchers` argument here.
        let buys: Vec<_> = self
            .voucher_offers()
            .filter(|v| v.cost() <= balance)
            .map(Action::BuyVoucher)
            .collect();
        Some(buys.into_iter())
    }
}

#[cfg(test)]
mod tests;
