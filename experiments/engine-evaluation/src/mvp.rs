use std::time::Duration;

use balatro_rs::{
    action::{Action, MoveDirection as EngineMoveDirection, SortBy},
    card::{Edition, Enhancement, Seal, Suit},
    config::{Config, RngMode},
    game::Game,
    hand::SelectHand,
    pack::PackContent,
    rank::HandRank,
    score::{ScoreSource as NativeSource, ScoreTrace as NativeTrace},
    stage::{Blind, BlindExt, End, Stage, blind_display},
    voucher::Voucher,
};

pub use ante_core::contract::*;
pub use ante_core::contract::{
    Command as MvpAction, Outcome as ActionOutcome, RunError as ActionError,
};

fn phase_from(stage: Stage) -> Phase {
    match stage {
        Stage::PreBlind() => Phase::BeforeBlind,
        Stage::Blind(_) => Phase::Playing,
        Stage::PostBlind() => Phase::Reward,
        Stage::Shop() => Phase::Shop,
        Stage::End(End::Win) => Phase::Won,
        Stage::End(End::Lose) => Phase::Lost,
        Stage::PackOpen() => Phase::Pack,
        Stage::TarotHand(_) | Stage::SpectralHand(_) => Phase::Target,
    }
}

const STARTING_MONEY: usize = 4;

pub struct MvpRun {
    game: Game,
    sort_by_suit: bool,
    last_hand_score: Option<usize>,
    revision: u64,
    skipped_blinds: Vec<Blind>,
    stats: RunStats,
    debug: bool,
}

impl MvpRun {
    pub fn new(seed: u64) -> Self {
        Self::with_config(RunConfig {
            seed,
            deck: Deck::Red,
            goal: CampaignGoal::Standard,
            starting_money: None,
        })
    }

    pub fn with_config(run: RunConfig) -> Self {
        let (plays, discards) = match run.deck {
            Deck::Red => (4, 4),
            Deck::Blue => (5, 3),
        };
        let config = Config {
            seed: Some(run.seed),
            money_start: run.starting_money.unwrap_or(STARTING_MONEY),
            plays,
            discards,
            ante_end: if run.goal == CampaignGoal::SingleAnte {
                1
            } else {
                8
            },
            rng_mode: RngMode::Fast,
            ..Config::default()
        };
        let mut game = Game::new(config);
        game.start();
        Self {
            game,
            sort_by_suit: false,
            last_hand_score: None,
            revision: 0,
            skipped_blinds: Vec::new(),
            stats: RunStats::default(),
            debug: false,
        }
    }

    pub fn phase(&self) -> Phase {
        phase_from(self.game.stage)
    }

    pub fn snapshot(&self) -> Result<String, serde_json::Error> {
        self.game.to_json()
    }

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

    /// Applies a Dev shortcut outside the score trace, like the original debug panel.
    fn apply_debug(&mut self, command: DebugCommand) -> Result<ActionOutcome, ActionError> {
        let game = &mut self.game;
        let phase = phase_from(game.stage);
        let playing = phase == Phase::Playing;
        let simulation =
            |error: balatro_rs::error::GameError| ActionError::Simulation(error.to_string());
        match command {
            DebugCommand::AddMoney(amount) => game.money += amount,
            DebugCommand::AddHand if playing => game.plays += 1,
            DebugCommand::AddDiscard if playing => game.discards_remaining += 1,
            DebugCommand::AddRound => game.round += 1,
            DebugCommand::AddAnte if !playing && !matches!(phase, Phase::Won | Phase::Lost) => {
                game.debug_next_ante().map_err(simulation)?;
                self.skipped_blinds.clear();
            }
            DebugCommand::AddScore(amount) if playing => {
                game.debug_add_score(amount).map_err(simulation)?;
            }
            DebugCommand::WinBlind if playing => {
                let missing = game.required_score().saturating_sub(game.score);
                game.debug_add_score(missing).map_err(simulation)?;
            }
            DebugCommand::LoseRun if !matches!(phase, Phase::Won | Phase::Lost) => {
                game.stage = Stage::End(End::Lose);
            }
            _ => return Err(ActionError::Unavailable),
        }
        self.debug = true;
        self.revision = self.revision.wrapping_add(1);
        let phase = phase_from(game.stage);
        Ok(ActionOutcome {
            phase,
            round_score: visible_score(game, phase),
            campaign_won: game.campaign_won,
            hand_score: self.last_hand_score,
            trace: None,
            trace_time: Duration::ZERO,
            discovered_item: None,
        })
    }

    pub fn apply(&mut self, action: MvpAction) -> Result<(), ActionError> {
        self.resolve(action, None).map(|_| ())
    }

    pub fn apply_traced(
        &mut self,
        action: MvpAction,
        profile_trace: bool,
    ) -> Result<ActionOutcome, ActionError> {
        self.resolve(action, Some(profile_trace))
    }

