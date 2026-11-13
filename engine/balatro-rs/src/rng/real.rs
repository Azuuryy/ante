use super::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct RealBackend {
    pub(super) instance: Instance,
    // `Instance` only replays real Balatro's actual seed algorithm, need a
    // separate dedicated fast rng for some other things atm — also used
    // for shop playing cards (Magic Trick): `balatro-seed`'s
    // `ShopItem::PlayingCard` is a marker that carries no card data, and
    // drawing one out of the `Instance` would advance nodes the real game
    // doesn't, desyncing every later draw.
    pub(super) extra_rng: ChaCha8Rng,
    // Need a fast backend for gen that real doesn't yet impl
    pub(super) fast: FastBackend,
}

impl RealBackend {
    pub(crate) fn new(seed: &str, fast: FastBackend) -> Self {
        RealBackend {
            instance: Instance::new(seed),
            extra_rng: ChaCha8Rng::seed_from_u64(crate::seed_from_str(seed).wrapping_add(1)),
            fast,
        }
    }

    pub(super) fn gen_pack_contents(
        &mut self,
        ante: i32,
        category: PackCategory,
        count: i32,
    ) -> Vec<PackContent> {
        match category {
            PackCategory::Arcana => self
                .instance
                .next_arcana_pack(count, ante)
                .into_iter()
                .map(consumable_to_pack_content)
                .collect(),
            PackCategory::Celestial => self
                .instance
                .next_celestial_pack(count, ante)
                .into_iter()
                .map(consumable_to_pack_content)
                .collect(),
            PackCategory::Spectral => self
                .instance
                .next_spectral_pack(count, ante)
                .into_iter()
                .map(consumable_to_pack_content)
                .collect(),
            PackCategory::Buffoon => self
                .instance
                .next_buffoon_pack(count, ante)
                .into_iter()
                .map(|j| seed_joker_with_id(j, &mut self.extra_rng))
                .map(PackContent::Joker)
                .collect(),
            PackCategory::Standard => self
                .instance
                .next_standard_pack(count, ante)
                .into_iter()
                .map(|c| PackContent::PlayingCard(seed_card_to_core_card(c)))
                .collect(),
        }
    }
}

/// `balatro_types::Card` has no `id`; `core::Card` adds one, so only
/// `value`/`suit` go through the id-assigning constructor.
pub(super) fn seed_card_to_core_card(c: balatro_types::Card) -> Card {
    let mut card = Card::new(c.value, c.suit);
    card.edition = c.edition;
    card.enhancement = c.enhancement;
    card.seal = c.seal;
    card
}

/// Joker generation needs some engine only init at create time
pub(super) fn seed_joker_with_id(mut j: Jokers, rng: &mut impl Rng) -> Jokers {
    j.set_instance_id(crate::joker::mint_joker_id());
    crate::joker::roll_discard_selector(rng, &mut j);
    j
}

/// `Consumable` may carry Soul/Black Hole (a Spectral) even from a
/// nominally Tarot/Planet draw.
pub(super) fn consumable_to_pack_content(c: Consumable) -> PackContent {
    match c {
        Consumable::Tarot(t) => PackContent::Tarot(t),
        Consumable::Planet(p) => PackContent::Planet(p),
        Consumable::Spectral(s) => PackContent::Spectral(s),
    }
}

impl RngBackend for RealBackend {
    // `ctx.planetarium` is unused: `Fast` mode uses it to gate secret
    // planets behind discovery state, which isn't wired into `Real` mode
    // TODO. Voucher-driven rates come from the `Instance`'s own active
    // list (kept current by `on_voucher_bought`), not `ctx.vouchers`.
    fn gen_shop_item(
        &mut self,
        ctx: &ShopContext,
        _exclude_jokers: &[Jokers],
        _exclude_tarots: &[Tarot],
        _exclude_planets: &[Planets],
    ) -> GeneratedItem {
        match self.instance.next_shop_item(ctx.ante) {
            balatro_seed::ShopItem::Joker(j) => {
                GeneratedItem::Joker(seed_joker_with_id(j, &mut self.extra_rng))
            }
            balatro_seed::ShopItem::Consumable(c) => GeneratedItem::Consumable(c),
            balatro_seed::ShopItem::PlayingCard => {
                GeneratedItem::PlayingCard(gen_shop_playing_card(
                    ctx.edition_prob_mult(),
                    ctx.vouchers.shop_cards_are_modified(),
                    &mut self.extra_rng,
                ))
            }
        }
    }

    fn gen_pack(
        &mut self,
        ctx: &ShopContext,
        _exclude: Option<(&PackCategory, &PackSize)>,
    ) -> Pack {
        let (category, size) = self.instance.next_pack(ctx.ante);
        let count = balatro_seed::pack_card_count(category, size);
        let contents = self.gen_pack_contents(ctx.ante, category, count);
        Pack {
            category,
            size,
            contents,
        }
    }

