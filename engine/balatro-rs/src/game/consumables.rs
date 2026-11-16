use super::*;
use crate::spectral::Spectral;

impl Game {
    /// Règles d'utilisation d'un consommable, paraphrasées de Card:can_use_consumeable.
    /// Source unique : appelée par le générateur et par use_consumable, avant tout retrait.
    pub(crate) fn consumable_usable(&self, c: &Consumable) -> bool {
        use crate::spectral::Spectral;
        use crate::tarot::Tarot;
        if matches!(self.stage, Stage::End(_)) {
            return false;
        }
        let selected = self.available.selected();
        let n = selected.len();
        let hand = self.available.cards().len();
        // SELECTING_HAND, TAROT_PACK and SPECTRAL_PACK of the original: the hand is there.
        let blind = self.hand_selectable();
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

    /// Whether hand cards can be selected: during a Blind, and while an Arcana or Spectral
    /// pack has drawn the hand to pick targets in.
    pub(crate) fn hand_selectable(&self) -> bool {
        self.stage.is_blind()
            || (self.stage == Stage::PackOpen() && !self.available.cards().is_empty())
    }

    /// Whether a pack card can be taken now: room for what it creates, and for a Tarot or
    /// Spectral the same `can_use_consumeable` rule as from a slot, with the current
    /// hand selection as targets.
    pub fn pack_card_usable(&self, content: &PackContent) -> bool {
        self.pack_card_pickable(content)
            && match content {
                PackContent::Tarot(t) => self.consumable_usable(&Consumable::Tarot(*t)),
                PackContent::Spectral(s) => self.consumable_usable(&Consumable::Spectral(*s)),
                _ => true,
            }
    }

    pub(crate) fn use_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
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
    pub(super) fn apply_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        match consumable {
            Consumable::Planet(planet) => {
                self.planetarium.level_up(planet.hand_rank());
                self.last_consumable_used = Some(Consumable::Planet(planet));
                self.planets_used.insert(planet);
                // Constellation: X0.1 more per Planet used.
                self.grow_jokers(
                    |j| matches!(j, Jokers::Constellation(_)),
                    0.1,
                    |value| JokerNote::XMult(100 + (value * 100.0).round() as usize),
                );
            }
            Consumable::Tarot(t) => {
                t.apply(self)?;
                // Fortune Teller: +1 Mult per Tarot used this run.
                self.tarots_used += 1;
                let used = self.tarots_used;
                self.grow_jokers(
                    |j| matches!(j, Jokers::FortuneTeller(_)),
                    0.0,
                    move |_| JokerNote::MultPlus(used),
                );
                if t != Tarot::Fool {
                    self.last_consumable_used = Some(Consumable::Tarot(t));
                }
            }
            Consumable::Spectral(s) => {
                // Fool only ever copies the last Tarot or Planet used, never a Spectral.
                s.apply(self)?;
            }
        }
        self.unhighlight_after(consumable);
        Ok(())
    }

    /// Clears the hand selection after the consumables whose original
    /// `Card:use_consumeable` ends with `G.hand:unhighlight_all()`: the enhancement and
    /// suit Tarots, Strength and Death, and the seal Spectrals.
    pub(super) fn unhighlight_after(&mut self, consumable: Consumable) {
        let clears = match consumable {
            Consumable::Tarot(t) => matches!(
                t,
                Tarot::Magician
                    | Tarot::Empress
                    | Tarot::Hierophant
                    | Tarot::Lovers
                    | Tarot::Chariot
                    | Tarot::Justice
                    | Tarot::Strength
                    | Tarot::Death
                    | Tarot::Devil
                    | Tarot::Tower
                    | Tarot::Star
                    | Tarot::Moon
                    | Tarot::Sun
                    | Tarot::World
            ),
            Consumable::Spectral(s) => matches!(
                s,
                Spectral::Talisman | Spectral::DejaVu | Spectral::Trance | Spectral::Medium
            ),
            Consumable::Planet(_) => false,
        };
        if clears {
            self.available.deselect_all();
        }
    }

    // Room for cards a consumable creates when it is not already in a consumable slot,
    // like `can_use_consumeable` for a card in a pack or the shop.
    pub(super) fn creation_room(&self, c: &Consumable) -> bool {
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

    pub(super) fn open_booster(&mut self, mut pack: Pack) {
        self.card_makers_open_booster();
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
        if !self.pack_card_usable(&content) {
            return Err(GameError::InvalidAction);
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

        // A consumable taken from a pack is used at once, with the hand's selection as
        // targets, and counts as the last one used, like `use_card` on a pack card.
        match content {
            PackContent::Tarot(t) => self.apply_consumable(Consumable::Tarot(t))?,
            PackContent::Planet(p) => self.apply_consumable(Consumable::Planet(p))?,
            PackContent::Joker(j) => self.add_acquired_joker(j),
            PackContent::PlayingCard(c) => {
                self.deck.push(c);
                self.playing_cards_added(1);
            }
            PackContent::Spectral(s) => self.apply_consumable(Consumable::Spectral(s))?,
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

    pub(crate) fn skip_pack(&mut self) -> Result<(), GameError> {
        if self.stage != Stage::PackOpen() {
            return Err(GameError::InvalidStage);
        }
        // Red Card: +3 Mult per Booster Pack skipped.
        self.grow_jokers(
            |j| matches!(j, Jokers::RedCard(_)),
            3.0,
            |_| JokerNote::MultPlus(3),
        );
        self.finish_pack();
        Ok(())
    }

    pub(super) fn finish_pack(&mut self) {
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
}