    fn resolve(
        &mut self,
        action: MvpAction,
        profile_trace: Option<bool>,
    ) -> Result<ActionOutcome, ActionError> {
        if let MvpAction::Debug(command) = action {
            return self.apply_debug(command);
        }
        let game = &mut self.game;
        if action == MvpAction::ContinueEndless {
            game.continue_endless()
                .map_err(|_| ActionError::Unavailable)?;
            self.skipped_blinds.clear();
            self.revision = self.revision.wrapping_add(1);
            return Ok(ActionOutcome {
                phase: Phase::Reward,
                round_score: game.last_score,
                campaign_won: game.campaign_won,
                hand_score: self.last_hand_score,
                trace: None,
                trace_time: Duration::ZERO,
                discovered_item: None,
            });
        }
        match action {
            MvpAction::MoveCard { id, direction }
                if matches!(phase_from(game.stage), Phase::Playing | Phase::Target) =>
            {
                let card = game
                    .available
                    .cards()
                    .into_iter()
                    .find(|card| card.id == id.0)
                    .ok_or(ActionError::Unavailable)?;
                game.handle_action(Action::MoveCard(
                    match direction {
                        MoveDirection::Left => EngineMoveDirection::Left,
                        MoveDirection::Right => EngineMoveDirection::Right,
                    },
                    card,
                ))
                .map_err(|error| ActionError::Simulation(error.to_string()))?;
                self.revision = self.revision.wrapping_add(1);
                let phase = phase_from(game.stage);
                return Ok(ActionOutcome {
                    phase,
                    round_score: visible_score(game, phase),
                    campaign_won: game.campaign_won,
                    hand_score: self.last_hand_score,
                    trace: None,
                    trace_time: Duration::ZERO,
                    discovered_item: None,
                });
            }
            MvpAction::ReorderJoker { from, to, revision }
                if revision == self.revision
                    && from < game.jokers.len()
                    && to < game.jokers.len() =>
            {
                let item = game.jokers.remove(from);
                game.jokers.insert(to, item);
                let jokers = game.jokers.clone();
                game.effect_registry.register_jokers(jokers, &game.clone());
                self.revision = self.revision.wrapping_add(1);
                let phase = phase_from(game.stage);
                return Ok(ActionOutcome {
                    phase,
                    round_score: visible_score(game, phase),
                    campaign_won: game.campaign_won,
                    hand_score: self.last_hand_score,
                    trace: None,
                    trace_time: Duration::ZERO,
                    discovered_item: None,
                });
            }
            MvpAction::ReorderConsumable { from, to, revision }
                if revision == self.revision
                    && from < game.consumables.len()
                    && to < game.consumables.len() =>
            {
                let item = game.consumables.remove(from);
                game.consumables.insert(to, item);
                self.revision = self.revision.wrapping_add(1);
                let phase = phase_from(game.stage);
                return Ok(ActionOutcome {
                    phase,
                    round_score: visible_score(game, phase),
                    campaign_won: game.campaign_won,
                    hand_score: self.last_hand_score,
                    trace: None,
                    trace_time: Duration::ZERO,
                    discovered_item: None,
                });
            }
            MvpAction::ReorderJoker { .. } | MvpAction::ReorderConsumable { .. } => {
                return Err(ActionError::Unavailable);
            }
            _ => {}
        }
        let ante_before = game.ante_current;
        let selected_count = game.available.selected().len();
        let indexed = match action {
            MvpAction::SkipBlind { revision } | MvpAction::RedeemVoucher { revision, .. } => {
                Some(revision)
            }
            MvpAction::BuyPlayingCard { revision, .. }
            | MvpAction::BuyPack { revision, .. }
            | MvpAction::PickPack { revision, .. }
            | MvpAction::BuyConsumable { revision, .. }
            | MvpAction::UseConsumable { revision, .. }
            | MvpAction::SellConsumable { revision, .. }
            | MvpAction::SellJoker { revision, .. } => Some(revision),
            _ => None,
        };
        if indexed.is_some_and(|revision| revision != self.revision) {
            return Err(ActionError::Unavailable);
        }
        let native = game
            .gen_actions()
            .find(|candidate| match (action, candidate) {
                (MvpAction::StartBlind, Action::SelectBlind(_))
                | (MvpAction::SkipBlind { .. }, Action::SkipBlind(_))
                | (MvpAction::SortRank, Action::SortHand(SortBy::Rank))
                | (MvpAction::SortSuit, Action::SortHand(SortBy::Suit))
                | (MvpAction::Play, Action::Play())
                | (MvpAction::Discard, Action::Discard())
                | (MvpAction::CashOut, Action::CashOut(_))
                | (MvpAction::LeaveShop, Action::NextRound())
                | (MvpAction::RerollBoss, Action::RerollBoss()) => true,
                (MvpAction::RedeemVoucher { voucher, .. }, Action::BuyVoucher(offered)) => {
                    voucher.0 == offered.id()
                }
                (
                    MvpAction::ToggleCard(id),
                    Action::SelectCard(card) | Action::DeselectCard(card),
                ) => id.0 == card.id,
                (MvpAction::BuyPlayingCard { id, .. }, Action::BuyPlayingCard(card)) => {
                    id.0 == card.id
                }
                (MvpAction::BuyPack { index, .. }, Action::BuyPack(pack)) => {
                    game.shop.packs.get(index) == Some(pack)
                }
                (MvpAction::PickPack { index, .. }, Action::PickPackCard(content)) => {
                    game.open_pack
                        .as_ref()
                        .and_then(|pack| pack.contents.get(index))
                        == Some(content)
                }
                (MvpAction::ApplyTarget, Action::ApplyTarot() | Action::ApplySpectral())
                | (MvpAction::SkipPack, Action::SkipPack()) => true,
                (MvpAction::BuyJoker(id), Action::BuyJoker(joker)) => id.0 == joker.instance_id(),
                (MvpAction::BuyConsumable { index, .. }, Action::BuyConsumable(c)) => {
                    game.shop.consumables.get(index) == Some(c)
                }
                (MvpAction::UseConsumable { index, .. }, Action::UseConsumable(c)) => {
                    game.consumables.get(index) == Some(c)
                }
                (MvpAction::SellConsumable { index, .. }, Action::SellConsumable(i)) => index == *i,
                (MvpAction::SellJoker { index, .. }, Action::SellJoker(i)) => index == *i,
                (MvpAction::Reroll, Action::Reroll()) => true,
                _ => false,
            })
            .ok_or(ActionError::Unavailable)?;
        if action == MvpAction::SortRank {
            self.sort_by_suit = false;
        } else if action == MvpAction::SortSuit {
            self.sort_by_suit = true;
        }
        if action == MvpAction::Play {
            let selected = game.available.selected();
            let sorted_ids = game
                .available
                .cards()
                .into_iter()
                .filter(|card| selected.contains(card))
                .map(|card| card.id)
                .collect::<Vec<_>>();
            for card in selected {
                game.handle_action(Action::DeselectCard(card))
                    .map_err(|error| ActionError::Simulation(error.to_string()))?;
            }
            for id in sorted_ids {
                let card = game
                    .available
                    .cards()
                    .into_iter()
                    .find(|card| card.id == id)
                    .ok_or(ActionError::Unavailable)?;
                game.handle_action(Action::SelectCard(card))
                    .map_err(|error| ActionError::Simulation(error.to_string()))?;
            }
        }
        let skipped = if let Action::SkipBlind(blind) = native {
            Some(blind)
        } else {
            None
        };
        let before = game.score;
        let discovered_item = match &native {
            Action::BuyJoker(joker) | Action::PickPackCard(PackContent::Joker(joker)) => {
                Some(joker.id())
            }
            Action::BuyVoucher(voucher) => Some(voucher.id()),
            Action::SkipBlind(blind) => game.skip_tag(*blind).map(|tag| tag.id()),
            Action::BuyConsumable(card) | Action::UseConsumable(card) => Some(card.id()),
            Action::PickPackCard(PackContent::Tarot(card)) => Some(card.id()),
            Action::PickPackCard(PackContent::Planet(card)) => Some(card.id()),
            Action::PickPackCard(PackContent::Spectral(card)) => Some(card.id()),
            Action::BuyPack(pack) => match pack_art(pack) {
                ShopArt::Pack { family, size } => Some(ante_core::contract::booster_id(family, size)),
                _ => None,
            },
            Action::SelectBlind(balatro_rs::stage::Blind::Boss) => {
                game.current_boss.map(|boss| boss.id())
            }
            _ => None,
        };
        let (trace, trace_time) = if let (MvpAction::Play, Some(profile)) = (action, profile_trace)
        {
            let (trace, elapsed) = game
                .handle_action_traced(native, profile)
                .map_err(|error| ActionError::Simulation(error.to_string()))?;
            (Some(map_trace(trace)), elapsed)
        } else {
            game.handle_action(native)
                .map_err(|error| ActionError::Simulation(error.to_string()))?;
            (None, Duration::ZERO)
        };

        if matches!(
            action,
            MvpAction::StartBlind | MvpAction::Play | MvpAction::Discard
        ) && phase_from(game.stage) == Phase::Playing
            && game
                .gen_actions()
                .any(|candidate| matches!(candidate, Action::SortHand(_)))
        {
            let sort = if self.sort_by_suit {
                Action::SortHand(SortBy::Suit)
            } else {
                Action::SortHand(SortBy::Rank)
            };
            game.handle_action(sort)
                .map_err(|error| ActionError::Simulation(error.to_string()))?;
        }

        if ante_before != game.ante_current {
            self.skipped_blinds.clear();
        }
        if let Some(blind) = skipped {
            self.skipped_blinds.push(blind);
        }
        self.revision = self.revision.wrapping_add(1);
        match action {
            MvpAction::StartBlind => self.last_hand_score = None,
            MvpAction::Play => {
                self.stats.played += selected_count;
                let after = if matches!(game.stage, Stage::PostBlind() | Stage::End(End::Win)) {
                    game.last_score
                } else {
                    game.score
                };
                let hand = after.saturating_sub(before);
                self.last_hand_score = Some(hand);
                self.stats.best_hand = self.stats.best_hand.max(hand);
            }
            MvpAction::Discard => self.stats.discarded += selected_count,
            MvpAction::BuyJoker(_)
            | MvpAction::BuyConsumable { .. }
            | MvpAction::RedeemVoucher { .. } => self.stats.purchased += 1,
            MvpAction::Reroll => self.stats.rerolls += 1,
            _ => {}
        }
        let phase = phase_from(game.stage);
        Ok(ActionOutcome {
            phase,
            round_score: visible_score(game, phase),
            campaign_won: game.campaign_won,
            hand_score: self.last_hand_score,
            trace,
            trace_time,
            discovered_item,
        })
    }
}

fn hand_view(game: &Game, rank: HandRank) -> HandView {
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

fn map_rank(rank: HandRank) -> HandKind {
    match rank {
        HandRank::HighCard => HandKind::HighCard,
        HandRank::OnePair => HandKind::OnePair,
        HandRank::TwoPair => HandKind::TwoPair,
        HandRank::ThreeOfAKind => HandKind::ThreeOfAKind,
        HandRank::Straight => HandKind::Straight,
        HandRank::Flush => HandKind::Flush,
        HandRank::FullHouse => HandKind::FullHouse,
        HandRank::FourOfAKind => HandKind::FourOfAKind,
        HandRank::StraightFlush => HandKind::StraightFlush,
        HandRank::RoyalFlush => HandKind::RoyalFlush,
        HandRank::FiveOfAKind => HandKind::FiveOfAKind,
        HandRank::FlushHouse => HandKind::FlushHouse,
        HandRank::FlushFive => HandKind::FlushFive,
    }
}

fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

fn suit_name(suit: Suit) -> &'static str {
    SUITS[suit_index(suit)]
}

fn edition_name(value: Edition) -> &'static str {
    match value {
        Edition::Base => "Base",
        Edition::Foil => "Foil",
        Edition::Holographic => "Holographic",
        Edition::Polychrome => "Polychrome",
        Edition::Negative => "Negative",
    }
}

fn enhancement_name(value: Enhancement) -> &'static str {
    match value {
        Enhancement::Bonus => "Bonus",
        Enhancement::Mult => "Mult",
        Enhancement::Wild => "Wild",
        Enhancement::Glass => "Glass",
        Enhancement::Steel => "Steel",
        Enhancement::Stone => "Stone",
        Enhancement::Gold => "Gold",
        Enhancement::Lucky => "Lucky",
    }
}

fn seal_name(value: Seal) -> &'static str {
    match value {
        Seal::Gold => "Gold",
        Seal::Red => "Red",
        Seal::Blue => "Blue",
        Seal::Purple => "Purple",
    }
}

fn card_art(card: balatro_rs::card::Card) -> ShopArt {
    ShopArt::Card {
        rank: char::from(card.value),
        suit: suit_name(card.suit),
    }
}

fn pack_art(pack: &balatro_rs::pack::Pack) -> ShopArt {
    use balatro_rs::pack::{PackCategory as Category, PackSize as Size};
    ShopArt::Pack {
        family: match pack.category {
            Category::Arcana => PackFamily::Arcana,
            Category::Celestial => PackFamily::Celestial,
            Category::Spectral => PackFamily::Spectral,
            Category::Standard => PackFamily::Standard,
            Category::Buffoon => PackFamily::Buffoon,
        },
        size: match pack.size {
            Size::Normal => PackSize::Normal,
            Size::Jumbo => PackSize::Jumbo,
            Size::Mega => PackSize::Mega,
        },
    }
}

