use crate::Card;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandError {
    Empty,
    TooManyCards,
}

pub fn evaluate(cards: &[Card]) -> Result<Hand, HandError> {
    if cards.is_empty() {
        return Err(HandError::Empty);
    }
    if cards.len() > 5 {
        return Err(HandError::TooManyCards);
    }
    let mut counts = [0; 15];
    for card in cards {
        counts[card.rank as usize] += 1;
    }
    let mut groups = [0; 5];
    let mut ranks = [0; 5];
    let mut distinct = 0;
    for (rank, count) in counts
        .into_iter()
        .enumerate()
        .filter(|(_, count)| *count != 0)
    {
        groups[distinct] = count;
        ranks[distinct] = rank;
        distinct += 1;
    }
    let groups = &mut groups[..distinct];
    groups.sort_unstable_by(|a, b| b.cmp(a));
    let flush = cards.len() == 5 && cards.iter().all(|c| c.suit == cards[0].suit);

    let straight = cards.len() == 5
        && distinct == 5
        && (ranks[4] - ranks[0] == 4 || ranks == [2, 3, 4, 5, 14]);
    Ok(match (&*groups, flush, straight) {
        ([5], true, _) => Hand::FlushFive,
        ([3, 2], true, _) => Hand::FlushHouse,
        ([5], _, _) => Hand::FiveOfAKind,
        (_, true, true) => Hand::StraightFlush,
        ([4, ..], _, _) => Hand::FourOfAKind,
        ([3, 2], _, _) => Hand::FullHouse,
        (_, true, _) => Hand::Flush,
        (_, _, true) => Hand::Straight,
        ([3, ..], _, _) => Hand::ThreeOfAKind,
        ([2, 2, ..], _, _) => Hand::TwoPair,
        ([2, ..], _, _) => Hand::Pair,
        _ => Hand::HighCard,
    })
}
