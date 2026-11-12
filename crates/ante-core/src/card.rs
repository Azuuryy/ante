#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub id: CardId,
    pub rank: Rank,
    pub suit: Suit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Rank,
    Suit,
}

pub fn sort_cards(cards: &mut [Card], mode: SortMode) {
    match mode {
        SortMode::Rank => cards.sort_by_key(|c| (c.rank as u8, c.suit as u8, c.id)),
        SortMode::Suit => cards.sort_by_key(|c| (c.suit as u8, c.rank as u8, c.id)),
    }
}

pub fn standard_deck() -> Vec<Card> {
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let ranks = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];
    let mut cards = Vec::with_capacity(52);
    for suit in suits {
        for rank in ranks {
            cards.push(Card {
                id: CardId(cards.len() as u32),
                rank,
                suit,
            });
        }
    }
    cards
}