fn pack_content_art(content: &PackContent) -> ShopArt {
    match content {
        PackContent::PlayingCard(card) => card_art(*card),
        PackContent::Joker(joker) => ShopArt::Joker(joker.id()),
        PackContent::Tarot(card) => ShopArt::Consumable(card.id()),
        PackContent::Planet(card) => ShopArt::Consumable(card.id()),
        PackContent::Spectral(card) => ShopArt::Consumable(card.id()),
    }
}

fn pack_content_description(content: &PackContent) -> String {
    use balatro_rs::consumable::Consumable;
    match content {
        PackContent::PlayingCard(card) => card_effect_summary(*card),
        PackContent::Joker(joker) => joker_description(joker.id()),
        PackContent::Tarot(card) => consumable_description(Consumable::Tarot(*card)),
        PackContent::Planet(card) => consumable_description(Consumable::Planet(*card)),
        PackContent::Spectral(card) => consumable_description(Consumable::Spectral(*card)),
    }
}

fn trace_card(card: balatro_rs::card::Card, trigger: CardTrigger) -> ScoreSource {
    ScoreSource::Card {
        trigger,
        id: CardKey(card.id),
        rank: char::from(card.value),
        suit: suit_name(card.suit),
    }
}

fn map_trace(trace: NativeTrace) -> ScoreTrace {
    ScoreTrace(
        trace
            .0
            .into_iter()
            .map(|step| ScoreStep {
                source: match step.source {
                    NativeSource::PlayedCard(card) => trace_card(card, CardTrigger::Played),
                    NativeSource::StoneKicker(card) => trace_card(card, CardTrigger::StoneKicker),
                    NativeSource::HeldCard(card) => trace_card(card, CardTrigger::Held),
                    NativeSource::HandLevel(rank) => ScoreSource::HandLevel(map_rank(rank)),
                    NativeSource::Joker(joker) => ScoreSource::Joker {
                        id: JokerKey(joker.instance_id()),
                        kind: ContentId(joker.id()),
                        name: joker.name(),
                    },
                    NativeSource::Voucher(voucher) => {
                        ScoreSource::Voucher(ContentId(voucher.id()), voucher_name(voucher))
                    }
                },
                chips_before: step.chips_before,
                chips_after: step.chips_after,
                mult_before: step.mult_before,
                mult_after: step.mult_after,
                retrigger: step.retrigger,
                xmult: step
                    .xmult
                    .map(|factor| (factor * 100.0).round() as usize),
                money: step.money,
            })
            .collect(),
    )
}

fn voucher_name(voucher: Voucher) -> &'static str {
    VOUCHERS
        .iter()
        .find(|item| item.id.0 == voucher.id())
        .unwrap()
        .name
        .as_str()
}

use balatro_rs::tag::Tag;
use std::sync::LazyLock;
use strum::IntoEnumIterator;

static VOUCHERS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    Voucher::iter()
        .map(|voucher| Content {
            id: ContentId(voucher.id()),
            name: voucher.name().to_owned(),
            description: voucher.description().to_owned(),
            cost: voucher.cost(),
            supported: true,
            unlock: unlock_condition(voucher.id()),
        })
        .collect()
});

fn voucher_content(voucher: Voucher) -> &'static Content {
    VOUCHERS
        .iter()
        .find(|item| item.id.0 == voucher.id())
        .unwrap()
}

fn consumable_content(card: balatro_rs::consumable::Consumable) -> Content {
    Content {
        id: ContentId(card.id()),
        name: card.name().to_owned(),
        description: consumable_description(card),
        cost: card.cost(),
        supported: true,
        unlock: None,
    }
}

static TAROTS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    balatro_rs::tarot::Tarot::iter()
        .map(|card| consumable_content(balatro_rs::consumable::Consumable::Tarot(card)))
        .collect()
});

static PLANETS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    balatro_rs::planet::Planets::iter()
        .map(|card| consumable_content(balatro_rs::consumable::Consumable::Planet(card)))
        .collect()
});

static SPECTRALS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    balatro_rs::spectral::Spectral::iter()
        .map(|card| consumable_content(balatro_rs::consumable::Consumable::Spectral(card)))
        .collect()
});

/// Pack contents and price by size: choose 1 of 3, 1 of 5 or 2 of 5 (2, 4 and 4 for
/// Spectral and Buffoon packs), for $4, $6 or $8.
static BOOSTER_CONTENT: LazyLock<Vec<Content>> = LazyLock::new(|| {
    use ante_core::contract::{BOOSTERS, PackFamily, PackSize};
    BOOSTERS
        .iter()
        .map(|&(id, name, family, size)| {
            let small = matches!(family, PackFamily::Spectral | PackFamily::Buffoon);
            let (choose, shown, cost) = match size {
                PackSize::Normal => (1, if small { 2 } else { 3 }, 4),
                PackSize::Jumbo => (1, if small { 4 } else { 5 }, 6),
                PackSize::Mega => (2, if small { 4 } else { 5 }, 8),
            };
            let what = match family {
                PackFamily::Arcana => "Tarot cards, used at once",
                PackFamily::Celestial => "Planet cards, used at once",
                PackFamily::Spectral => "Spectral cards, used at once",
                PackFamily::Standard => "playing cards added to your deck",
                PackFamily::Buffoon => "Jokers",
            };
            Content {
                id: ContentId(id),
                name: name.to_owned(),
                description: format!("Pick {choose} of {shown} {what}."),
                cost,
                supported: true,
                unlock: None,
            }
        })
        .collect()
});

static BLINDS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    use balatro_types::BossBlind;
    let mut blinds = vec![
        Content {
            id: ContentId("bl_small"),
            name: "Small Blind".to_owned(),
            description: "No special effect.".to_owned(),
            cost: 0,
            supported: true,
            unlock: None,
        },
        Content {
            id: ContentId("bl_big"),
            name: "Big Blind".to_owned(),
            description: "No special effect.".to_owned(),
            cost: 0,
            supported: true,
            unlock: None,
        },
    ];
    blinds.extend(BossBlind::iter().map(|boss| Content {
        id: ContentId(boss.id()),
        name: boss.name().to_owned(),
        description: boss.description().to_owned(),
        cost: 0,
        supported: true,
        unlock: None,
    }));
    blinds
});

fn rule(id: &'static str, name: &str, description: &str) -> Content {
    Content {
        id: ContentId(id),
        name: name.to_owned(),
        description: description.to_owned(),
        cost: 0,
        supported: true,
        unlock: None,
    }
}

/// Ante's own summaries of each rule, matching the tooltip side boxes.
static ENHANCEMENTS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    vec![
        rule("m_bonus", "Bonus Card", "+30 Chips"),
        rule("m_mult", "Mult Card", "+4 Mult"),
        rule("m_wild", "Wild Card", "Counts as every suit"),
        rule("m_glass", "Glass Card", "X2 Mult, 1 in 4 chance to break after scoring"),
        rule("m_steel", "Steel Card", "X1.5 Mult while held in hand"),
        rule("m_stone", "Stone Card", "+50 Chips; has no rank or suit and always scores"),
        rule("m_gold", "Gold Card", "Earn $3 if held in hand at end of round"),
        rule("m_lucky", "Lucky Card", "+20 Mult on a 1 in 5 chance; $20 on a 1 in 15 chance"),
    ]
});

static EDITIONS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    vec![
        rule("e_foil", "Foil", "+50 Chips"),
        rule("e_holo", "Holographic", "+10 Mult"),
        rule("e_polychrome", "Polychrome", "X1.5 Mult"),
        rule("e_negative", "Negative", "+1 Joker slot"),
    ]
});

static SEALS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    vec![
        rule("gold_seal", "Gold Seal", "Played and scoring: earn $3"),
        rule("red_seal", "Red Seal", "This card triggers 1 more time"),
        rule(
            "blue_seal",
            "Blue Seal",
            "Held in hand at end of round: creates the Planet of the last played hand",
        ),
        rule("purple_seal", "Purple Seal", "Discarding this card creates a Tarot card"),
    ]
});

