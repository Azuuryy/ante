use std::{fmt, time::Duration};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CardKey(pub usize);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct JokerKey(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentId(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub id: ContentId,
    pub name: String,
    pub description: String,
    pub cost: usize,
    pub supported: bool,
    /// Human-readable condition that unlocks this item in the collection.
    /// `None` means the item is available from the start.
    pub unlock: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentGroup {
    Jokers,
    Vouchers,
    Tags,
    Tarots,
    Planets,
    Spectrals,
    Boosters,
    Blinds,
    Enhancements,
    Editions,
    Seals,
}

impl ContentGroup {
    /// Every group the Collection lists, in hub order.
    pub const ALL: [Self; 11] = [
        Self::Jokers,
        Self::Vouchers,
        Self::Tarots,
        Self::Planets,
        Self::Spectrals,
        Self::Enhancements,
        Self::Seals,
        Self::Editions,
        Self::Boosters,
        Self::Tags,
        Self::Blinds,
    ];
}

/// Booster packs by collection id, with their art family and size.
pub const BOOSTERS: [(&str, &str, PackFamily, PackSize); 15] = [
    (
        "p_arcana_normal",
        "Arcana Pack",
        PackFamily::Arcana,
        PackSize::Normal,
    ),
    (
        "p_arcana_jumbo",
        "Jumbo Arcana Pack",
        PackFamily::Arcana,
        PackSize::Jumbo,
    ),
    (
        "p_arcana_mega",
        "Mega Arcana Pack",
        PackFamily::Arcana,
        PackSize::Mega,
    ),
    (
        "p_celestial_normal",
        "Celestial Pack",
        PackFamily::Celestial,
        PackSize::Normal,
    ),
    (
        "p_celestial_jumbo",
        "Jumbo Celestial Pack",
        PackFamily::Celestial,
        PackSize::Jumbo,
    ),
    (
        "p_celestial_mega",
        "Mega Celestial Pack",
        PackFamily::Celestial,
        PackSize::Mega,
    ),
    (
        "p_spectral_normal",
        "Spectral Pack",
        PackFamily::Spectral,
        PackSize::Normal,
    ),
    (
        "p_spectral_jumbo",
        "Jumbo Spectral Pack",
        PackFamily::Spectral,
        PackSize::Jumbo,
    ),
    (
        "p_spectral_mega",
        "Mega Spectral Pack",
        PackFamily::Spectral,
        PackSize::Mega,
    ),
    (
        "p_standard_normal",
        "Standard Pack",
        PackFamily::Standard,
        PackSize::Normal,
    ),
    (
        "p_standard_jumbo",
        "Jumbo Standard Pack",
        PackFamily::Standard,
        PackSize::Jumbo,
    ),
    (
        "p_standard_mega",
        "Mega Standard Pack",
        PackFamily::Standard,
        PackSize::Mega,
    ),
    (
        "p_buffoon_normal",
        "Buffoon Pack",
        PackFamily::Buffoon,
        PackSize::Normal,
    ),
    (
        "p_buffoon_jumbo",
        "Jumbo Buffoon Pack",
        PackFamily::Buffoon,
        PackSize::Jumbo,
    ),
    (
        "p_buffoon_mega",
        "Mega Buffoon Pack",
        PackFamily::Buffoon,
        PackSize::Mega,
    ),
];

/// Collection id of a booster pack.
pub fn booster_id(family: PackFamily, size: PackSize) -> &'static str {
    BOOSTERS
        .iter()
        .find(|(_, _, f, s)| *f == family && *s == size)
        .map_or("p_arcana_normal", |(id, ..)| id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopArt {
    Card { rank: char, suit: &'static str },
    Joker(&'static str),
    Consumable(&'static str),
    Pack { family: PackFamily, size: PackSize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackFamily {
    Arcana,
    Celestial,
    Spectral,
    Standard,
    Buffoon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackSize {
    Normal,
    Jumbo,
    Mega,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandKind {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandLevel {
    pub level: usize,
    pub chips: usize,
    pub mult: usize,
    pub plays: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardTrigger {
    Played,
    Held,
    StoneKicker,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreSource {
    HandLevel(HandKind),
    Card {
        trigger: CardTrigger,
        id: CardKey,
        rank: char,
        suit: &'static str,
    },
    Joker {
        id: JokerKey,
        kind: ContentId,
        name: &'static str,
    },
    Voucher(ContentId, &'static str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScoreStep {
    pub source: ScoreSource,
    pub chips_before: usize,
    pub chips_after: usize,
    /// Mult is fractional, like the original: X1.5 on 7 Mult gives 10.5.
    pub mult_before: f64,
    pub mult_after: f64,
    pub retrigger: bool,
    /// Factor in hundredths when the Mult change is multiplicative: 150 is X1.5 Mult.
    pub xmult: Option<usize>,
    /// Money earned during the step, such as a Gold Seal.
    pub money: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScoreTrace(pub Vec<ScoreStep>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceMode {
    Off,
    Capture,
    Profile,
}

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

pub trait RunDriver: Send + Sync {
    fn view(&self) -> RunView;
    fn apply(&mut self, command: Command, trace: TraceMode) -> Result<RunUpdate, RunError>;
}

pub trait GameBackend: Send + Sync + 'static {
    fn start(&self, config: RunConfig) -> Box<dyn RunDriver>;
    fn catalog(&self, group: ContentGroup) -> &'static [Content];
    fn parse_seed(&self, seed: &str) -> u64;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Deck {
    #[default]
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CampaignGoal {
    SingleAnte,
    #[default]
    Standard,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunConfig {
    pub seed: u64,
    pub deck: Deck,
    pub goal: CampaignGoal,
    /// Overrides the deck's starting money; the Dev build uses it for testing.
    pub starting_money: Option<usize>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    BeforeBlind,
    Playing,
    Reward,
    Shop,
    Pack,
    Target,
    Won,
    Lost,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CardView {
    pub id: CardKey,
    pub rank: char,
    pub suit: &'static str,
    pub red: bool,
    pub selected: bool,
    pub debuffed: bool,
    pub enabled: bool,
    pub chips: usize,
    pub enhancement: String,
    pub edition: String,
    pub seal: String,
    pub effect_summary: String,
}

/// Value a scaling joker would apply now, shown as the tooltip's "Currently" line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentValue {
    Mult(usize),
    Chips(usize),
    /// Multiplier in hundredths: 150 is X1.5.
    XMult(usize),
    Money(usize),
    Count(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct JokerView {
    pub id: JokerKey,
    pub kind: &'static str,
    pub name: &'static str,
    pub price: usize,
    pub enabled: bool,
    pub description: String,
    pub state: String,
    pub current: Option<CurrentValue>,
    pub rarity: String,
    pub edition: String,
    pub stickers: String,
    pub can_sell: bool,
    pub sell_price: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConsumableView {
    pub index: usize,
    pub kind: &'static str,
    pub name: String,
    pub category: String,
    pub description: String,
    pub price: usize,
    pub can_buy: bool,
    pub can_use: bool,
    pub can_sell: bool,
    pub sell_price: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BlindView {
    pub id: &'static str,
    pub name: String,
    pub target: usize,
    pub reward: usize,
    pub description: String,
    pub selectable: bool,
    pub passed: bool,
    pub skipped: bool,
    pub tag: Option<TagView>,
    pub can_skip: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TagView {
    pub content: &'static Content,
    pub name: String,
}

pub const SUITS: [&str; 4] = ["Spades", "Hearts", "Clubs", "Diamonds"];
pub const RANKS: &str = "AKQJT98765432";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeckCardView {
    pub rank: char,
    pub suit: usize,
    pub edition: &'static str,
    pub enhancement: Option<&'static str>,
    pub seal: Option<&'static str>,
    pub debuffed: bool,
    pub remaining: bool,
    pub effective_suits: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RunStats {
    pub best_hand: usize,
    pub played: usize,
    pub discarded: usize,
    pub purchased: usize,
    pub rerolls: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandView {
    pub rank: HandKind,
    pub level: HandLevel,
}

impl HandView {
    pub fn name(self) -> &'static str {
        match self.rank {
            HandKind::HighCard => "High Card",
            HandKind::OnePair => "Pair",
            HandKind::TwoPair => "Two Pair",
            HandKind::ThreeOfAKind => "Three of a Kind",
            HandKind::Straight => "Straight",
            HandKind::Flush => "Flush",
            HandKind::FullHouse => "Full House",
            HandKind::FourOfAKind => "Four of a Kind",
            HandKind::StraightFlush => "Straight Flush",
            HandKind::RoyalFlush => "Royal Flush",
            HandKind::FiveOfAKind => "Five of a Kind",
            HandKind::FlushHouse => "Flush House",
            HandKind::FlushFive => "Flush Five",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ShopItem {
    pub name: String,
    pub description: String,
    pub price: usize,
    pub enabled: bool,
    pub art: ShopArt,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RunView {
    pub seed: u64,
    pub revision: u64,
    pub round: usize,
    pub ante: usize,
    pub ante_end: usize,
    pub campaign_won: bool,
    pub endless_mode: bool,
    pub can_continue_endless: bool,
    pub stats: RunStats,
    pub deck_view: Vec<DeckCardView>,
    pub tags: Vec<TagView>,
    pub can_sort: bool,
    pub blind_chip: Option<&'static str>,
    pub blind_rule: String,
    pub blinds: Vec<BlindView>,
    pub selected_hand: Option<HandView>,
    pub hand_levels: Vec<HandView>,
    pub vouchers: Vec<&'static Content>,
    pub voucher_offers: Vec<(&'static Content, bool)>,
    pub can_reroll_boss: bool,
    pub can_skip_pack: bool,
    pub shop_cards: Vec<(CardKey, ShopItem)>,
    pub packs: Vec<ShopItem>,
    pub pack_choices: Vec<ShopItem>,
    pub pack_picks: usize,
    pub can_apply_target: bool,
    pub deck_total: usize,
    pub phase: Phase,
    pub blind: String,
    pub target: usize,
    pub score: usize,
    pub last_hand_score: Option<usize>,
    pub hands: usize,
    pub discards: usize,
    pub money: usize,
    pub reward: usize,
    pub deck_cards: usize,
    pub discarded_cards: usize,
    pub cards: Vec<CardView>,
    pub offers: Vec<JokerView>,
    pub jokers: Vec<&'static str>,
    pub held_jokers: Vec<JokerView>,
    pub joker_slots: usize,
    pub consumables: Vec<ConsumableView>,
    pub consumable_offers: Vec<ConsumableView>,
    pub consumable_slots: usize,
    pub reroll_cost: usize,
    pub can_reroll: bool,
    pub can_play: bool,
    pub can_discard: bool,
    /// True once a debug command changed this run.
    pub debug: bool,
}

#[derive(Debug)]
pub enum RunError {
    Unavailable,
    Simulation(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("This action is unavailable."),
            Self::Simulation(error) => f.write_str(error),
        }
    }
}

#[derive(Debug)]
pub struct Outcome {
    pub phase: Phase,
    pub round_score: usize,
    pub campaign_won: bool,
    pub hand_score: Option<usize>,
    pub trace: Option<ScoreTrace>,
    pub trace_time: Duration,
    pub discovered_item: Option<&'static str>,
}
