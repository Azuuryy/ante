use super::*;

/// Telescope guarantees the most played hand's Planet card in every
/// Celestial Pack. Applied after generation so it works the same for both
/// RNG backends.
pub(crate) fn apply_telescope(pack: &mut Pack, ctx: &ShopContext) {
    if !ctx.vouchers.telescope() || pack.category != PackCategory::Celestial {
        return;
    }
    let Some(wanted) = Planets::iter().find(|p| p.hand_rank() == ctx.most_played) else {
        return;
    };
    if pack
        .contents
        .iter()
        .any(|c| matches!(c, PackContent::Planet(p) if *p == wanted))
    {
        return;
    }
    if let Some(first) = pack.contents.first_mut() {
        *first = PackContent::Planet(wanted);
    }
}

pub(crate) fn gen_edition(prob_mult: u32, rng: &mut impl Rng, allow_negative: bool) -> Edition {
    if allow_negative && rng.gen_ratio(3u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Negative;
    }
    if rng.gen_ratio(3u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Polychrome;
    }
    if rng.gen_ratio(14u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Holographic;
    }
    if rng.gen_ratio(20u32.saturating_mul(prob_mult).min(1000), 1000) {
        return Edition::Foil;
    }
    Edition::Base
}

/// A shop playing card (Magic Trick). Plain unless Illusion is also
/// redeemed, which is the only thing that lets it roll an enhancement,
/// edition, or seal.
pub(crate) fn gen_shop_playing_card(prob_mult: u32, illusion: bool, rng: &mut impl Rng) -> Card {
    if illusion {
        return gen_random_playing_card(prob_mult, rng, false, None);
    }
    let values: Vec<Value> = Value::iter().collect();
    let suits: Vec<Suit> = Suit::iter().collect();
    let v = values[rng.gen_range(0..values.len())];
    let s = suits[rng.gen_range(0..suits.len())];
    Card::new(v, s)
}

pub(crate) fn gen_random_playing_card(
    prob_mult: u32,
    rng: &mut impl Rng,
    force_enhance: bool,
    force_values: Option<&[Value]>,
) -> Card {
    let values: Vec<Value> = Value::iter()
        .filter(|v| force_values.is_none_or(|fv| fv.contains(v)))
        .collect();
    let suits: Vec<Suit> = Suit::iter().collect();
    let v = values[rng.gen_range(0..values.len())];
    let s = suits[rng.gen_range(0..suits.len())];
    let mut card = Card::new(v, s);

    if rng.gen_ratio(1, 5) || force_enhance {
        const ENHANCEMENTS: [Enhancement; 8] = [
            Enhancement::Bonus,
            Enhancement::Mult,
            Enhancement::Wild,
            Enhancement::Glass,
            Enhancement::Steel,
            Enhancement::Stone,
            Enhancement::Gold,
            Enhancement::Lucky,
        ];
        card.enhancement = Some(ENHANCEMENTS[rng.gen_range(0..ENHANCEMENTS.len())]);
    }

    card.edition = gen_edition(prob_mult, rng, false);

    if rng.gen_ratio(1, 10) {
        const SEALS: [Seal; 4] = [Seal::Gold, Seal::Red, Seal::Blue, Seal::Purple];
        card.seal = Some(SEALS[rng.gen_range(0..SEALS.len())]);
    }

    card
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct JokerGenerator {
    /// `G.GAME.pool_flags.gros_michel_extinct`: Cavendish enters the pool, Gros Michel
    /// leaves it.
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) gros_michel_extinct: bool,
    /// Joker ids not unlocked yet; Legendaries are never left out.
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) locked: Vec<String>,
}

impl JokerGenerator {
    pub(crate) fn new() -> Self {
        JokerGenerator {
            gros_michel_extinct: false,
            locked: Vec::new(),
        }
    }

    // Randomly generate rarity of new joker.
    // 70% chance Common, 25% chance Uncommon, 5% chance Rare.
    // Legendary can only appear from Soul Spectral Card.
    pub(super) fn gen_rarity(&self, rng: &mut impl Rng) -> Rarity {
        let choices = [Rarity::Common, Rarity::Uncommon, Rarity::Rare];
        let weights = [70u32, 25, 5];
        let dist = WeightedIndex::new(weights).unwrap();
        choices[dist.sample(rng)]
    }