static TAGS: LazyLock<Vec<Content>> = LazyLock::new(|| {
    Tag::iter()
        .map(|tag| Content {
            id: ContentId(tag.id()),
            name: tag.name().to_owned(),
            description: tag.description().to_owned(),
            cost: 0,
            supported: true,
            unlock: None,
        })
        .collect()
});
/// Collection unlock text for items that are not available from the start.
/// Conditions are paraphrased from the reference rules; they are display
/// metadata only and do not drive unlock progression yet.
pub fn unlock_condition(id: &str) -> Option<&'static str> {
    Some(match id {
        "j_ticket" => "Play a hand containing 5 Gold cards.",
        "j_mr_bones" => "Lose 5 runs.",
        "j_acrobat" => "Play 200 hands.",
        "j_sock_and_buskin" => "Play 300 face cards.",
        "j_swashbuckler" => "Sell 20 Jokers.",
        "j_troubadour" => "Win 5 consecutive rounds in a single hand.",
        "j_certificate" => "Score with Gold cards in a hand.",
        "j_smeared" => "Have 3 Wild cards in your deck.",
        "j_throwback" => "Continue a saved run.",
        "j_hanging_chad" => "Defeat a Boss Blind with a High Card hand.",
        "j_rough_gem" => "Have 30 Diamonds in your deck.",
        "j_bloodstone" => "Have 30 Hearts in your deck.",
        "j_arrowhead" => "Have 30 Spades in your deck.",
        "j_onyx_agate" => "Have 30 Clubs in your deck.",
        "j_glass" => "Have 5 Glass cards in your deck.",
        "j_ring_master" => "Reach Ante 4.",
        "j_flower_pot" => "Reach Ante 8.",
        "j_blueprint" => "Win a run.",
        "j_wee" => "Win a run in 18 rounds or fewer.",
        "j_merry_andy" => "Win a run in 12 rounds or fewer.",
        "j_oops" => "Score 10,000 chips in one hand.",
        "j_idol" => "Score 1,000,000 chips in one hand.",
        "j_seeing_double" => "Play four 7 of Clubs in one hand.",
        "j_matador" => "Defeat a Boss Blind in one hand without discarding.",
        "j_hit_the_road" => "Discard 5 Jacks at once.",
        "j_duo" => "Win a run without playing a Pair.",
        "j_trio" => "Win a run without playing Three of a Kind.",
        "j_family" => "Win a run without playing Four of a Kind.",
        "j_order" => "Win a run without playing a Straight.",
        "j_tribe" => "Win a run without playing a Flush.",
        "j_stuntman" => "Score 100,000,000 chips in one hand.",
        "j_invisible" => "Win a run with 4 or fewer Joker slots.",
        "j_brainstorm" => "Discard a Royal Flush.",
        "j_satellite" => "Have $400 at once.",
        "j_shoot_the_moon" => "Play every Heart from your deck in one hand.",
        "j_drivers_license" => "Have 16 enhanced cards in your deck.",
        "j_cartomancer" => "Discover 22 Tarot cards.",
        "j_astronomer" => "Discover 12 Planet cards.",
        "j_burnt" => "Sell 50 cards.",
        "j_bootstraps" => "Have 2 Polychrome Jokers.",
        "j_caino" | "j_triboulet" | "j_yorick" | "j_chicot" | "j_perkeo" => {
            "Discover it from a Spectral pack."
        }
        "v_overstock_plus" => "Spend $2,500 in the shop.",
        "v_liquidation" => "Redeem 10 Vouchers in one run.",
        "v_glow_up" => "Have 5 Jokers with editions.",
        "v_reroll_glut" => "Reroll the shop 100 times.",
        "v_omen_globe" => "Use 25 Tarot cards.",
        "v_observatory" => "Use 25 Planet cards.",
        "v_nacho_tong" => "Play 2,500 cards.",
        "v_recyclomancy" => "Discard 2,500 cards.",
        "v_tarot_tycoon" => "Buy 50 Tarot cards.",
        "v_planet_tycoon" => "Buy 50 Planet cards.",
        "v_money_tree" => "Reach maximum interest in 10 consecutive rounds.",
        "v_antimatter" => "Redeem the Blank Voucher 10 times.",
        "v_illusion" => "Buy 20 playing cards.",
        "v_petroglyph" => "Reach Ante 12.",
        "v_retcon" => "Discover 25 Blinds.",
        "v_palette" => "Win a round with a hand size of 5 or less.",
        _ => return None,
    })
}

fn tag_view(tag: balatro_rs::tag::Tag, name: String) -> TagView {
    TagView {
        content: TAGS
            .iter()
            .find(|item| item.id.0 == tag.id())
            .expect("known tag"),
        name,
    }
}

pub struct ExternalBackend;
impl GameBackend for ExternalBackend {
    fn start(&self, config: RunConfig) -> Box<dyn RunDriver> {
        Box::new(MvpRun::with_config(config))
    }

    fn catalog(&self, group: ContentGroup) -> &'static [Content] {
        use balatro_rs::joker::{JokerEffects, joker_catalog};
        static JOKERS: LazyLock<Vec<Content>> = LazyLock::new(|| {
            joker_catalog()
                .map(|joker| Content {
                    id: ContentId(joker.id()),
                    name: joker.name().to_owned(),
                    description: joker_description(joker.id()),
                    cost: joker.cost(),
                    supported: joker.is_implemented(),
                    unlock: unlock_condition(joker.id()),
                })
                .collect()
        });
        match group {
            ContentGroup::Jokers => &JOKERS,
            ContentGroup::Vouchers => &VOUCHERS,
            ContentGroup::Tags => &TAGS,
            ContentGroup::Tarots => &TAROTS,
            ContentGroup::Planets => &PLANETS,
            ContentGroup::Spectrals => &SPECTRALS,
            ContentGroup::Boosters => &BOOSTER_CONTENT,
            ContentGroup::Blinds => &BLINDS,
            ContentGroup::Enhancements => &ENHANCEMENTS,
            ContentGroup::Editions => &EDITIONS,
            ContentGroup::Seals => &SEALS,
        }
    }

    fn parse_seed(&self, seed: &str) -> u64 {
        balatro_rs::seed_from_str(seed)
    }
}

impl RunDriver for MvpRun {
    fn view(&self) -> RunView {
        MvpRun::view(self)
    }

    fn apply(&mut self, command: Command, trace: TraceMode) -> Result<RunUpdate, RunError> {
        let before = self.phase();
        let won = self.game.campaign_won;
        let outcome = self.resolve(
            command,
            match trace {
                TraceMode::Off => None,
                TraceMode::Capture => Some(false),
                TraceMode::Profile => Some(true),
            },
        )?;
        Ok(RunUpdate::confirmed(
            before,
            won,
            command,
            outcome,
            self.view(),
        ))
    }
}

fn card_effect_summary(card: balatro_rs::card::Card) -> String {
    use balatro_rs::card::{Edition, Enhancement, Seal};
    let mut parts = Vec::new();
    match card.enhancement {
        Some(Enhancement::Bonus) => parts.push("+30 chips when scored"),
        Some(Enhancement::Mult) => parts.push("+4 Mult when scored"),
        Some(Enhancement::Wild) => parts.push("Counts as any suit"),
        Some(Enhancement::Glass) => parts.push("Doubles Mult when scored; may break"),
        Some(Enhancement::Steel) => parts.push("Mult x1.5 while held"),
        Some(Enhancement::Stone) => parts.push("Stone card: scores without its rank"),
        Some(Enhancement::Gold) => parts.push("Pays $3 while held"),
        Some(Enhancement::Lucky) => parts.push("Chance for Mult or money when scored"),
        None => {}
    }
    match card.edition {
        Edition::Foil => parts.push("Foil: +50 chips"),
        Edition::Holographic => parts.push("Holographic: +10 Mult"),
        Edition::Polychrome => parts.push("Polychrome: Mult x1.5"),
        Edition::Negative => parts.push("Negative edition"),
        Edition::Base => {}
    }
    match card.seal {
        Some(Seal::Gold) => parts.push("Gold seal: earns $3 when scored"),
        Some(Seal::Red) => parts.push("Red seal: repeats activation"),
        Some(Seal::Blue) => parts.push("Blue seal: may create a planet at round end"),
        Some(Seal::Purple) => parts.push("Purple seal: creates a tarot on discard if space allows"),
        None => {}
    }
    parts.join("; ")
}

