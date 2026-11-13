use super::*;

#[derive(Debug)]
pub struct RunUpdate {
    pub events: Vec<RunEvent>,
    pub outcome: Outcome,
    pub view: RunView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEvent {
    PhaseChanged(Phase),
    HandScored(usize),
    CampaignWon,
    Discovered(ContentId),
}

impl RunUpdate {
    pub fn confirmed(
        before: Phase,
        won_before: bool,
        command: Command,
        outcome: Outcome,
        view: RunView,
    ) -> Self {
        let mut events = Vec::new();
        if before != view.phase {
            events.push(RunEvent::PhaseChanged(view.phase));
        }
        if command == Command::Play
            && let Some(score) = outcome.hand_score
        {
            events.push(RunEvent::HandScored(score));
        }
        if !won_before && view.campaign_won {
            events.push(RunEvent::CampaignWon);
        }
        if let Some(id) = outcome.discovered_item {
            events.push(RunEvent::Discovered(ContentId(id)));
        }
        Self {
            events,
            outcome,
            view,
        }
    }
}

/// Dev-only shortcuts mirroring the original debug panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugCommand {
    AddMoney(usize),
    AddHand,
    AddDiscard,
    AddRound,
    AddAnte,
    AddScore(usize),
    WinBlind,
    LoseRun,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    StartBlind,
    ContinueEndless,
    SortRank,
    SortSuit,
    MoveCard {
        id: CardKey,
        direction: MoveDirection,
    },
    ReorderJoker {
        from: usize,
        to: usize,
        revision: u64,
    },
    ReorderConsumable {
        from: usize,
        to: usize,
        revision: u64,
    },
    SkipBlind {
        revision: u64,
    },
    ToggleCard(CardKey),
    Play,
    Discard,
    CashOut,
    BuyJoker(JokerKey),
    RedeemVoucher {
        voucher: ContentId,
        revision: u64,
    },
    BuyPlayingCard {
        id: CardKey,
        revision: u64,
    },
    BuyPack {
        index: usize,
        revision: u64,
    },
    PickPack {
        index: usize,
        revision: u64,
    },
    ApplyTarget,
    SkipPack,
    BuyConsumable {
        index: usize,
        revision: u64,
    },
    /// Buys a shop consumable and uses it at once, without a consumable slot.
    BuyAndUseConsumable {
        index: usize,
        revision: u64,
    },
    UseConsumable {
        index: usize,
        revision: u64,
    },
    SellConsumable {
        index: usize,
        revision: u64,
    },
    SellJoker {
        index: usize,
        revision: u64,
    },
    Reroll,
    RerollBoss,
    LeaveShop,
    Debug(DebugCommand),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDirection {
    Left,
    Right,
}
