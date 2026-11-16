use super::*;

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
    /// A Joker's effect on one scored or held card: the popup shows on the card and the
    /// Joker juices, like `card_eval_status_text` on the card in `evaluate_play`.
    JokerOnCard {
        joker: JokerKey,
        kind: ContentId,
        name: &'static str,
        card: CardKey,
    },
    /// The boss refused the hand: "Not Allowed!" over the play area, like
    /// `play_area_status_text` in `evaluate_play`. The hand scores nothing.
    NotAllowed,
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