fn consumable_description(c: balatro_rs::consumable::Consumable) -> String {
    use balatro_rs::consumable::Consumable;
    match c {
        Consumable::Planet(p) => format!("Improves the {:?} hand.", p.hand_rank()),
        Consumable::Tarot(t) => {
            use balatro_rs::tarot::Tarot;
            let effect = match t {
                Tarot::Magician => "Adds Lucky to selected cards.",
                Tarot::Empress => "Adds Mult enhancement to selected cards.",
                Tarot::Hierophant => "Adds Bonus enhancement to selected cards.",
                Tarot::Lovers => "Adds Wild enhancement to selected cards.",
                Tarot::Chariot => "Adds Steel enhancement to selected cards.",
                Tarot::Justice => "Adds Glass enhancement to selected cards.",
                Tarot::Devil => "Adds Gold enhancement to selected cards.",
                Tarot::Tower => "Adds Stone enhancement to selected cards.",
                Tarot::Star => "Changes selected cards to diamonds.",
                Tarot::Moon => "Changes selected cards to clubs.",
                Tarot::Sun => "Changes selected cards to hearts.",
                Tarot::World => "Changes selected cards to spades.",
                Tarot::Strength => "Raises selected ranks by one.",
                Tarot::HangedMan => "Destroys the selected cards.",
                Tarot::Death => "Copies the second selected card onto the first.",
                Tarot::Hermit => "Doubles current money, adding at most $20.",
                Tarot::Temperance => "Pays the total sell value of jokers, up to $50.",
                Tarot::WheelOfFortune => "May add a random edition to an owned joker.",
                Tarot::HighPriestess => "Creates up to two random planet cards if there is room.",
                Tarot::Emperor => "Creates up to two random tarot cards if there is room.",
                Tarot::Judgement => "Creates a random joker if a slot is free.",
                Tarot::Fool => "Copies the last tarot or planet used, if there is room.",
            };
            if t.requires_hand() {
                format!(
                    "{effect} Select {} to {} cards.",
                    t.min_targets(),
                    t.max_targets()
                )
            } else {
                effect.into()
            }
        }
        Consumable::Spectral(s) => {
            use balatro_rs::spectral::Spectral;
            let effect = match s {
                Spectral::Talisman => "Adds a gold seal to a selected card.",
                Spectral::DejaVu => "Adds a red seal to a selected card.",
                Spectral::Trance => "Adds a blue seal to a selected card.",
                Spectral::Medium => "Adds a purple seal to a selected card.",
                Spectral::Aura => "Adds a random edition to a selected card.",
                Spectral::Cryptid => "Creates two copies of a selected card.",
                Spectral::Familiar => "Destroys a random card and adds three enhanced face cards.",
                Spectral::Grim => "Destroys a random card and adds two enhanced aces.",
                Spectral::Incantation => {
                    "Destroys a random card and adds four enhanced numbered cards."
                }
                Spectral::Sigil => "Changes the entire hand to one random suit.",
                Spectral::Ouija => "Changes the hand to one random rank and reduces hand size.",
                Spectral::Immolate => "Destroys up to five random cards and grants $20.",
                Spectral::BlackHole => "Improves every poker hand level.",
                Spectral::Wraith => "Creates a rare joker if possible and sets money to zero.",
                Spectral::Soul => "Creates a legendary joker if a slot is free.",
                Spectral::Ectoplasm => {
                    "Adds a negative edition to a random joker and reduces hand size."
                }
                Spectral::Ankh => "Keeps a random joker, duplicates it and removes the others.",
                Spectral::Hex => {
                    "Gives a random joker a polychrome edition and removes the others."
                }
            };
            if s.requires_hand() {
                format!(
                    "{effect} Select {} to {} cards.",
                    s.min_targets(),
                    s.max_targets()
                )
            } else {
                effect.into()
            }
        }
    }
}

pub fn joker_description(id: &str) -> String {
    match id {
        "j_joker" => "Adds 4 Mult when a hand scores.",
        "j_greedy_joker" => "Each scored diamond adds 3 Mult.",
        "j_lusty_joker" => "Each scored heart adds 3 Mult.",
        "j_wrathful_joker" => "Each scored spade adds 3 Mult.",
        "j_gluttenous_joker" => "Each scored club adds 3 Mult.",
        "j_jolly" => "A hand containing a pair gains 8 Mult.",
        "j_zany" => "A hand containing three of a kind gains 12 Mult.",
        "j_mad" => "A hand containing two pairs gains 10 Mult.",
        "j_crazy" => "A hand containing a straight gains 12 Mult.",
        "j_droll" => "A hand containing a flush gains 10 Mult.",
        "j_sly" => "A hand containing a pair gains 50 chips.",
        "j_wily" => "A hand containing three of a kind gains 100 chips.",
        "j_clever" => "A hand containing two pairs gains 80 chips.",
        "j_devious" => "A hand containing a straight gains 100 chips.",
        "j_crafty" => "A hand containing a flush gains 80 chips.",
        "j_half" => "A played hand of at most three cards gains 20 Mult.",
        "j_banner" => "Each unused discard contributes 30 chips.",
        "j_mystic_summit" => "Adds 15 Mult when no discards remain.",
        "j_supernova" => "Adds Mult based on the number of times this hand was played.",
        "j_ride_the_bus" => "Gains Mult for hands without scoring face cards.",
        "j_fibonacci" => "Scored aces, twos, threes, fives and eights each add 8 Mult.",
        "j_scary_face" => "Each scored face card adds 30 chips.",
        "j_abstract" => "Each owned joker adds 3 Mult.",
        "j_pareidolia" => "Treats every playing card as a face card.",
        "j_even_steven" => "Scored even ranks each add 4 Mult.",
        "j_odd_todd" => "Scored odd ranks each add 31 chips.",
        "j_scholar" => "Each scored ace adds 20 chips and 4 Mult.",
        "j_blue_joker" => "Each card remaining in the deck adds 2 chips.",
        "j_baron" => "Each king held in hand multiplies Mult by 1.5.",
        "j_mime" => "Repeats abilities of cards held in hand.",
        "j_hack" => "Repeats scored twos, threes, fours and fives.",
        "j_dusk" => "Repeats played cards on the last hand of the round.",
        "j_hanging_chad" => "Repeats the first scoring card.",
        "j_raised_fist" => "Adds twice the lowest held rank to Mult.",
        "j_card_sharp" => "A hand type already played this round multiplies Mult by 3.",
        "j_stencil" => {
            "Mult is multiplied by the number of empty joker slots, counting this joker."
        }
        "j_steel_joker" => "Mult gains a 0.2 multiplier for each steel card in the deck.",
        "j_business" => "Each scored face card has a one-in-two chance to pay $2.",
        "j_faceless" => "Discard at least three face cards together to earn $5.",
        "j_green_joker" => "Gains 1 Mult per hand and loses 1 per discard.",
        "j_obelisk" => "Gains a 0.2 Mult multiplier when avoiding your most played hand type.",
        "j_midas_mask" => "Scored face cards become gold cards.",
        "j_luchador" => "Selling this joker disables the current boss.",
        "j_photograph" => "The first scored face card doubles Mult.",
        "j_erosion" => "Each card missing from the starting deck size adds 4 Mult.",
        "j_reserved_parking" => "Each held face card has a one-in-two chance to pay $1.",
        "j_mail" => "Each discarded card of the current rank pays $5; rank changes each round.",
        "j_stone" => "Each stone card in the deck contributes 25 chips.",
        "j_baseball" => "Each uncommon joker multiplies Mult by 1.5.",
        "j_bull" => "Each dollar held adds 2 chips.",
        "j_trading" => "A single-card first discard destroys the card and earns $3.",
        "j_trousers" => "Gains 2 Mult when a hand contains two pairs.",
        "j_walkie_talkie" => "Each scored four or ten adds 10 chips and 4 Mult.",
        "j_castle" => {
            "Gains 3 chips per discarded card of the current suit; suit changes each round."
        }
        "j_smiley" => "Each scored face card adds 5 Mult.",
        "j_ticket" => "Each scored gold card pays $4.",
        "j_acrobat" => "The last hand of the round multiplies Mult by 3.",
        "j_sock_and_buskin" => "Repeats each scored face card.",
        "j_rough_gem" => "Each scored diamond pays $1.",
        "j_bloodstone" => "Scored hearts have a one-in-two chance to multiply Mult by 1.5.",
        "j_arrowhead" => "Each scored spade adds 50 chips.",
        "j_onyx_agate" => "Each scored club adds 7 Mult.",
        "j_wee" => "Permanently gains 8 chips for every scored two.",
        "j_matador" => "Earns $8 when a played hand triggers the boss ability.",
        "j_hit_the_road" => "Gains a 0.5 Mult multiplier per jack discarded this round.",
        "j_duo" => "A hand containing a pair multiplies Mult by 2.",
        "j_trio" => "A hand containing three of a kind multiplies Mult by 3.",
        "j_family" => "A hand containing four of a kind multiplies Mult by 4.",
        "j_order" => "A hand containing a straight multiplies Mult by 3.",
        "j_tribe" => "A hand containing a flush multiplies Mult by 2.",
        "j_shoot_the_moon" => "Each queen held in hand adds 13 Mult.",
        "j_drivers_license" => "At least sixteen enhanced cards in the deck multiply Mult by 3.",
        "j_yorick" => "Gains a Mult multiplier after every 23 cards discarded.",
        "j_chicot" => "Disables boss blind abilities.",
        "j_misprint" => "Adds a random amount of Mult when scoring.",
        "j_golden" => "Earns money at the end of a round.",
        "j_ice_cream" => "Adds chips when scoring; decreases after each hand.",
        _ => "Joker effect: see supported rules in the engine.",
    }
    .into()
}