    pub(crate) fn gen_joker(
        &self,
        prob_mult: u32,
        exclude: &[Jokers],
        rng: &mut impl Rng,
    ) -> Jokers {
        let rarity = self.gen_rarity(rng);
        self.gen_joker_of_rarity(prob_mult, exclude, rarity, rng)
    }

    pub(crate) fn gen_joker_of_rarity(
        &self,
        prob_mult: u32,
        exclude: &[Jokers],
        rarity: Rarity,
        rng: &mut impl Rng,
    ) -> Jokers {
        let extinct = self.gros_michel_extinct;
        let all: Vec<_> = jokers_by_rarity(rarity)
            .into_iter()
            .filter(|j| match j {
                Jokers::GrosMichel(_) => !extinct,
                Jokers::Cavendish(_) => extinct,
                _ => true,
            })
            .filter(|j| {
                j.rarity() == Rarity::Legendary || !self.locked.iter().any(|id| id == j.id())
            })
            .collect();
        let choices: Vec<_> = all
            .iter()
            .filter(|j| {
                !exclude
                    .iter()
                    .any(|e| std::mem::discriminant(e) == std::mem::discriminant(*j))
            })
            .cloned()
            .collect();
        let choices = if choices.is_empty() { all } else { choices };
        let i = rng.gen_range(0..choices.len());
        let mut joker = choices[i].clone();
        joker.set_edition(gen_edition(prob_mult, rng, true));
        joker.set_instance_id(crate::joker::mint_joker_id());
        crate::joker::roll_discard_selector(rng, &mut joker);
        joker
    }

