use super::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
// Both variants are inherently chunky — each owns an rng, and `Real` also
// owns the seed `Instance`'s node and lock tables. Exactly one exists per
// `Game`, so boxing would only trade a few hundred bytes for an
// indirection on every draw.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Backend {
    Fast(FastBackend),
    Real(RealBackend),
}

#[cfg(feature = "ante-extra-jokers")]
impl Backend {
    pub(crate) fn roll_misprint_mult(&mut self) -> usize {
        let rng = match self {
            Self::Fast(backend) => &mut backend.rng,
            Self::Real(backend) => &mut backend.fast.rng,
        };
        rng.gen_range(0..=23)
    }
}

impl Backend {
    pub(crate) fn omen_globe_contents(&mut self, ante: i32, pack: &mut Pack) {
        if pack.category != PackCategory::Arcana {
            return;
        }
        for slot in &mut pack.contents {
            if !self.prob_roll(1, 5) {
                continue;
            }
            match self {
                Self::Fast(fast) => {
                    *slot = PackContent::Spectral(crate::spectral::random_spectral(&mut fast.rng));
                }
                Self::Real(real) => {
                    if let Some(card) = real
                        .gen_pack_contents(ante, PackCategory::Spectral, 1)
                        .pop()
                    {
                        *slot = card;
                    }
                }
            }
        }
    }

    pub(crate) fn tag_pack(
        &mut self,
        category: PackCategory,
        size: PackSize,
        ctx: &ShopContext,
    ) -> Pack {
        let count = balatro_seed::pack_card_count(category, size);
        let contents = match self {
            Self::Fast(fast) => fast.pack_gen.gen_contents(
                &category,
                count as usize,
                ctx.planetarium,
                ctx.edition_prob_mult(),
                ctx.held_jokers,
                &mut fast.rng,
            ),
            Self::Real(real) => real.gen_pack_contents(ctx.ante, category, count as i32),
        };
        let mut pack = Pack {
            category,
            size,
            contents,
        };
        crate::shop::apply_telescope(&mut pack, ctx);
        pack
    }
}

