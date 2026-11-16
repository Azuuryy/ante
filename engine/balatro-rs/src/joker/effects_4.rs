use super::*;
use balatro_types::joker::SelectorValue;

/// Effects of the Jokers that score from the board or the round's target, without growing:
/// Swashbuckler, Blackboard, Flower Pot, Seeing Double, Stuntman, Bootstraps, Gros Michel,
/// Cavendish, Ancient Joker and The Idol (`Card:calculate_joker`, `joker_main` and
/// `individual` contexts).
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    let id = joker.instance_id();
    Some(match joker {
        Jokers::Swashbuckler(_) => {
            // `Card:update`: Mult equal to the sell value of the other held Jokers.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                let total: usize = g
                    .jokers
                    .iter()
                    .filter(|other| other.instance_id() != id)
                    .map(|other| g.joker_sell_value(other))
                    .sum();
                g.mult += total as f64;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Blackboard(_) => {
            // X3 when every held card is a Spade or a Club (`is_suit(…, nil, true)`: a Stone
            // card is no suit, a Wild card not debuffed is every suit). An empty hand counts.
            fn apply(g: &mut Game, _hand: MadeHand) {
                let held = g.available.not_selected();
                let black = held.iter().all(|card| {
                    card.enhancement != Some(Enhancement::Stone)
                        && ((card.enhancement == Some(Enhancement::Wild)
                            && !g.is_card_debuffed(card))
                            || matches!(card.suit, Suit::Spade | Suit::Club))
                });
                if black {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::FlowerPot(_) => {
            // X3 when the scoring hand holds a Diamond, a Club, a Heart and a Spade. Plain
            // cards claim their suit first, then each Wild card fills one missing suit.
            fn apply(g: &mut Game, hand: MadeHand) {
                let order = [Suit::Heart, Suit::Diamond, Suit::Spade, Suit::Club];
                if suits_present(g, &hand, order, true)
                    .iter()
                    .all(|&found| found)
                {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::SeeingDouble(_) => {
            // X2 when the scoring hand holds a Club and a card of another suit. Wild cards
            // fill Clubs first.
            fn apply(g: &mut Game, hand: MadeHand) {
                let order = [Suit::Club, Suit::Diamond, Suit::Spade, Suit::Heart];
                let found = suits_present(g, &hand, order, false);
                if found[0] && found[1..].iter().any(|&other| other) {
                    g.mult *= 2.0;
                    g.note_xmult(2.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Stuntman(_) => {
            // +250 Chips; its -2 hand size is in `Game::hand_size`.
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.chips += 250;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Bootstraps(_) => {
            // +2 Mult for every $5 held.
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.mult += (2 * (g.money / 5)) as f64;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::GrosMichel(_) => {
            // +15 Mult; 1 in 6 to go extinct at the end of the round (`handle_round_end`).
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.mult += 15.0;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Cavendish(_) => {
            // X3 Mult; 1 in 1000 to go extinct at the end of the round.
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.mult *= 3.0;
                g.note_xmult(3.0);
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::AncientJoker(_) => {
            // Each scored card of the round's suit gives X1.5 Mult.
            let apply = move |g: &mut Game, card: Card, _hand: &MadeHand| {
                let Some(SelectorValue::Suit(suit)) =
                    g.joker_state_mut(id).and_then(|state| state.selector)
                else {
                    return;
                };
                if g.is_suit(&card, suit) {
                    g.mult *= 1.5;
                    g.note_xmult(1.5);
                }
            };
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::TheIdol(_) => {
            // Each scored card of the round's rank and suit gives X2 Mult.
            let apply = move |g: &mut Game, card: Card, _hand: &MadeHand| {
                let Some(SelectorValue::Card(value, suit)) =
                    g.joker_state_mut(id).and_then(|state| state.selector)
                else {
                    return;
                };
                if card.value == value
                    && card.enhancement != Some(Enhancement::Stone)
                    && g.is_suit(&card, suit)
                {
                    g.mult *= 2.0;
                    g.note_xmult(2.0);
                }
            };
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        _ => return None,
    })
}

/// Which of the four suits of `order` the scoring hand holds, like Flower Pot and Seeing
/// Double count them: each non-Wild card marks its suit (a Stone card none; a debuffed one
/// only with `bypass_debuff`, as Flower Pot's `is_suit(suit, true)`), then each Wild card
/// not debuffed marks the first suit of `order` still missing.
fn suits_present(g: &Game, hand: &MadeHand, order: [Suit; 4], bypass_debuff: bool) -> [bool; 4] {
    let mut found = [false; 4];
    let cards = hand.hand.cards();
    for card in cards
        .iter()
        .filter(|c| c.enhancement != Some(Enhancement::Wild))
    {
        if card.enhancement == Some(Enhancement::Stone)
            || (!bypass_debuff && g.is_card_debuffed(card))
        {
            continue;
        }
        if let Some(slot) = order.iter().position(|suit| *suit == card.suit) {
            found[slot] = true;
        }
    }
    for card in cards
        .iter()
        .filter(|c| c.enhancement == Some(Enhancement::Wild))
    {
        if g.is_card_debuffed(card) {
            continue;
        }
        if let Some(slot) = found.iter().position(|&present| !present) {
            found[slot] = true;
        }
    }
    found
}