    pub(crate) fn clone_joker(&self, j: Jokers, rng: &mut impl Rng) -> Jokers {
        let mut joker = j.to_owned();
        joker.set_instance_id(crate::joker::mint_joker_id());
        crate::joker::roll_discard_selector(rng, &mut joker);
        joker
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct ConsumableGenerator {}

impl ConsumableGenerator {
    pub(crate) fn new() -> Self {
        ConsumableGenerator {}
    }

    /// Generate a random planet, excluding secret planets unless their hand has been played,
    /// and excluding any already-picked planets.
    pub(crate) fn gen_planet(
        &self,
        planetarium: &Planetarium,
        exclude: &[Planets],
        rng: &mut impl Rng,
    ) -> Planets {
        let available: Vec<Planets> = Planets::iter()
            .filter(|p| {
                if exclude.contains(p) {
                    return false;
                }
                if p.is_secret() {
                    planetarium.level(p.hand_rank()).plays > 0
                } else {
                    true
                }
            })
            .collect();
        if available.is_empty() {
            return self.gen_planet(planetarium, &[], rng);
        }
        let i = rng.gen_range(0..available.len());
        available[i]
    }

    pub(crate) fn gen_planet_consumable(
        &self,
        planetarium: &Planetarium,
        exclude: &[Planets],
        rng: &mut impl Rng,
    ) -> Consumable {
        Consumable::Planet(self.gen_planet(planetarium, exclude, rng))
    }

    pub(super) fn gen_tarot(&self, exclude: &[Tarot], rng: &mut impl Rng) -> Tarot {
        let all: Vec<Tarot> = Tarot::iter().collect();
        let choices: Vec<_> = all.iter().filter(|t| !exclude.contains(t)).collect();
        let choices = if choices.is_empty() {
            all.iter().collect()
        } else {
            choices
        };
        let i = rng.gen_range(0..choices.len());
        *choices[i]
    }

    pub(crate) fn gen_tarot_consumable(&self, exclude: &[Tarot], rng: &mut impl Rng) -> Consumable {
        Consumable::Tarot(self.gen_tarot(exclude, rng))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub(crate) struct PackGenerator {
    /// Same flag as [`JokerGenerator`], for Buffoon packs.
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) gros_michel_extinct: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub(crate) locked: Vec<String>,
}

impl PackGenerator {
    // Weighted random pack selection per spec (real per-mille weights from
    // balatro-seed/src/pools.rs's PACKS table, scaled *100 so every category,
    // including Spectral's 0.07 Mega weight, is an exact integer):
    // Standard/Arcana/Celestial: Normal=4.0, Jumbo=2.0, Mega=0.5 -> 400,200,50
    // Buffoon: Normal=1.2, Jumbo=0.6, Mega=0.15 -> 120,60,15
    // Spectral: Normal=0.6, Jumbo=0.3, Mega=0.07 -> 60,30,7
    pub(crate) fn gen_pack(
        &self,
        planetarium: &Planetarium,
        prob_mult: u32,
        exclude: Option<(&PackCategory, &PackSize)>,
        held_jokers: &[Jokers],
        rng: &mut impl Rng,
    ) -> Pack {
        #[rustfmt::skip]
        let all_choices: &[(PackCategory, PackSize, u32)] = &[
            (PackCategory::Standard,  PackSize::Normal, 400),
            (PackCategory::Standard,  PackSize::Jumbo,  200),
            (PackCategory::Standard,  PackSize::Mega,    50),
            (PackCategory::Arcana,    PackSize::Normal, 400),
            (PackCategory::Arcana,    PackSize::Jumbo,  200),
            (PackCategory::Arcana,    PackSize::Mega,    50),
            (PackCategory::Celestial, PackSize::Normal, 400),
            (PackCategory::Celestial, PackSize::Jumbo,  200),
            (PackCategory::Celestial, PackSize::Mega,    50),
            (PackCategory::Buffoon,   PackSize::Normal, 120),
            (PackCategory::Buffoon,   PackSize::Jumbo,   60),
            (PackCategory::Buffoon,   PackSize::Mega,    15),
            (PackCategory::Spectral,  PackSize::Normal,  60),
            (PackCategory::Spectral,  PackSize::Jumbo,   30),
            (PackCategory::Spectral,  PackSize::Mega,     7),
        ];

        let choices: Vec<&(PackCategory, PackSize, u32)> = all_choices
            .iter()
            .filter(|(cat, sz, _)| exclude.is_none_or(|(ec, es)| cat != ec || sz != es))
            .collect();

        let weights: Vec<u32> = choices.iter().map(|(_, _, w)| *w).collect();
        let dist = WeightedIndex::new(&weights).unwrap();
        let idx = dist.sample(rng);
        let (category, size, _) = choices[idx];

        // Shared with `RealBackend` (`rng.rs`) so Fast/Real modes can never disagree
        // on pack content counts - Buffoon and Spectral share the smaller 2/4/4
        // shape, everything else is 3/5/5.
        let count = balatro_seed::pack_card_count(*category, *size) as usize;

        let contents = self.gen_contents(category, count, planetarium, prob_mult, held_jokers, rng);

        Pack {
            category: *category,
            size: *size,
            contents,
        }
    }

    pub(crate) fn gen_contents(
        &self,
        category: &PackCategory,
        count: usize,
        planetarium: &Planetarium,
        prob_mult: u32,
        held_jokers: &[Jokers],
        rng: &mut impl Rng,
    ) -> Vec<PackContent> {
        let joker_gen = JokerGenerator {
            gros_michel_extinct: self.gros_michel_extinct,
            locked: self.locked.clone(),
        };
        let consumable_gen = ConsumableGenerator {};

        match category {
            PackCategory::Arcana => (0..count)
                .map(|_| PackContent::Tarot(crate::tarot::random_tarot(rng)))
                .collect(),
            PackCategory::Celestial => {
                let mut exclude: Vec<Planets> = vec![];
                (0..count)
                    .map(|_| {
                        let planet = consumable_gen.gen_planet(planetarium, &exclude, rng);
                        exclude.push(planet);
                        PackContent::Planet(planet)
                    })
                    .collect()
            }
            PackCategory::Buffoon => {
                let mut seen: Vec<Jokers> = held_jokers.to_vec();
                (0..count)
                    .map(|_| {
                        let joker = joker_gen.gen_joker(prob_mult, &seen, rng);
                        seen.push(joker.clone());
                        PackContent::Joker(joker)
                    })
                    .collect()
            }
            PackCategory::Standard => (0..count)
                .map(|_| {
                    PackContent::PlayingCard(gen_random_playing_card(prob_mult, rng, false, None))
                })
                .collect(),
            PackCategory::Spectral => (0..count)
                .map(|_| PackContent::Spectral(crate::spectral::random_spectral(rng)))
                .collect(),
        }
    }
}
