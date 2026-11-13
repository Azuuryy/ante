use super::*;

impl Instance {
    /// `functions.hpp::nextPack`. The run's first pack (ante <= 2, only
    /// once) is always a Buffoon pack.
    pub fn next_pack(&mut self, ante: i32) -> (PackCategory, PackSize) {
        if ante <= 2 && !self.generated_first_pack && self.params.version > 10099 {
            self.generated_first_pack = true;
            return (PackCategory::Buffoon, PackSize::Normal);
        }
        let name = self.randweightedchoice(NodeId::ShopPack(ante), pools::PACKS);
        parse_pack_name(name)
    }

    /// Locks each drawn card so the pack can't repeat one, then unlocks
    /// them all once fully drawn — bypassed under `showman`.
    pub fn next_arcana_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item =
                if self.is_voucher_active("Omen Globe") && self.random(NodeId::OmenGlobe) > 0.8 {
                    self.next_spectral("ar2", ante, true)
                } else {
                    self.next_tarot("ar1", ante, true)
                };
            if !self.params.showman {
                self.lock(&item);
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(item);
        }
        pack
    }

    /// `functions.hpp::nextCelestialPack`.
    pub fn next_celestial_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = self.next_planet("pl1", ante, true);
            if !self.params.showman {
                self.lock(&item);
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(item);
        }
        pack
    }

    /// `functions.hpp::nextSpectralPack`.
    pub fn next_spectral_pack(&mut self, size: i32, ante: i32) -> Vec<Consumable> {
        let mut pack: Vec<Consumable> = Vec::new();
        for _ in 0..size {
            let item = self.next_spectral("spe", ante, true);
            if !self.params.showman {
                self.lock(&item);
            }
            pack.push(item);
        }
        for item in &pack {
            self.unlock(item);
        }
        pack
    }

    /// `functions.hpp::nextBuffoonPack`.
    pub fn next_buffoon_pack(&mut self, size: i32, ante: i32) -> Vec<Jokers> {
        let mut pack: Vec<Jokers> = Vec::new();
        for _ in 0..size {
            let joker = self.next_joker("buf", ante);
            if !self.params.showman {
                self.lock(&joker);
            }
            pack.push(joker);
        }
        for joker in &pack {
            self.unlock(joker);
        }
        pack
    }

    /// `functions.hpp::nextStandardCard`.
    pub fn next_standard_card(&mut self, ante: i32) -> Card {
        let enhancement = if self.random(NodeId::StdSet(ante)) <= 0.6 {
            None
        } else {
            Some(self.randchoice_typed(NodeId::EnhancedStandard(ante), &pools::ENHANCEMENTS_POOL))
        };

        let mut card: Card = self.randchoice_typed(NodeId::FrontStandard(ante), &pools::CARDS_POOL);
        card.enhancement = enhancement;

        let edition_poll = self.random(NodeId::StandardEdition(ante));
        card.edition = if edition_poll > 0.988 {
            Edition::Polychrome
        } else if edition_poll > 0.96 {
            Edition::Holographic
        } else if edition_poll > 0.92 {
            Edition::Foil
        } else {
            Edition::Base
        };

        card.seal = if self.random(NodeId::StdSeal(ante)) <= 0.8 {
            None
        } else {
            let seal_poll = self.random(NodeId::StdSealType(ante));
            Some(if seal_poll > 0.75 {
                Seal::Red
            } else if seal_poll > 0.5 {
                Seal::Blue
            } else if seal_poll > 0.25 {
                Seal::Gold
            } else {
                Seal::Purple
            })
        };

        card
    }

    /// Unlike every other pack-content method, never locks drawn cards —
    /// duplicates within one Standard Pack are expected.
    pub fn next_standard_pack(&mut self, size: i32, ante: i32) -> Vec<Card> {
        (0..size).map(|_| self.next_standard_card(ante)).collect()
    }

    /// `functions.hpp::nextShopItem` (via `getShopInstance` inlined). The
    /// `PlayingCard` branch is a placeholder — see module docs.
    pub fn next_shop_item(&mut self, ante: i32) -> ShopItem {
        let joker_rate = 20.0f64;
        let mut tarot_rate = 4.0;
        let mut planet_rate = 4.0;
        let mut playing_card_rate = 0.0;
        let mut spectral_rate = 0.0;
        if self.params.deck == "Ghost Deck" {
            spectral_rate = 2.0;
        }
        if self.is_voucher_active("Tarot Tycoon") {
            tarot_rate = 32.0;
        } else if self.is_voucher_active("Tarot Merchant") {
            tarot_rate = 9.6;
        }
        if self.is_voucher_active("Planet Tycoon") {
            planet_rate = 32.0;
        } else if self.is_voucher_active("Planet Merchant") {
            planet_rate = 9.6;
        }
        if self.is_voucher_active("Magic Trick") {
            playing_card_rate = 4.0;
        }
        let total = joker_rate + tarot_rate + planet_rate + playing_card_rate + spectral_rate;

        let mut poll = self.random(NodeId::Cdt(ante)) * total;

        if poll < joker_rate {
            return ShopItem::Joker(self.next_joker("sho", ante));
        }
        poll -= joker_rate;
        if poll < tarot_rate {
            return ShopItem::Consumable(self.next_tarot("sho", ante, false));
        }
        poll -= tarot_rate;
        if poll < planet_rate {
            return ShopItem::Consumable(self.next_planet("sho", ante, false));
        }
        poll -= planet_rate;
        if poll < playing_card_rate {
            return ShopItem::PlayingCard;
        }
        ShopItem::Consumable(self.next_spectral("sho", ante, false))
    }
}