/// Value each scaling joker holds now, read from its engine state like the original `loc_vars`.
fn joker_current(joker: &balatro_rs::joker::Jokers, game: &Game) -> Option<CurrentValue> {
    let counter = joker.state().counter.max(0.0) as usize;
    let full_deck = || game.full_deck();
    let enhanced = |kind: Enhancement| {
        full_deck()
            .iter()
            .filter(|card| card.enhancement == Some(kind))
            .count()
    };
    let discarded = || game.non_debuffed(game.discarded_this_round.iter());
    let hundredths = |factor: f64| (factor * 100.0).round() as usize;
    Some(match joker.id() {
        "j_green_joker" => CurrentValue::Mult(counter),
        "j_ride_the_bus" => CurrentValue::Mult(game.consecutive_hands_without_face_card),
        "j_trousers" => CurrentValue::Mult(counter),
        "j_abstract" => CurrentValue::Mult(game.jokers.len() * 3),
        "j_erosion" => CurrentValue::Mult(52usize.saturating_sub(full_deck().len()) * 4),
        "j_blue_joker" => CurrentValue::Chips(game.deck.cards().len() * 2),
        "j_bull" => CurrentValue::Chips(game.money * 2),
        "j_wee" => CurrentValue::Chips(counter),
        "j_stone" => CurrentValue::Chips(enhanced(Enhancement::Stone) * 25),
        "j_castle" => CurrentValue::Chips(counter),
        "j_ice_cream" => CurrentValue::Chips(100usize.saturating_sub(5 * counter)),
        "j_obelisk" => CurrentValue::XMult(hundredths(1.0 + joker.state().counter as f64)),
        "j_hit_the_road" => {
            let jacks = discarded()
                .iter()
                .filter(|card| card.value == balatro_rs::card::Value::Jack)
                .count();
            CurrentValue::XMult(hundredths(1.0 + 0.5 * jacks as f64))
        }
        "j_steel_joker" => {
            CurrentValue::XMult(hundredths(1.0 + 0.2 * enhanced(Enhancement::Steel) as f64))
        }
        "j_yorick" => CurrentValue::XMult(100 * (1 + counter / 23)),
        "j_stencil" => {
            // Empty slots plus every Stencil held, like the original `x_mult`.
            let empty = game.joker_slots().saturating_sub(game.jokers.len());
            let stencils = game
                .jokers
                .iter()
                .filter(|joker| joker.id() == "j_stencil")
                .count();
            CurrentValue::XMult(100 * (empty + stencils))
        }
        "j_drivers_license" => CurrentValue::Count(
            full_deck()
                .iter()
                .filter(|card| card.enhancement.is_some())
                .count(),
        ),
        _ => return None,
    })
}

fn sticker_summary(stickers: balatro_rs::joker::Stickers) -> String {
    let mut active = Vec::new();
    if stickers.eternal {
        active.push("Eternal");
    }
    if stickers.perishable {
        active.push("Perishable");
    }
    if stickers.rental {
        active.push("Rental");
    }
    active.join(" · ")
}

fn visible_score(game: &Game, phase: Phase) -> usize {
    if matches!(phase, Phase::Reward | Phase::Shop | Phase::Won) {
        game.last_score
    } else {
        game.score
    }
}

#[cfg(test)]
mod scoring_tests {
    use super::*;
    use balatro_rs::joker::Jokers;

    #[test]
    fn trace_steps_keep_the_xmult_factor_and_money() {
        let native = NativeTrace(vec![balatro_rs::score::ScoreStep {
            source: NativeSource::HandLevel(balatro_rs::rank::HandRank::OnePair),
            chips_before: 10,
            chips_after: 10,
            mult_before: 4.0,
            mult_after: 6.0,
            retrigger: false,
            xmult: Some(1.5),
            money: 3,
        }]);
        let step = &map_trace(native).0[0];
        assert_eq!(step.xmult, Some(150));
        assert_eq!(step.money, 3);
    }

    fn start() -> MvpRun {
        let mut run = MvpRun::new(0);
        run.apply(MvpAction::StartBlind).unwrap();
        run
    }

    fn select_first(run: &mut MvpRun) {
        let id = run.view().cards[0].id;
        run.apply(MvpAction::ToggleCard(id)).unwrap();
    }

    #[test]
    fn traced_play_matches_plain_state_and_rng() {
        let mut plain = start();
        let mut first = plain.game.available.cards()[0];
        first.enhancement = Some(balatro_rs::card::Enhancement::Lucky);
        let mut snapshot: serde_json::Value =
            serde_json::from_str(&plain.snapshot().unwrap()).unwrap();
        snapshot["available"]["cards"][0][0] = serde_json::to_value(first).unwrap();
        plain.game = Game::from_json(&snapshot.to_string()).unwrap();
        assert_eq!(plain.game.available.cards()[0], first);
        plain
            .game
            .jokers
            .push(Jokers::HangingChad(Default::default()));
        #[cfg(feature = "extra-jokers")]
        plain.game.jokers.push(Jokers::Misprint(Default::default()));
        let before = plain.game.clone();
        plain
            .game
            .effect_registry
            .register_jokers(before.jokers.clone(), &before);
        select_first(&mut plain);
        let mut traced = MvpRun {
            game: plain.game.clone(),
            last_hand_score: None,
            revision: plain.revision,
            skipped_blinds: plain.skipped_blinds.clone(),
            stats: plain.stats,
            sort_by_suit: plain.sort_by_suit,
            debug: plain.debug,
        };
        plain.apply(MvpAction::Play).unwrap();
        let outcome = traced.apply_traced(MvpAction::Play, true).unwrap();
        let trace = outcome.trace.unwrap();
        assert!(trace.0.iter().any(|step| step.retrigger));
        assert_eq!(plain.view(), traced.view());
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&plain.snapshot().unwrap()).unwrap(),
            serde_json::from_str::<serde_json::Value>(&traced.snapshot().unwrap()).unwrap(),
        );
        for _ in 0..64 {
            assert_eq!(plain.game.prob_roll(1, 2), traced.game.prob_roll(1, 2));
        }
    }

    #[cfg(feature = "extra-jokers")]
    #[test]
    fn golden_joker_rewards_the_cleared_blind_once() {
        let mut run = start();
        run.game
            .jokers
            .push(Jokers::GoldenJoker(Default::default()));
        let game = run.game.clone();
        run.game
            .effect_registry
            .register_jokers(game.jokers.clone(), &game);
        select_first(&mut run);
        let money = run.view().money;
        run.game.score = run.game.required_score() - 1;
        run.apply_traced(MvpAction::Play, false).unwrap();
        assert_eq!(run.view().phase, Phase::Reward);
        assert_eq!(run.view().money, money + 4);
        let reward = run.view().reward;
        run.apply(MvpAction::CashOut).unwrap();
        assert_eq!(run.view().money, money + 4 + reward);
    }

    #[cfg(feature = "extra-jokers")]
    #[test]
    fn ice_cream_decreases_after_the_score_then_expires() {
        let mut run = start();
        let mut joker = Jokers::IceCream(Default::default());
        joker.state_mut().counter = 18.0;
        run.game.jokers.push(joker);
        for expected in [10, 5] {
            select_first(&mut run);
            let trace = run
                .apply_traced(MvpAction::Play, false)
                .unwrap()
                .trace
                .unwrap();
            assert!(trace.0.iter().any(|step| {
                matches!(
                    step.source,
                    ScoreSource::Joker {
                        kind: ContentId("j_ice_cream"),
                        ..
                    }
                ) && step.chips_after - step.chips_before == expected
            }));
        }
        assert!(run.game.jokers.is_empty());
    }

    #[cfg(feature = "extra-jokers")]
    #[test]
    fn misprint_rolls_vary_and_stay_in_range() {
        use balatro_rs::{
            card::{Card, Suit, Value},
            hand::SelectHand,
        };
        let mut game = start().game;
        game.jokers.push(Jokers::Misprint(Default::default()));
        let hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Spade)])
            .best_hand()
            .unwrap();
        let results: std::collections::BTreeSet<_> = (0..64)
            .map(|_| {
                game.calc_score_traced(hand.clone())
                    .1
                    .0
                    .iter()
                    .find(|step| matches!(step.source, NativeSource::Joker(Jokers::Misprint(_))))
                    .map_or(0, |step| (step.mult_after - step.mult_before) as usize)
            })
            .collect();
        assert!(results.len() > 1);
        assert!(results.iter().all(|&mult| mult <= 23));
    }
}

#[cfg(test)]
mod inventory_tests {
    use super::*;
    use balatro_rs::{
        consumable::Consumable, joker::Jokers, planet::Planets, spectral::Spectral, stage::Stage,
        tarot::Tarot,
    };

    fn shop() -> MvpRun {
        let mut run = MvpRun::new(7);
        run.game.stage = Stage::Shop();
        run.game.money = 30;
        run.game.shop.consumables = vec![Consumable::Planet(Planets::Mercury)];
        run.game.shop.jokers = vec![Jokers::TheJoker(Default::default())];
        run
    }

    #[test]
    fn every_supported_joker_has_an_effect_description() {
        use balatro_rs::joker::{JokerEffects, joker_catalog};
        for joker in joker_catalog().filter(|joker| joker.is_implemented()) {
            assert_ne!(
                joker_description(joker.id()),
                "Joker effect: see supported rules in the engine.",
                "{}",
                joker.id()
            );
        }
    }

    #[test]
    fn consumable_use_availability_follows_targets_and_stage() {
        let mut run = MvpRun::new(7);
        run.game.stage = Stage::Shop();
        run.game.consumables = vec![
            Consumable::Planet(Planets::Mercury),
            Consumable::Tarot(Tarot::Hermit),
            Consumable::Tarot(Tarot::Magician),
            Consumable::Spectral(Spectral::BlackHole),
            Consumable::Spectral(Spectral::Familiar),
        ];
        let view = run.view();
        let can_use: Vec<_> = view
            .consumables
            .iter()
            .map(|card| (card.kind, card.can_use))
            .collect();
        assert_eq!(
            can_use,
            vec![
                ("c_mercury", true),
                ("c_hermit", true),
                ("c_magician", false),
                ("c_black_hole", true),
                ("c_familiar", false),
            ],
            "non-target consumables stay usable outside a blind; hand-touching ones wait for a target"
        );
    }

