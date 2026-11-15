use serde::{Deserialize, Serialize};

use crate::contract::{Command, DebugCommand, MoveDirection, RunView};

/// A command as the log stores it: cards and Jokers by their position in the confirmed
/// view, vouchers by content id, and no view revision (the replay uses its own).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum LoggedCommand {
    StartBlind,
    ContinueEndless,
    SortRank,
    SortSuit,
    MoveCard {
        hand_index: usize,
        right: bool,
    },
    ReorderJoker {
        from: usize,
        to: usize,
    },
    ReorderConsumable {
        from: usize,
        to: usize,
    },
    SkipBlind,
    ToggleCard {
        hand_index: usize,
    },
    Play,
    Discard,
    CashOut,
    BuyJoker {
        offer_index: usize,
    },
    RedeemVoucher {
        voucher: String,
    },
    BuyPlayingCard {
        offer_index: usize,
    },
    BuyPack {
        index: usize,
    },
    PickPack {
        index: usize,
    },
    SkipPack,
    BuyConsumable {
        index: usize,
    },
    BuyAndUseConsumable {
        index: usize,
    },
    UseConsumable {
        index: usize,
    },
    SellConsumable {
        index: usize,
    },
    SellJoker {
        index: usize,
    },
    Reroll,
    RerollBoss,
    LeaveShop,
    Debug {
        debug: String,
        amount: Option<usize>,
    },
    /// A command whose card or offer was not in the confirmed view.
    Unresolved {
        debug: String,
    },
}

impl LoggedCommand {
    pub fn from_command(command: Command, view: &RunView) -> Self {
        let hand = |id| view.cards.iter().position(|card| card.id == id);
        let unresolved = || Self::Unresolved {
            debug: format!("{command:?}"),
        };
        match command {
            Command::StartBlind => Self::StartBlind,
            Command::ContinueEndless => Self::ContinueEndless,
            Command::SortRank => Self::SortRank,
            Command::SortSuit => Self::SortSuit,
            Command::MoveCard { id, direction } => match hand(id) {
                Some(hand_index) => Self::MoveCard {
                    hand_index,
                    right: direction == MoveDirection::Right,
                },
                None => unresolved(),
            },
            Command::ReorderJoker { from, to, .. } => Self::ReorderJoker { from, to },
            Command::ReorderConsumable { from, to, .. } => Self::ReorderConsumable { from, to },
            Command::SkipBlind { .. } => Self::SkipBlind,
            Command::ToggleCard(id) => match hand(id) {
                Some(hand_index) => Self::ToggleCard { hand_index },
                None => unresolved(),
            },
            Command::Play => Self::Play,
            Command::Discard => Self::Discard,
            Command::CashOut => Self::CashOut,
            Command::BuyJoker(id) => match view.offers.iter().position(|joker| joker.id == id) {
                Some(offer_index) => Self::BuyJoker { offer_index },
                None => unresolved(),
            },
            Command::RedeemVoucher { voucher, .. } => Self::RedeemVoucher {
                voucher: voucher.0.to_owned(),
            },
            Command::BuyPlayingCard { id, .. } => {
                match view.shop_cards.iter().position(|(card, _)| *card == id) {
                    Some(offer_index) => Self::BuyPlayingCard { offer_index },
                    None => unresolved(),
                }
            }
            Command::BuyPack { index, .. } => Self::BuyPack { index },
            Command::PickPack { index, .. } => Self::PickPack { index },
            Command::SkipPack => Self::SkipPack,
            Command::BuyConsumable { index, .. } => Self::BuyConsumable { index },
            Command::BuyAndUseConsumable { index, .. } => Self::BuyAndUseConsumable { index },
            Command::UseConsumable { index, .. } => Self::UseConsumable { index },
            Command::SellConsumable { index, .. } => Self::SellConsumable { index },
            Command::SellJoker { index, .. } => Self::SellJoker { index },
            Command::Reroll => Self::Reroll,
            Command::RerollBoss => Self::RerollBoss,
            Command::LeaveShop => Self::LeaveShop,
            Command::Debug(debug) => {
                let (name, amount) = match debug {
                    DebugCommand::AddMoney(amount) => ("add_money", Some(amount)),
                    DebugCommand::AddHand => ("add_hand", None),
                    DebugCommand::AddDiscard => ("add_discard", None),
                    DebugCommand::AddRound => ("add_round", None),
                    DebugCommand::AddAnte => ("add_ante", None),
                    DebugCommand::AddScore(amount) => ("add_score", Some(amount)),
                    DebugCommand::WinBlind => ("win_blind", None),
                    DebugCommand::LoseRun => ("lose_run", None),
                };
                Self::Debug {
                    debug: name.to_owned(),
                    amount,
                }
            }
        }
    }

