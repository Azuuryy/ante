use super::*;

impl Game {
    pub(super) fn cashout(&mut self) -> Result<(), GameError> {
        self.money += self.reward;
        self.reward = 0;
        self.round_reward = Default::default();
        self.reroll_cost = self.base_reroll_cost();
        self.stage = Stage::Shop();
        self.refresh_voucher_offer();
        let planetarium = self.planetarium.clone();
        let held_consumables = self.consumables.clone();
        let held_jokers = self.jokers.clone();
        let vouchers = self.vouchers.clone();
        let ctx = self.shop_context(&planetarium, &held_consumables, &held_jokers, &vouchers);
        self.shop.refresh(&ctx, &mut self.backend);
        self.shop.extra_vouchers.clear();
        self.resolve_shop_tags(true);
        Ok(())
    }

    /// Draws this ante's voucher the first time its shop opens. Within an
    /// ante the offer is left alone: it survives rerolls, and once bought
    /// the slot stays empty until the next ante.
    pub(super) fn refresh_voucher_offer(&mut self) {
        // Strictly forward: Hieroglyph/Petroglyph walk the ante backwards,
        // and an ante already drawn for must not offer a second voucher.
        if self.voucher_ante.is_some_and(|a| self.ante_current <= a) {
            return;
        }
        self.voucher_ante = Some(self.ante_current);
        let vouchers = self.vouchers.clone();
        self.shop.voucher = self
            .backend
            .gen_voucher(self.ante_current.number() as i32, &vouchers);
    }