    #[test]
    fn purchases_use_sell_and_reroll_follow_engine() {
        let mut run = shop();
        let initial = run.view();
        assert_eq!((initial.joker_slots, initial.consumable_slots), (5, 2));
        assert!(initial.consumable_offers[0].can_buy);
        run.apply(MvpAction::BuyConsumable {
            index: 0,
            revision: initial.revision,
        })
        .unwrap();
        let bought = run.view();
        assert_eq!(
            bought.money,
            initial.money - initial.consumable_offers[0].price
        );
        assert_eq!(bought.consumables[0].kind, "c_mercury");
        assert!(
            run.apply(MvpAction::UseConsumable {
                index: 0,
                revision: initial.revision
            })
            .is_err()
        );
        run.apply(MvpAction::UseConsumable {
            index: 0,
            revision: bought.revision,
        })
        .unwrap();
        assert!(run.view().consumables.is_empty());
        let joker = &run.view().offers[0];
        run.apply(MvpAction::BuyJoker(joker.id)).unwrap();
        let with_joker = run.view();
        assert_eq!(with_joker.held_jokers.len(), 1);
        run.apply(MvpAction::SellJoker {
            index: 0,
            revision: with_joker.revision,
        })
        .unwrap();
        assert!(run.view().held_jokers.is_empty());
        let before = run.view();
        assert!(before.can_reroll);
        run.apply(MvpAction::Reroll).unwrap();
        let after = run.view();
        assert_eq!(after.money, before.money - before.reroll_cost);
        assert_eq!(after.reroll_cost, before.reroll_cost + 1);
    }

    #[test]
    fn view_uses_current_inventory_capacities() {
        let mut run = shop();
        run.game.config.joker_slots = 3;
        run.game.config.consumable_slots = 4;
        let view = run.view();
        assert_eq!((view.joker_slots, view.consumable_slots), (3, 4));
    }

    #[test]
    fn tarot_requires_selected_card_and_view_reports_enhancement() {
        use balatro_rs::{card::Enhancement, tarot::Tarot};
        let mut run = MvpRun::new(0);
        run.apply(MvpAction::StartBlind).unwrap();
        run.game
            .consumables
            .push(Consumable::Tarot(Tarot::Magician));
        let before = run.view();
        assert!(!before.consumables[0].can_use);
        run.apply(MvpAction::ToggleCard(before.cards[0].id))
            .unwrap();
        let selected = run.view();
        assert!(selected.consumables[0].can_use);
        run.apply(MvpAction::UseConsumable {
            index: 0,
            revision: selected.revision,
        })
        .unwrap();
        assert!(
            run.game
                .available
                .cards()
                .iter()
                .any(|card| card.enhancement == Some(Enhancement::Lucky))
        );
        assert!(
            run.view()
                .cards
                .iter()
                .any(|card| card.enhancement == "Lucky")
        );
    }

    #[test]
    fn full_slots_insufficient_money_and_stale_indexes_are_refused() {
        let mut run = shop();
        run.game.money = 0;
        assert!(!run.view().consumable_offers[0].can_buy);
        assert!(
            run.apply(MvpAction::BuyConsumable {
                index: 0,
                revision: run.view().revision
            })
            .is_err()
        );
        assert!(run.apply(MvpAction::Reroll).is_err());
        run.game.money = 30;
        run.game.consumables = vec![
            Consumable::Planet(Planets::Earth),
            Consumable::Planet(Planets::Mars),
        ];
        let full = run.view();
        assert!(!full.consumable_offers[0].can_buy);
        assert!(
            run.apply(MvpAction::BuyConsumable {
                index: 0,
                revision: full.revision
            })
            .is_err()
        );
        run.apply(MvpAction::SellConsumable {
            index: 0,
            revision: full.revision,
        })
        .unwrap();
        let after = run.view();
        assert_eq!(after.consumables[0].kind, "c_mars");
        assert!(
            run.apply(MvpAction::SellConsumable {
                index: 0,
                revision: full.revision
            })
            .is_err()
        );
        assert_eq!(run.view(), after);
    }
}

#[cfg(test)]
mod campaign_tests {
    use super::*;
    use balatro_rs::ante::Ante;

    fn clear(run: &mut MvpRun) -> ActionOutcome {
        run.apply(MvpAction::StartBlind).unwrap();
        for id in run.view().cards.iter().take(5).map(|card| card.id) {
            run.apply(MvpAction::ToggleCard(id)).unwrap();
        }
        // Drive the real score and round-end actions without requiring a winning
        // strategy for every boss generated by this fixture.
        run.game.score = run.game.required_score();
        run.apply_traced(MvpAction::Play, false).unwrap()
    }

    fn cash_out_and_leave(run: &mut MvpRun) {
        assert_eq!(run.phase(), Phase::Reward);
        let money = run.game.money;
        let reward = run.game.reward;
        run.apply(MvpAction::CashOut).unwrap();
        assert_eq!(run.game.money, money + reward);
        assert!(run.apply(MvpAction::CashOut).is_err());
        run.apply(MvpAction::LeaveShop).unwrap();
        assert_eq!(run.phase(), Phase::BeforeBlind);
    }

    #[test]
    fn hand_preview_uses_the_evaluator_and_current_level_without_mutation() {
        use balatro_rs::card::{Card, Value};
        let mut run = MvpRun::new(0);
        run.apply(MvpAction::StartBlind).unwrap();
        assert_eq!(run.view().selected_hand, None);

        for (values, rank, chips, mult) in [
            (
                [
                    Value::Ace,
                    Value::King,
                    Value::Queen,
                    Value::Ten,
                    Value::Two,
                ],
                HandRank::HighCard,
                5,
                1,
            ),
            (
                [
                    Value::Five,
                    Value::Five,
                    Value::Six,
                    Value::Six,
                    Value::Nine,
                ],
                HandRank::TwoPair,
                20,
                2,
            ),
            (
                [
                    Value::Ace,
                    Value::Two,
                    Value::Three,
                    Value::Four,
                    Value::Five,
                ],
                HandRank::Straight,
                30,
                4,
            ),
        ] {
            let mut snapshot: serde_json::Value =
                serde_json::from_str(&run.snapshot().unwrap()).unwrap();
            let cards = values
                .into_iter()
                .zip([
                    Suit::Heart,
                    Suit::Spade,
                    Suit::Diamond,
                    Suit::Club,
                    Suit::Heart,
                ])
                .map(|(value, suit)| (Card::new(value, suit), true))
                .collect::<Vec<_>>();
            snapshot["available"]["cards"] = serde_json::to_value(cards).unwrap();
            run.game = Game::from_json(&snapshot.to_string()).unwrap();
            let snapshot = run.snapshot().unwrap();
            let hand = run.view().selected_hand.unwrap();
            assert_eq!(
                (hand.rank, hand.level.chips, hand.level.mult),
                (map_rank(rank), chips, mult)
            );
            assert_eq!(run.snapshot().unwrap(), snapshot);
            run.game.planetarium.level_up(rank);
            assert_eq!(run.view().selected_hand.unwrap().level.level, 2);
        }
    }

    #[test]
    fn voucher_checkout_checks_revision_funds_and_discovery() {
        let mut run = MvpRun::new(0);
        run.game.stage = Stage::Shop();
        run.game.shop.voucher = Some(Voucher::Grabber);
        run.game.money = 0;
        let action = MvpAction::RedeemVoucher {
            voucher: ContentId(Voucher::Grabber.id()),
            revision: run.view().revision,
        };
        assert!(run.apply(action).is_err());
        run.game.money = 30;
        let outcome = run.apply_traced(action, false).unwrap();
        assert_eq!(outcome.discovered_item, Some(Voucher::Grabber.id()));
        assert_eq!(run.view().vouchers, [voucher_content(Voucher::Grabber)]);
        assert_eq!(run.view().money, 20);
        assert!(run.apply(action).is_err());
        run.apply(MvpAction::Reroll).unwrap();
        assert!(run.view().voucher_offers.is_empty());
    }

    #[test]
    fn both_sorts_preserve_the_selected_card() {
        let mut run = MvpRun::new(0);
        run.apply(MvpAction::StartBlind).unwrap();
        let id = run.view().cards[0].id;
        run.apply(MvpAction::ToggleCard(id)).unwrap();

        for action in [MvpAction::SortSuit, MvpAction::SortRank] {
            run.apply(action).unwrap();
            let view = run.view();
            assert_eq!(
                view.cards
                    .iter()
                    .filter(|card| card.selected)
                    .map(|card| card.id)
                    .collect::<Vec<_>>(),
                [id]
            );
        }
    }