    /// The command against `view`, or `None` when its card or offer is not there.
    pub fn to_command(&self, view: &RunView) -> Option<Command> {
        let revision = view.revision;
        let card = |index: usize| view.cards.get(index).map(|card| card.id);
        Some(match self {
            Self::StartBlind => Command::StartBlind,
            Self::ContinueEndless => Command::ContinueEndless,
            Self::SortRank => Command::SortRank,
            Self::SortSuit => Command::SortSuit,
            Self::MoveCard { hand_index, right } => Command::MoveCard {
                id: card(*hand_index)?,
                direction: if *right {
                    MoveDirection::Right
                } else {
                    MoveDirection::Left
                },
            },
            Self::ReorderJoker { from, to } => Command::ReorderJoker {
                from: *from,
                to: *to,
                revision,
            },
            Self::ReorderConsumable { from, to } => Command::ReorderConsumable {
                from: *from,
                to: *to,
                revision,
            },
            Self::SkipBlind => Command::SkipBlind { revision },
            Self::ToggleCard { hand_index } => Command::ToggleCard(card(*hand_index)?),
            Self::Play => Command::Play,
            Self::Discard => Command::Discard,
            Self::CashOut => Command::CashOut,
            Self::BuyJoker { offer_index } => Command::BuyJoker(view.offers.get(*offer_index)?.id),
            Self::RedeemVoucher { voucher } => Command::RedeemVoucher {
                voucher: view
                    .voucher_offers
                    .iter()
                    .find(|(content, _)| content.id.0 == voucher)?
                    .0
                    .id,
                revision,
            },
            Self::BuyPlayingCard { offer_index } => Command::BuyPlayingCard {
                id: view.shop_cards.get(*offer_index)?.0,
                revision,
            },
            Self::BuyPack { index } => Command::BuyPack {
                index: *index,
                revision,
            },
            Self::PickPack { index } => Command::PickPack {
                index: *index,
                revision,
            },
            Self::SkipPack => Command::SkipPack,
            Self::BuyConsumable { index } => Command::BuyConsumable {
                index: *index,
                revision,
            },
            Self::BuyAndUseConsumable { index } => Command::BuyAndUseConsumable {
                index: *index,
                revision,
            },
            Self::UseConsumable { index } => Command::UseConsumable {
                index: *index,
                revision,
            },
            Self::SellConsumable { index } => Command::SellConsumable {
                index: *index,
                revision,
            },
            Self::SellJoker { index } => Command::SellJoker {
                index: *index,
                revision,
            },
            Self::Reroll => Command::Reroll,
            Self::RerollBoss => Command::RerollBoss,
            Self::LeaveShop => Command::LeaveShop,
            Self::Debug { debug, amount } => Command::Debug(match (debug.as_str(), amount) {
                ("add_money", Some(amount)) => DebugCommand::AddMoney(*amount),
                ("add_hand", _) => DebugCommand::AddHand,
                ("add_discard", _) => DebugCommand::AddDiscard,
                ("add_round", _) => DebugCommand::AddRound,
                ("add_ante", _) => DebugCommand::AddAnte,
                ("add_score", Some(amount)) => DebugCommand::AddScore(*amount),
                ("win_blind", _) => DebugCommand::WinBlind,
                ("lose_run", _) => DebugCommand::LoseRun,
                _ => return None,
            }),
            Self::Unresolved { .. } => return None,
        })
    }
}
