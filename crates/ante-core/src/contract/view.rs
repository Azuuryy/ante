use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    BeforeBlind,
    Playing,
    Reward,
    Shop,
    Pack,
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
    /// The shop's "Buy & Use" is available for this offer.
    pub can_buy_and_use: bool,
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
    /// Family of the open booster pack.
    pub open_pack: Option<PackFamily>,
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
    /// The rows that make up `reward`, in the order of `G.FUNCS.evaluate_round`; empty
    /// outside the Reward phase.
    pub reward_rows: Vec<RewardRow>,
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

/// One row of the round evaluation screen and the dollars it adds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardRow {
    pub kind: RewardKind,
    pub dollars: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewardKind {
    /// The cleared Blind's own reward.
    Blind,
    /// $1 per hand left.
    Hands { count: usize },
    /// A Joker's end-of-round dollar bonus.
    Joker {
        id: JokerKey,
        kind: ContentId,
        name: &'static str,
    },
    /// A tag paying out, such as Investment.
    Tag(ContentId),
    /// $1 per $5 held, up to the cap.
    Interest,
}