    // Unlike `Fast` mode this draws from the full pool and then filters:
    // the draw has to consume the same node the real game would, whether
    // or not the result is still offerable. A base voucher already owned
    // is upgraded in place (`voucher_upgrade`), matching the real game.
    fn gen_voucher(&mut self, ante: i32, vouchers: &Vouchers) -> Option<Voucher> {
        let drawn = self.instance.next_voucher(ante);
        let offer = if vouchers.has(drawn) {
            balatro_seed::voucher_upgrade(drawn).filter(|u| vouchers.is_offerable(*u))
        } else {
            Some(drawn).filter(|v| vouchers.is_offerable(*v))
        };
        if let Some(v) = offer {
            self.instance.lock(&v);
        }
        offer
    }

    fn on_joker_bought(&mut self, joker: &Jokers) {
        self.instance.lock(joker);
    }

    fn on_joker_sold(&mut self, joker: &Jokers) {
        self.instance.unlock(joker);
    }

    fn on_voucher_bought(&mut self, voucher: &Voucher) {
        self.instance.activate_voucher(voucher);
    }

    fn set_showman(&mut self, owned: bool) {
        self.instance.params.showman = owned;
    }

    // `Instance::next_joker(source, ante)` exists and isn't wired up yet,
    // real accuracy here is a follow-up. Reuses the same Fast-style
    // generator `FastBackend` uses, seeded off `extra_rng` instead.
    fn gen_joker(&mut self, ante: i32, prob_mult: u32, exclude: &[Jokers]) -> Jokers {
        self.fast.gen_joker(ante, prob_mult, exclude)
    }

    fn gen_joker_of_rarity(
        &mut self,
        ante: i32,
        prob_mult: u32,
        exclude: &[Jokers],
        rarity: Rarity,
    ) -> Jokers {
        self.fast
            .gen_joker_of_rarity(ante, prob_mult, exclude, rarity)
    }

    fn pick_random_joker(&mut self, available: Vec<Jokers>) -> Jokers {
        self.fast.pick_random_joker(available)
    }

    fn clone_joker(&mut self, j: Jokers) -> Jokers {
        self.fast.clone_joker(j)
    }

    // No real Instance equivalent for deck draw order, yet
    fn shuffle_deck(&mut self, deck: &mut Deck) {
        self.fast.shuffle_deck(deck);
    }

    // No real Instance equivalent for scoring-time probability procs
    // (Lucky/Misprint/SpaceJoker/OopsAllSixes-class effects).
    fn prob_roll(&mut self, numerator: u32, denominator: u32) -> bool {
        self.fast.prob_roll(numerator, denominator)
    }

    // `Instance::next_tag(ante)` exists and isn't wired up yet - real
    // accuracy here is a follow-up.
    fn draw_ante_tags(&mut self) -> (Tag, Tag) {
        self.fast.draw_ante_tags()
    }

    // Correct category (regular vs. finisher) and correct pool-cycling, both
    // handled internally by `next_boss`. Minimum-ante is NOT: real Balatro
    // enforces that by pre-locking each not-yet-eligible boss name via
    // `Instance::init_locks`/`init_unlocks`, which nothing in `core` calls.
    // TODO: Fix min ante for real rng boss gen
    fn draw_boss(&mut self, ante: i32) -> BossBlind {
        self.instance.next_boss(ante)
    }

    // `Instance::next_tarot(source, ante, soulable)` exists and isn't
    // wired up yet - real accuracy here is a follow-up.
    fn roll_random_tarot(&mut self, exclude: &[Tarot]) -> Consumable {
        self.fast.roll_random_tarot(exclude)
    }

    // `Instance::next_planet(source, ante, soulable)` exists and isn't
    // wired up yet - real accuracy here is a follow-up.
    fn roll_random_planet(&mut self, planetarium: &Planetarium, exclude: &[Planets]) -> Consumable {
        self.fast.roll_random_planet(planetarium, exclude)
    }

    fn roll_discard_selector(&mut self, j: &mut Jokers) {
        self.fast.roll_discard_selector(j);
    }

    fn roll_random_edition(&mut self) -> Edition {
        self.fast.roll_random_edition()
    }

    fn roll_random_suit(&mut self) -> Suit {
        self.fast.roll_random_suit()
    }

    fn roll_random_value(&mut self) -> Value {
        self.fast.roll_random_value()
    }

    fn pick_random_card(&mut self, available: Vec<Card>) -> Card {
        self.fast.pick_random_card(available)
    }

    fn gen_random_card(
        &mut self,
        prob_mult: u32,
        force_enhance: bool,
        force_values: Option<&[Value]>,
    ) -> Card {
        self.fast
            .gen_random_card(prob_mult, force_enhance, force_values)
    }
}