    pub(crate) fn reroll(&mut self) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.money < self.reroll_cost {
            return Err(GameError::InvalidBalance);
        }
        self.money -= self.reroll_cost;
        self.reroll_cost += 1;
        let planetarium = self.planetarium.clone();
        let mut held = self.consumables.clone();
        held.extend(self.shop.consumables.clone());
        let mut held_jokers = self.jokers.clone();
        held_jokers.extend(self.shop.jokers.clone());
        let vouchers = self.vouchers.clone();
        // Note: `refresh_cards`, not `refresh` — rerolls leave the packs
        // and the voucher offer untouched.
        let ctx = self.shop_context(&planetarium, &held, &held_jokers, &vouchers);
        self.shop.refresh_cards(&ctx, &mut self.backend);
        self.resolve_shop_tags(false);
        Ok(())
    }

    pub(crate) fn sell_joker(&mut self, idx: usize) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        if idx >= self.jokers.len() {
            return Err(GameError::InvalidAction);
        }
        self.money += self.jokers[idx].sell_value();
        let sold = self.jokers.remove(idx);
        for e in sold.effects(self) {
            if let Effects::OnSell(f) = e {
                f.lock().unwrap()(self);
            }
        }
        if sold.edition() == Edition::Negative {
            self.config.joker_slots = self.config.joker_slots.saturating_sub(1);
        }
        // Only unlock if this was the last owned copy (Showman is the only
        // way to own more than one) — a discriminant compare, matching
        // shop.rs's own exclude-list comparisons, since Jokers' derived
        // Eq also compares edition/stickers, not just "which joker".
        let still_owned = self
            .jokers
            .iter()
            .any(|j| std::mem::discriminant(j) == std::mem::discriminant(&sold));
        if !still_owned {
            self.backend.on_joker_sold(&sold);
        }
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
        Ok(())
    }

    pub(crate) fn sell_consumable(&mut self, idx: usize) -> Result<(), GameError> {
        if matches!(self.stage, Stage::End(_)) {
            return Err(GameError::InvalidStage);
        }
        if idx >= self.consumables.len() {
            return Err(GameError::InvalidAction);
        }
        self.money += self.consumables[idx].sell_value();
        self.consumables.remove(idx);
        Ok(())
    }

    pub(crate) fn can_add_joker(&self, joker: &Jokers) -> bool {
        let slots = self.joker_slots() + usize::from(joker.edition() == Edition::Negative);
        self.jokers.len() < slots.min(self.config.joker_slots_max)
    }

    pub(crate) fn buy_joker(&mut self, joker: Jokers) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if !self.can_add_joker(&joker) {
            return Err(GameError::NoAvailableSlot);
        }
        let price = self.shop.joker_price(&joker, &self.vouchers);
        if price > self.money {
            return Err(GameError::InvalidBalance);
        }
        self.shop.buy_joker(&joker)?;
        self.money -= price;
        self.add_acquired_joker(joker);
        Ok(())
    }

    pub(super) fn add_acquired_joker(&mut self, joker: Jokers) {
        if joker.edition() == Edition::Negative {
            self.config.joker_slots =
                (self.config.joker_slots + 1).min(self.config.joker_slots_max);
        }
        self.backend.on_joker_bought(&joker);
        self.jokers.push(joker);
        self.effect_registry
            .register_jokers(self.jokers.clone(), &self.clone());
    }

    pub(crate) fn buy_consumable(&mut self, consumable: Consumable) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.consumables.len() >= self.consumable_slots() {
            return Err(GameError::NoAvailableSlot);
        }
        if self.shop.consumable_price(&consumable, &self.vouchers) > self.money {
            return Err(GameError::InvalidBalance);
        }
        let price = self.shop.consumable_price(&consumable, &self.vouchers);
        self.shop.buy_consumable(&consumable)?;
        self.shop
            .free_consumables
            .retain(|offer| *offer != consumable);
        self.money -= price;
        self.consumables.push(consumable);
        Ok(())
    }

    /// Redeems the shop's voucher offer. Permanent for the run — there is
    /// no sell action, and the slot stays empty until the next ante.
    pub(crate) fn buy_voucher(&mut self, voucher: Voucher) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.vouchers.has(voucher) {
            return Err(GameError::VoucherAlreadyRedeemed);
        }
        // Vouchers are exempt from Clearance Sale / Liquidation.
        if voucher.cost() > self.money {
            return Err(GameError::InvalidBalance);
        }
        if !self.vouchers.is_offerable(voucher) {
            return Err(GameError::InvalidAction);
        }
        self.shop.buy_voucher(&voucher)?;
        self.money -= voucher.cost();
        self.vouchers.redeem(voucher);
        self.backend.on_voucher_bought(&voucher);
        self.apply_voucher(voucher);
        Ok(())
    }

    /// The parts of a voucher that aren't derived from the redeemed set on
    /// demand (see `voucher.rs`) — one-shot changes to live game state.
    pub(super) fn apply_voucher(&mut self, voucher: Voucher) {
        // "-1 Ante" (Hieroglyph/Petroglyph): the run steps back an ante,
        // making every remaining blind easier but adding a round.
        if Vouchers::lowers_ante(voucher) {
            if let Some(prev) = self.ante_current.prev() {
                self.ante_current = prev;
            }
        }
        // Reroll discounts apply to the shop the player is standing in,
        // not just the next one.
        if matches!(voucher, Voucher::RerollSurplus | Voucher::RerollGlut) {
            self.reroll_cost = self.reroll_cost.saturating_sub(2);
        }
        // Overstock widens the current shop immediately: top up the new
        // slots rather than making the player wait for the next round.
        if matches!(voucher, Voucher::Overstock | Voucher::OverstockPlus) {
            let planetarium = self.planetarium.clone();
            let mut held = self.consumables.clone();
            held.extend(self.shop.consumables.clone());
            let mut held_jokers = self.jokers.clone();
            held_jokers.extend(self.shop.jokers.clone());
            let vouchers = self.vouchers.clone();
            let ctx = self.shop_context(&planetarium, &held, &held_jokers, &vouchers);
            self.shop.top_up_cards(&ctx, &mut self.backend);
        }
    }

    pub(crate) fn buy_playing_card(&mut self, card: Card) -> Result<(), GameError> {
        if self.stage != Stage::Shop() {
            return Err(GameError::InvalidStage);
        }
        if self.deck.len() + self.available.cards().len() + self.discarded.len()
            >= self.config.deck_max
        {
            return Err(GameError::NoAvailableSlot);
        }
        let cost = self.shop.card_price(&card, &self.vouchers);
        if cost > self.money {
            return Err(GameError::InvalidBalance);
        }
        let bought = self.shop.buy_card(&card)?;
        self.money -= cost;
        self.deck.push(bought);
        Ok(())
    }
}
