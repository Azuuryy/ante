use super::*;

impl Instance {
    /// `functions.hpp::nextTarot`. Can return `The Soul` (a Spectral card)
    /// instead of a real tarot — a real-game quirk, not a bug.
    pub fn next_tarot(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        if soulable
            && (self.params.showman || !self.is_locked(&Spectral::Soul))
            && self.random(NodeId::SoulTarot(ante)) > 0.997
        {
            return Consumable::Spectral(Spectral::Soul);
        }
        let tarot = self.randchoice_typed(NodeId::Tarot { source, ante }, &pools::TAROTS_POOL);
        Consumable::Tarot(tarot)
    }

    /// `functions.hpp::nextPlanet`. Can return `Black Hole` (a Spectral
    /// card) instead of a real planet.
    pub fn next_planet(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        if soulable
            && (self.params.showman || !self.is_locked(&Spectral::BlackHole))
            && self.random(NodeId::SoulPlanet(ante)) > 0.997
        {
            return Consumable::Spectral(Spectral::BlackHole);
        }
        let planet = self.randchoice_typed(NodeId::Planet { source, ante }, &pools::PLANETS_POOL);
        Consumable::Planet(planet)
    }

    /// `functions.hpp::nextSpectral`. Soul and Black Hole both draw from the
    /// same node ID; if both succeed, Black Hole wins (checked second).
    pub fn next_spectral(&mut self, source: &str, ante: i32, soulable: bool) -> Consumable {
        if soulable {
            let mut forced: Option<Spectral> = None;
            if (self.params.showman || !self.is_locked(&Spectral::Soul))
                && self.random(NodeId::SoulSpectral(ante)) > 0.997
            {
                forced = Some(Spectral::Soul);
            }
            if (self.params.showman || !self.is_locked(&Spectral::BlackHole))
                && self.random(NodeId::SoulSpectral(ante)) > 0.997
            {
                forced = Some(Spectral::BlackHole);
            }
            if let Some(s) = forced {
                return Consumable::Spectral(s);
            }
        }
        let spectral =
            self.randchoice_typed(NodeId::Spectral { source, ante }, &pools::SPECTRALS_POOL);
        Consumable::Spectral(spectral)
    }

    /// `source` selects a forced rarity for a few callers ("sou" =
    /// legendary, "wra"/"rta" = rare, "uta" = uncommon), else rarity is rolled.
    pub fn next_joker(&mut self, source: &str, ante: i32) -> Jokers {
        let rarity: &str = match source {
            "sou" => "4",
            "wra" | "rta" => "3",
            "uta" => "2",
            _ => {
                let poll = self.random(NodeId::Rarity { source, ante });
                if poll > 0.95 {
                    "3"
                } else if poll > 0.7 {
                    "2"
                } else {
                    "1"
                }
            }
        };

        let edition_rate: f64 = if self.is_voucher_active("Glow Up") {
            4.0
        } else if self.is_voucher_active("Hone") {
            2.0
        } else {
            1.0
        };
        let edition_poll = self.random(NodeId::Edition { source, ante });
        let edition = if edition_poll > 0.997 {
            Edition::Negative
        } else if edition_poll > 1.0 - 0.006 * edition_rate {
            Edition::Polychrome
        } else if edition_poll > 1.0 - 0.02 * edition_rate {
            Edition::Holographic
        } else if edition_poll > 1.0 - 0.04 * edition_rate {
            Edition::Foil
        } else {
            Edition::Base
        };

        let mut joker: Jokers = match rarity {
            "4" => self.randchoice_typed(NodeId::Joker4, &pools::LEGENDARY_JOKERS_POOL),
            "3" => self.randchoice_typed(NodeId::Joker3 { source, ante }, &pools::RARE_JOKERS_POOL),
            "2" => self.randchoice_typed(
                NodeId::Joker2 { source, ante },
                &pools::UNCOMMON_JOKERS_POOL,
            ),
            _ => self.randchoice_typed(NodeId::Joker1 { source, ante }, &pools::COMMON_JOKERS_POOL),
        };
        joker.set_edition(edition);
        joker
    }

    /// Does not lock the result — locking on purchase is the caller's job.
    pub fn next_voucher(&mut self, ante: i32) -> Voucher {
        self.randchoice_typed(NodeId::Voucher(ante), &pools::VOUCHERS_POOL)
    }

    /// `functions.hpp::nextTag`.
    pub fn next_tag(&mut self, ante: i32) -> Tag {
        self.randchoice_typed(NodeId::Tag(ante), &pools::TAGS_POOL)
    }

    /// Filters to the current ante's category (finisher on `ante % 8 == 0`),
    /// retrying with a full unlock if that category's pool is exhausted.
    pub fn next_boss(&mut self, ante: i32) -> BossBlind {
        let is_finisher_ante = ante % 8 == 0;
        let matches_category = |name: &str| {
            let is_t = name.starts_with('T');
            (is_finisher_ante && !is_t) || (!is_finisher_ante && is_t)
        };

        let pool: Vec<&str> = pools::BOSSES
            .iter()
            .copied()
            .filter(|&name| !self.is_locked(name) && matches_category(name))
            .collect();

        if pool.is_empty() {
            for &name in pools::BOSSES {
                if matches_category(name) {
                    self.unlock(name);
                }
            }
            return self.next_boss(ante);
        }

        let chosen = self.randchoice(NodeId::Boss, &pool);
        self.lock(chosen);
        pools::BOSSES_POOL.resolve(chosen)
    }
}