impl RngBackend for Backend {
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        exclude_jokers: &[Jokers],
        exclude_tarots: &[Tarot],
        exclude_planets: &[Planets],
    ) -> GeneratedItem {
        match self {
            Backend::Fast(b) => {
                b.gen_shop_item(ctx, exclude_jokers, exclude_tarots, exclude_planets)
            }
            Backend::Real(b) => {
                b.gen_shop_item(ctx, exclude_jokers, exclude_tarots, exclude_planets)
            }
        }
    }

    fn gen_pack(&mut self, ctx: &ShopContext, exclude: Option<(&PackCategory, &PackSize)>) -> Pack {
        match self {
            Backend::Fast(b) => b.gen_pack(ctx, exclude),
            Backend::Real(b) => b.gen_pack(ctx, exclude),
        }
    }

    fn gen_voucher(&mut self, ante: i32, vouchers: &Vouchers) -> Option<Voucher> {
        match self {
            Backend::Fast(b) => b.gen_voucher(ante, vouchers),
            Backend::Real(b) => b.gen_voucher(ante, vouchers),
        }
    }

    fn on_joker_bought(&mut self, joker: &Jokers) {
        match self {
            Backend::Fast(b) => b.on_joker_bought(joker),
            Backend::Real(b) => b.on_joker_bought(joker),
        }
    }

    fn on_joker_sold(&mut self, joker: &Jokers) {
        match self {
            Backend::Fast(b) => b.on_joker_sold(joker),
            Backend::Real(b) => b.on_joker_sold(joker),
        }
    }

    fn on_voucher_bought(&mut self, voucher: &Voucher) {
        match self {
            Backend::Fast(b) => b.on_voucher_bought(voucher),
            Backend::Real(b) => b.on_voucher_bought(voucher),
        }
    }

    fn roll_index(&mut self, len: usize) -> usize {
        match self {
            Backend::Fast(b) => b.roll_index(len),
            Backend::Real(b) => b.roll_index(len),
        }
    }

    fn roll_unit(&mut self) -> f64 {
        match self {
            Backend::Fast(b) => b.roll_unit(),
            Backend::Real(b) => b.roll_unit(),
        }
    }

    fn set_locked(&mut self, ids: Vec<String>) {
        match self {
            Backend::Fast(b) => b.set_locked(ids),
            Backend::Real(b) => b.set_locked(ids),
        }
    }

    fn set_gros_michel_extinct(&mut self, extinct: bool) {
        match self {
            Backend::Fast(b) => b.set_gros_michel_extinct(extinct),
            Backend::Real(b) => b.set_gros_michel_extinct(extinct),
        }
    }

    fn set_showman(&mut self, owned: bool) {
        match self {
            Backend::Fast(b) => b.set_showman(owned),
            Backend::Real(b) => b.set_showman(owned),
        }
    }

    fn gen_joker(&mut self, ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers {
        match self {
            Backend::Fast(b) => b.gen_joker(ante, prob_mult, exclude),
            Backend::Real(b) => b.gen_joker(ante, prob_mult, exclude),
        }
    }

    fn gen_joker_of_rarity(
        &mut self,
        ante: i32,
        prob_mult: u32,
        exclude: &[Jokers],
        rarity: Rarity,
    ) -> Jokers {
        match self {
            Backend::Fast(b) => b.gen_joker_of_rarity(ante, prob_mult, exclude, rarity),
            Backend::Real(b) => b.gen_joker_of_rarity(ante, prob_mult, exclude, rarity),
        }
    }

    fn pick_random_joker(&mut self, available: Vec<Jokers>) -> Jokers {
        match self {
            Backend::Fast(b) => b.pick_random_joker(available),
            Backend::Real(b) => b.pick_random_joker(available),
        }
    }

    fn clone_joker(&mut self, j: Jokers) -> Jokers {
        match self {
            Backend::Fast(b) => b.clone_joker(j),
            Backend::Real(b) => b.clone_joker(j),
        }
    }

    fn shuffle_deck(&mut self, deck: &mut Deck) {
        match self {
            Backend::Fast(b) => b.shuffle_deck(deck),
            Backend::Real(b) => b.shuffle_deck(deck),
        }
    }

    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        match self {
            Backend::Fast(b) => b.prob_roll(numerator, denominator),
            Backend::Real(b) => b.prob_roll(numerator, denominator),
        }
    }

    fn draw_ante_tags(&mut self) -> (Tag, Tag) {
        match self {
            Backend::Fast(b) => b.draw_ante_tags(),
            Backend::Real(b) => b.draw_ante_tags(),
        }
    }

    fn draw_boss(&mut self, ante: i32) -> BossBlind {
        match self {
            Backend::Fast(b) => b.draw_boss(ante),
            Backend::Real(b) => b.draw_boss(ante),
        }
    }

    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable {
        match self {
            Backend::Fast(b) => b.roll_random_tarot(exclude),
            Backend::Real(b) => b.roll_random_tarot(exclude),
        }
    }

    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable {
        match self {
            Backend::Fast(b) => b.roll_random_planet(planetarium, exclude),
            Backend::Real(b) => b.roll_random_planet(planetarium, exclude),
        }
    }

    fn roll_discard_selector(&mut self, j: &mut Jokers) {
        match self {
            Backend::Fast(b) => b.roll_discard_selector(j),
            Backend::Real(b) => b.roll_discard_selector(j),
        }
    }
    fn roll_random_edition(&mut self) -> Edition {
        match self {
            Backend::Fast(b) => b.roll_random_edition(),
            Backend::Real(b) => b.roll_random_edition(),
        }
    }

    fn roll_random_suit(&mut self) -> Suit {
        match self {
            Backend::Fast(b) => b.roll_random_suit(),
            Backend::Real(b) => b.roll_random_suit(),
        }
    }

    fn roll_random_value(&mut self) -> Value {
        match self {
            Backend::Fast(b) => b.roll_random_value(),
            Backend::Real(b) => b.roll_random_value(),
        }
    }

    fn pick_random_card(&mut self, available: Vec<Card>) -> Card {
        match self {
            Backend::Fast(b) => b.pick_random_card(available),
            Backend::Real(b) => b.pick_random_card(available),
        }
    }

    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card {
        match self {
            Backend::Fast(b) => b.gen_random_card(prob_mult, force_enhance, force_values),
            Backend::Real(b) => b.gen_random_card(prob_mult, force_enhance, force_values),
        }
    }
}