    #[test]
    fn campaign_progresses_to_eight_then_resumes_same_game() {
        let mut run = MvpRun::new(0);
        run.game.small_blind_tag = balatro_rs::tag::Tag::Investment;
        for ante in 1..=8 {
            let view = run.view();
            assert_eq!(view.ante, ante);
            assert!(view.blinds.iter().all(|choice| !choice.skipped));
            for blind in [Blind::Small, Blind::Big, Blind::Boss] {
                if ante == 1 && blind == Blind::Small {
                    let revision = run.view().revision;
                    run.apply(MvpAction::SkipBlind { revision }).unwrap();
                    assert_eq!(run.view().tags.len(), 1);
                    continue;
                }
                let outcome = clear(&mut run);
                if ante == 8 && blind == Blind::Boss {
                    assert_eq!(outcome.phase, Phase::Won);
                    assert!(outcome.campaign_won);
                    assert!(run.view().can_continue_endless);
                } else {
                    assert!(!outcome.campaign_won);
                    cash_out_and_leave(&mut run);
                }
            }
        }

        let before = run.view();
        let reward = run.game.reward;
        let mut resumed = MvpRun {
            game: Game::from_json(&run.snapshot().unwrap()).unwrap(),
            ..MvpRun::new(0)
        };
        run.apply(MvpAction::ContinueEndless).unwrap();
        resumed.apply(MvpAction::ContinueEndless).unwrap();
        assert_eq!(run.snapshot().unwrap(), resumed.snapshot().unwrap());
        let after = run.view();
        assert_eq!(after.ante, 9);
        assert_eq!(after.money, before.money);
        assert_eq!(after.deck_view, before.deck_view);
        assert_eq!(after.reward, reward);
        assert!(after.campaign_won && after.endless_mode);
        assert!(!after.can_continue_endless);
        assert!(run.apply(MvpAction::ContinueEndless).is_err());
        cash_out_and_leave(&mut run);
        assert_eq!(run.view().target, 110_000);
        run.apply(MvpAction::StartBlind).unwrap();
        run.game.plays = 1;
        let id = run.view().cards[0].id;
        run.apply(MvpAction::ToggleCard(id)).unwrap();
        assert_eq!(
            run.apply_traced(MvpAction::Play, false).unwrap().phase,
            Phase::Lost
        );
        assert!(run.view().campaign_won);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn endless_finishes_at_last_representable_ante() {
        let mut run = MvpRun::new(1);
        run.game.small_blind_tag = balatro_rs::tag::Tag::Investment;
        run.game.big_blind_tag = balatro_rs::tag::Tag::Double;
        run.game.ante_current = Ante::try_from(15).unwrap();
        run.game.campaign_won = true;
        run.game.endless_mode = true;
        assert_eq!(run.view().target, 77_000_000_000_000_000);
        assert!(Ante::try_from(16).is_err());
        for _ in 0..2 {
            let revision = run.view().revision;
            run.apply(MvpAction::SkipBlind { revision }).unwrap();
        }
        let outcome = clear(&mut run);
        assert_eq!(outcome.phase, Phase::Won);
        assert!(outcome.campaign_won);
        assert_eq!(run.view().ante, 15);
        assert!(!run.view().can_continue_endless);
    }

    #[test]
    fn legacy_snapshots_default_to_campaign_flags_off() {
        let mut old: serde_json::Value =
            serde_json::from_str(&MvpRun::new(0).snapshot().unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("campaign_won");
        old.as_object_mut().unwrap().remove("endless_mode");
        assert_eq!(old["ante_current"], "One");
        let game = Game::from_json(&old.to_string()).unwrap();
        assert!(!game.campaign_won && !game.endless_mode);
    }

    #[test]
    fn use_matches_original_rules_in_blind_without_selection() {
        use balatro_rs::{consumable::Consumable, spectral::Spectral, tarot::Tarot};
        let mut run = MvpRun::new(0);
        run.apply(MvpAction::StartBlind).unwrap();
        run.game.consumables = vec![
            Consumable::Spectral(Spectral::Familiar), // origine : main > 1 carte suffit
            Consumable::Tarot(Tarot::Hermit),         // origine : toujours utilisable
            Consumable::Tarot(Tarot::Magician),       // origine : cible requise
        ];
        let view = run.view();
        let can_use: Vec<_> = view
            .consumables
            .iter()
            .map(|c| (c.kind, c.can_use))
            .collect();
        assert_eq!(
            can_use,
            vec![
                ("c_familiar", true),
                ("c_hermit", true),
                ("c_magician", false)
            ],
        );
    }

    #[test]
    fn scaling_jokers_report_the_value_their_effect_will_use() {
        use balatro_rs::joker::Jokers;
        let mut run = MvpRun::new(0);
        run.game.jokers = vec![
            Jokers::GreenJoker(Default::default()),
            Jokers::AbstractJoker(Default::default()),
            Jokers::Bull(Default::default()),
            Jokers::BlueJoker(Default::default()),
            Jokers::JokerStencil(Default::default()),
            Jokers::Obelisk(Default::default()),
            Jokers::TheJoker(Default::default()),
        ];
        let current = |run: &MvpRun| {
            run.view()
                .held_jokers
                .iter()
                .map(|joker| joker.current)
                .collect::<Vec<_>>()
        };
        let deck = run.game.deck.cards().len();
        assert_eq!(
            current(&run),
            vec![
                Some(CurrentValue::Mult(0)),
                Some(CurrentValue::Mult(21)),
                Some(CurrentValue::Chips(8)),
                Some(CurrentValue::Chips(deck * 2)),
                Some(CurrentValue::XMult(100)),
                Some(CurrentValue::XMult(100)),
                None,
            ]
        );

        run.game.jokers.truncate(1);
        let before = run.game.clone();
        run.game
            .effect_registry
            .register_jokers(before.jokers.clone(), &before);
        run.apply(MvpAction::StartBlind).unwrap();
        let first = run.view().cards[0].id;
        run.apply(MvpAction::ToggleCard(first)).unwrap();
        run.apply(MvpAction::Play).unwrap();
        assert_eq!(run.view().phase, Phase::Playing);
        assert_eq!(current(&run)[0], Some(CurrentValue::Mult(1)));
        let next = run.view().cards[0].id;
        run.apply(MvpAction::ToggleCard(next)).unwrap();
        run.apply(MvpAction::Discard).unwrap();
        assert_eq!(current(&run)[0], Some(CurrentValue::Mult(0)));
    }

    #[test]
    fn starting_money_override_only_changes_the_first_balance() {
        let config = |starting_money| RunConfig {
            seed: 0,
            deck: Deck::Red,
            goal: CampaignGoal::Standard,
            starting_money,
        };
        assert_eq!(MvpRun::with_config(config(None)).view().money, 4);
        let rich = MvpRun::with_config(config(Some(1000)));
        assert_eq!(rich.view().money, 1000);
        assert!(!rich.view().debug);
    }

    #[test]
    fn debug_commands_change_the_run_and_mark_it() {
        let mut run = MvpRun::new(0);
        assert!(run.apply(MvpAction::Debug(DebugCommand::AddHand)).is_err());
        run.apply(MvpAction::Debug(DebugCommand::AddMoney(10)))
            .unwrap();
        assert_eq!(run.view().money, 14);
        assert!(run.view().debug);

        run.apply(MvpAction::Debug(DebugCommand::AddAnte)).unwrap();
        assert_eq!(run.view().ante, 2);

        run.apply(MvpAction::StartBlind).unwrap();
        let before = run.view();
        run.apply(MvpAction::Debug(DebugCommand::AddHand)).unwrap();
        run.apply(MvpAction::Debug(DebugCommand::AddDiscard))
            .unwrap();
        run.apply(MvpAction::Debug(DebugCommand::AddScore(10)))
            .unwrap();
        let after = run.view();
        assert_eq!(after.hands, before.hands + 1);
        assert_eq!(after.discards, before.discards + 1);
        assert_eq!(after.score, before.score + 10);

        run.apply(MvpAction::Debug(DebugCommand::WinBlind)).unwrap();
        assert_eq!(run.view().phase, Phase::Reward);
        run.apply(MvpAction::CashOut).unwrap();
        assert_eq!(run.view().phase, Phase::Shop);

        run.apply(MvpAction::Debug(DebugCommand::LoseRun)).unwrap();
        assert_eq!(run.view().phase, Phase::Lost);
        assert!(run.apply(MvpAction::Debug(DebugCommand::LoseRun)).is_err());
    }

    #[test]
    fn the_fool_requires_last_tarot_or_planet_not_fool() {
        use balatro_rs::{consumable::Consumable, planet::Planets, tarot::Tarot};
        let mut run = MvpRun::new(0);
        run.apply(MvpAction::StartBlind).unwrap();
        run.game.consumables = vec![Consumable::Tarot(Tarot::Fool)];
        run.game.last_consumable_used = Some(Consumable::Tarot(Tarot::Fool));
        assert!(!run.view().consumables[0].can_use);
        run.game.last_consumable_used = Some(Consumable::Planet(Planets::Mercury));
        assert!(run.view().consumables[0].can_use);
    }
}
