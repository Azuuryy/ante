use super::*;

impl EngineRun {
    pub fn view(&self) -> RunView {
        let game = &self.game;
        let legal: Vec<_> = game.gen_actions().collect();
        let phase = phase_from(game.stage);
        let blind = if phase == Phase::BeforeBlind {
            legal.iter().find_map(|action| match action {
                Action::SelectBlind(blind) => Some(*blind),
                _ => None,
            })
        } else {
            game.blind
        };
        let blind_name = match blind {
            Some(Blind::Boss) => game
                .current_boss
                .as_ref()
                .map_or("Boss Blind", |boss| boss.name()),
            Some(blind) => blind_display(&blind),
            None => "",
        };
        let sequence = [Blind::Small, Blind::Big, Blind::Boss];
        let targets = sequence.map(|candidate| game.required_score_for(Some(candidate)));
        let active = sequence
            .iter()
            .position(|&candidate| Some(candidate) == blind);
        let target = if phase == Phase::BeforeBlind {
            active.map_or_else(|| game.required_score(), |index| targets[index])
        } else {
            game.required_score()
        };
        let blinds = {
            sequence
                .into_iter()
                .enumerate()
                .map(|(index, candidate)| {
                    let boss = (candidate == Blind::Boss)
                        .then_some(game.current_boss)
                        .flatten();
                    BlindView {
                        id: boss.map_or(
                            match candidate {
                                Blind::Small => "bl_small",
                                Blind::Big => "bl_big",
                                Blind::Boss => "bl_boss",
                            },
                            |boss| boss.id(),
                        ),
                        name: boss.map_or_else(
                            || blind_display(&candidate).into(),
                            |boss| boss.name().to_owned(),
                        ),
                        description: boss
                            .map_or_else(String::new, |boss| boss.description().to_owned()),
                        target: targets[index],
                        reward: candidate.reward(),
                        selectable: active == Some(index),
                        passed: active.is_some_and(|current| index < current),
                        skipped: self.skipped_blinds.contains(&candidate),
                        tag: game.skip_tag(candidate).map(|tag| {
                            let title = game
                                .orbital_targets
                                .get(index)
                                .copied()
                                .flatten()
                                .map_or_else(
                                    || tag.name().to_owned(),
                                    |rank| {
                                        format!("{} ({})", tag.name(), hand_view(game, rank).name())
                                    },
                                );
                            tag_view(tag, title)
                        }),
                        can_skip: legal.contains(&Action::SkipBlind(candidate)),
                    }
                })
                .collect()
        };
        let selected = game.available.selected();
        let cards: Vec<_> = game
            .available
            .cards()
            .into_iter()
            .map(|card| {
                let is_selected = selected.contains(&card);
                let action = if is_selected {
                    Action::DeselectCard(card)
                } else {
                    Action::SelectCard(card)
                };
                let suit = match card.suit {
                    Suit::Heart => "Hearts",
                    Suit::Diamond => "Diamonds",
                    Suit::Spade => "Spades",
                    Suit::Club => "Clubs",
                };
                CardView {
                    id: CardKey(card.id),
                    rank: char::from(card.value),
                    suit,
                    red: matches!(card.suit, Suit::Heart | Suit::Diamond),
                    selected: is_selected,
                    debuffed: game.is_card_debuffed(&card),
                    enabled: legal.contains(&action),
                    chips: card.chips(),
                    enhancement: card
                        .enhancement
                        .map_or_else(|| "None".into(), |v| format!("{v:?}")),
                    edition: format!("{:?}", card.edition),
                    seal: card
                        .seal
                        .map_or_else(|| "None".into(), |v| format!("{v:?}")),
                    effect_summary: card_effect_summary(card),
                }
            })
            .collect();
        let offers = game
            .shop
            .jokers
            .iter()
            .map(|joker| JokerView {
                id: JokerKey(joker.instance_id()),
                kind: joker.id(),
                name: joker.name(),
                price: game.shop.joker_price(joker, &game.vouchers),
                enabled: legal.contains(&Action::BuyJoker(joker.clone())),
                description: joker_description(joker.id()),
                state: format!("{} / {:?}", joker.state().counter, joker.state().selector),
                current: joker_current(joker, game),
                rarity: format!("{:?}", joker.rarity()),
                edition: format!("{:?}", joker.edition()),
                stickers: sticker_summary(joker.stickers()),
                can_sell: false,
                sell_price: joker.sell_value(),
            })
            .collect();

        let held_jokers = game
            .jokers
            .iter()
            .enumerate()
            .map(|(index, joker)| JokerView {
                id: JokerKey(joker.instance_id()),
                kind: joker.id(),
                name: joker.name(),
                price: game.price(joker.cost()),
                enabled: true,
                description: joker_description(joker.id()),
                state: format!("{} / {:?}", joker.state().counter, joker.state().selector),
                current: joker_current(joker, game),
                rarity: format!("{:?}", joker.rarity()),
                edition: format!("{:?}", joker.edition()),
                stickers: sticker_summary(joker.stickers()),
                can_sell: legal.contains(&Action::SellJoker(index)),
                sell_price: joker.sell_value(),
            })
            .collect();
        let consumables = game
            .consumables
            .iter()
            .enumerate()
            .map(|(index, c)| ConsumableView {
                index,
                kind: c.id(),
                name: c.name().to_owned(),
                category: c.type_label().to_owned(),
                description: consumable_description(*c),
                price: game.price(c.cost()),
                can_buy: false,
                can_buy_and_use: false,
                can_use: legal.contains(&Action::UseConsumable(*c)),
                can_sell: legal.contains(&Action::SellConsumable(index)),
                sell_price: c.sell_value(),
            })
            .collect();
        let consumable_offers = game
            .shop
            .consumables
            .iter()
            .enumerate()
            .map(|(index, c)| ConsumableView {
                index,
                kind: c.id(),
                name: c.name().to_owned(),
                category: c.type_label().to_owned(),
                description: consumable_description(*c),
                price: game.shop.consumable_price(c, &game.vouchers),
                can_buy: legal.contains(&Action::BuyConsumable(*c)),
                can_buy_and_use: game.can_buy_and_use(c),
                can_use: false,
                can_sell: false,
                sell_price: c.sell_value(),
            })
            .collect();

        let deck = game.deck.cards();
        let deck_cards = deck.len();
        let mut deck_view: Vec<_> = deck
            .into_iter()
            .map(|card| (card, true))
            .chain(game.available.cards().into_iter().map(|card| (card, false)))
            .chain(game.discarded.iter().copied().map(|card| (card, false)))
            .map(|(card, remaining)| DeckCardView {
                rank: char::from(card.value),
                suit: suit_index(card.suit),
                effective_suits: if game.is_card_debuffed(&card)
                    || card.enhancement == Some(Enhancement::Stone)
                {
                    0
                } else if card.enhancement == Some(Enhancement::Wild) {
                    0b1111
                } else {
                    1 << suit_index(card.suit)
                },
                edition: edition_name(card.edition),
                enhancement: card.enhancement.map(enhancement_name),
                seal: card.seal.map(seal_name),
                debuffed: game.is_card_debuffed(&card),
                remaining,
            })
            .collect();
        deck_view.sort_unstable_by_key(|card| {
            (
                card.suit,
                RANKS.find(card.rank),
                card.edition,
                card.enhancement,
                card.seal,
                !card.remaining,
            )
        });
        let boss = (blind == Some(Blind::Boss))
            .then_some(game.current_boss)
            .flatten();
        RunView {
            seed: game.seed,
            revision: self.revision,
            round: game.round,
            ante: game.ante_current.number(),
            ante_end: game.ante_end.number(),
            campaign_won: game.campaign_won,
            endless_mode: game.endless_mode,
            can_continue_endless: game.can_continue_endless(),
            stats: self.stats,
            deck_view,
            tags: game
                .tags
                .iter()
                .map(|tag| tag_view(*tag, tag.name().to_owned()))
                .collect(),
            shop_cards: game
                .shop
                .cards
                .iter()
                .map(|card| {
                    (
                        CardKey(card.id),
                        ShopItem {
                            name: card.to_string(),
                            description: card_effect_summary(*card),
                            art: card_art(*card),
                            price: game.shop.card_price(card, &game.vouchers),
                            enabled: legal.contains(&Action::BuyPlayingCard(*card)),
                        },
                    )
                })
                .collect(),
            packs: game
                .shop
                .packs
                .iter()
                .map(|pack| ShopItem {
                    name: pack.name(),
                    description: String::new(),
                    art: pack_art(pack),
                    price: game.shop.pack_price(pack, &game.vouchers),
                    enabled: legal.contains(&Action::BuyPack(pack.clone())),
                })
                .collect(),
            pack_choices: game
                .open_pack
                .iter()
                .flat_map(|pack| &pack.contents)
                .map(|content| ShopItem {
                    name: content.name(),
                    description: pack_content_description(content),
                    art: pack_content_art(content),
                    price: 0,
                    enabled: legal.contains(&Action::PickPackCard(content.clone())),
                })
                .collect(),
            pack_picks: game
                .open_pack
                .as_ref()
                .map_or(0, |pack| pack.picks_remaining),
            open_pack: game
                .open_pack
                .as_ref()
                .map(|open| open_pack_family(&open.contents)),
            can_apply_target: legal.contains(&Action::ApplyTarot())
                || legal.contains(&Action::ApplySpectral()),
            can_sort: legal.contains(&Action::SortHand(SortBy::Rank)),
            blind_chip: blind.map(|blind| match blind {
                Blind::Small => "bl_small",
                Blind::Big => "bl_big",
                Blind::Boss => boss.map_or("bl_boss", |boss| boss.id()),
            }),
            blind_rule: boss.map_or_else(String::new, |boss| boss.description().to_owned()),
            blinds,
            selected_hand: (phase == Phase::Playing)
                .then(|| SelectHand::new(game.available.selected()).best_hand().ok())
                .flatten()
                .map(|hand| hand_view(game, hand.rank)),
            hand_levels: [
                HandRank::FlushFive,
                HandRank::FlushHouse,
                HandRank::FiveOfAKind,
                HandRank::StraightFlush,
                HandRank::FourOfAKind,
                HandRank::FullHouse,
                HandRank::Flush,
                HandRank::Straight,
                HandRank::ThreeOfAKind,
                HandRank::TwoPair,
                HandRank::OnePair,
                HandRank::HighCard,
            ]
            .into_iter()
            .map(|rank| hand_view(game, rank))
            .collect(),
            vouchers: game
                .vouchers
                .owned()
                .iter()
                .copied()
                .map(voucher_content)
                .collect(),
            voucher_offers: game
                .shop
                .voucher_offers()
                .filter(|_| phase == Phase::Shop)
                .map(|voucher| {
                    (
                        voucher_content(voucher),
                        legal.contains(&Action::BuyVoucher(voucher)),
                    )
                })
                .collect(),
            can_reroll_boss: legal.contains(&Action::RerollBoss()),
            can_skip_pack: legal.contains(&Action::SkipPack()),
            deck_total: deck_cards + game.discarded.len() + cards.len(),
            phase,
            blind: blind_name.to_owned(),
            target,
            score: visible_score(game, phase),
            last_hand_score: self.last_hand_score,
            hands: game.plays,
            discards: game.discards(),
            money: game.money,
            reward: game.reward,
            deck_cards,
            discarded_cards: game.discarded.len(),
            cards,
            offers,
            jokers: game.jokers.iter().map(|joker| joker.name()).collect(),
            held_jokers,
            joker_slots: game.joker_slots(),
            consumables,
            consumable_offers,
            consumable_slots: game.consumable_slots(),
            reroll_cost: game.reroll_cost,
            can_reroll: legal.contains(&Action::Reroll()),
            can_play: legal.contains(&Action::Play()),
            can_discard: legal.contains(&Action::Discard()),
            debug: self.debug,
        }
    }
}

pub(super) fn hand_view(game: &Game, rank: HandRank) -> HandView {
    let level = game.planetarium.level(rank);
    HandView {
        rank: map_rank(rank),
        level: HandLevel {
            level: level.level,
            chips: level.chips,
            mult: level.mult,
            plays: level.plays,
        },
    }
}

pub(super) fn visible_score(game: &Game, phase: Phase) -> usize {
    if matches!(phase, Phase::Reward | Phase::Shop | Phase::Won) {
        game.last_score
    } else {
        game.score
    }
}
