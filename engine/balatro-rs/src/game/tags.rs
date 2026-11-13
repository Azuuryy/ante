use super::*;

impl Game {
    pub(super) fn take_tag(&mut self, tag: Tag) -> bool {
        if let Some(index) = self.tags.iter().position(|candidate| *candidate == tag) {
            self.tags.remove(index);
            true
        } else {
            false
        }
    }

    pub(super) fn acquire_tag(&mut self, tag: Tag, orbital: Option<HandRank>) {
        let copies = if tag == Tag::Double {
            1
        } else {
            let before = self.tags.len();
            self.tags.retain(|tag| *tag != Tag::Double);
            1 + before - self.tags.len()
        };
        for _ in 0..copies {
            match tag {
                Tag::Speed => self.money += self.skipped_count * 5,
                Tag::Handy => {
                    self.money += HandRank::iter()
                        .filter(|rank| *rank != HandRank::RoyalFlush)
                        .map(|rank| self.planetarium.level(rank).plays)
                        .sum::<usize>()
                }
                Tag::Garbage => self.money += self.unused_discards,
                Tag::Economy => self.money += self.money.min(40),
                Tag::Orbital => {
                    let rank = orbital.unwrap_or(HandRank::HighCard);
                    for _ in 0..3 {
                        self.planetarium.level_up(rank);
                    }
                }
                Tag::TopUp => {
                    for _ in 0..2 {
                        if self.jokers.len() >= self.joker_slots() {
                            break;
                        }
                        let mut joker = self.backend.gen_joker_of_rarity(
                            self.ante_current.number() as i32,
                            self.prob_mult,
                            &self.jokers,
                            Rarity::Common,
                        );
                        joker.set_edition(Edition::Base);
                        self.backend.on_joker_bought(&joker);
                        self.jokers.push(joker);
                    }
                    self.effect_registry
                        .register_jokers(self.jokers.clone(), &self.clone());
                }
                _ => self.tags.push(tag),
            }
        }
    }

    pub(super) fn resolve_blind_choice_tags(&mut self) {
        while let Some(index) = self.tags.iter().position(|tag| {
            matches!(
                tag,
                Tag::Boss | Tag::Standard | Tag::Charm | Tag::Meteor | Tag::Ethereal | Tag::Buffoon
            )
        }) {
            let (category, size) = match self.tags.remove(index) {
                Tag::Boss => {
                    self.replace_boss();
                    continue;
                }
                Tag::Standard => (PackCategory::Standard, PackSize::Mega),
                Tag::Charm => (PackCategory::Arcana, PackSize::Mega),
                Tag::Meteor => (PackCategory::Celestial, PackSize::Mega),
                Tag::Ethereal => (PackCategory::Spectral, PackSize::Normal),
                Tag::Buffoon => (PackCategory::Buffoon, PackSize::Mega),
                _ => unreachable!(),
            };
            let planetarium = self.planetarium.clone();
            let consumables = self.consumables.clone();
            let jokers = self.jokers.clone();
            let vouchers = self.vouchers.clone();
            let ctx = self.shop_context(&planetarium, &consumables, &jokers, &vouchers);
            let pack = self.backend.tag_pack(category, size, &ctx);
            self.pack_returns_to_blinds = true;
            self.open_booster(pack);
            break;
        }
    }

    pub(super) fn resolve_shop_tags(&mut self, opening: bool) {
        let mut tags: Vec<_> = std::mem::take(&mut self.tags)
            .into_iter()
            .enumerate()
            .collect();
        tags.sort_by_key(|(_, tag)| !matches!(tag, Tag::Uncommon | Tag::Rare));
        let mut pending = Vec::new();
        let mut edited = HashSet::new();
        let mut coupon = false;
        for (index, tag) in tags {
            match tag {
                Tag::Uncommon | Tag::Rare => {
                    if self.shop.jokers.len() == edited.len()
                        && self.shop.consumables.is_empty()
                        && self.shop.cards.is_empty()
                    {
                        pending.push((index, tag));
                        continue;
                    }
                    let rarity = if tag == Tag::Rare {
                        Rarity::Rare
                    } else {
                        Rarity::Uncommon
                    };
                    let mut excluded = self.jokers.clone();
                    excluded.extend(self.shop.jokers.iter().cloned());
                    let mut joker = self.backend.gen_joker_of_rarity(
                        self.ante_current.number() as i32,
                        self.prob_mult,
                        &excluded,
                        rarity,
                    );
                    joker.set_edition(Edition::Base);
                    if let Some(index) = self
                        .shop
                        .jokers
                        .iter()
                        .rposition(|offer| !edited.contains(&offer.instance_id()))
                    {
                        self.shop.jokers.remove(index);
                    } else if self.shop.consumables.pop().is_none() {
                        self.shop.cards.pop();
                    }
                    self.shop.free_jokers.push(joker.instance_id());
                    edited.insert(joker.instance_id());
                    self.shop.jokers.push(joker);
                }
                Tag::Foil | Tag::Holographic | Tag::Polychrome | Tag::Negative => {
                    let edition = match tag {
                        Tag::Foil => Edition::Foil,
                        Tag::Holographic => Edition::Holographic,
                        Tag::Polychrome => Edition::Polychrome,
                        _ => Edition::Negative,
                    };
                    if let Some(joker) = self.shop.jokers.iter_mut().find(|joker| {
                        joker.edition() == Edition::Base && !edited.contains(&joker.instance_id())
                    }) {
                        joker.set_edition(edition);
                        edited.insert(joker.instance_id());
                        self.shop.free_jokers.push(joker.instance_id());
                    } else {
                        pending.push((index, tag));
                    }
                }
                Tag::Coupon if opening && !coupon => coupon = true,
                Tag::D6 if opening && self.reroll_cost != 0 => self.reroll_cost = 0,
                Tag::Voucher if opening => {
                    let mut unavailable = self.vouchers.clone();
                    for offer in self.shop.voucher_offers() {
                        unavailable.redeem(offer);
                    }
                    let mut extra = None;
                    while let Some(voucher) = self
                        .backend
                        .gen_voucher(self.ante_current.number() as i32, &unavailable)
                    {
                        if self.vouchers.is_offerable(voucher) {
                            extra = Some(voucher);
                            break;
                        }
                        unavailable.redeem(voucher);
                    }
                    if let Some(voucher) = extra {
                        self.shop.extra_vouchers.push(voucher);
                    } else {
                        pending.push((index, tag));
                    }
                }
                _ => pending.push((index, tag)),
            }
        }
        pending.sort_unstable_by_key(|(index, _)| *index);
        self.tags = pending.into_iter().map(|(_, tag)| tag).collect();
        if coupon {
            self.shop.coupon();
        }
    }

    pub fn boss_reroll_available(&self) -> bool {
        self.stage == Stage::PreBlind()
            && self.current_boss.is_some()
            && self.money >= 10
            && (self.vouchers.has(Voucher::Retcon)
                || (self.vouchers.has(Voucher::DirectorsCut) && !self.boss_rerolled))
    }

    pub(super) fn replace_boss(&mut self) {
        let previous = self.current_boss;
        for _ in 0..32 {
            self.draw_ante_boss();
            if self.current_boss != previous {
                break;
            }
        }
        self.boss_rerolled = true;
    }
}
