use super::*;

/// Effects of the Jokers from TheJoker to SpareTrousers, in collection order.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    Some(match joker {
        Jokers::TheJoker(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.mult += 4.0;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::GreedyJoker(_) => {
            fn apply(g: &mut Game, card: Card, _hand: &MadeHand) {
                if card.matches_suit(Suit::Diamond) {
                    g.mult += 3.0;
                }
            }
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::LustyJoker(_) => {
            fn apply(g: &mut Game, card: Card, _hand: &MadeHand) {
                if card.matches_suit(Suit::Heart) {
                    g.mult += 3.0;
                }
            }
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::WrathfulJoker(_) => {
            fn apply(g: &mut Game, card: Card, _hand: &MadeHand) {
                if card.matches_suit(Suit::Spade) {
                    g.mult += 3.0;
                }
            }
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::GluttonousJoker(_) => {
            fn apply(g: &mut Game, card: Card, _hand: &MadeHand) {
                if card.matches_suit(Suit::Club) {
                    g.mult += 3.0;
                }
            }
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::JollyJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_pair().is_some() {
                    g.mult += (8) as f64
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::ZanyJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_three_of_kind().is_some() {
                    g.mult += (12) as f64
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::MadJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_two_pair().is_some() {
                    g.mult += (10) as f64
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::CrazyJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_straight().is_some() {
                    g.mult += (12) as f64
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::DrollJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_flush().is_some() {
                    g.mult += (10) as f64
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::SlyJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_pair().is_some() {
                    g.chips += 50
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::WilyJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_three_of_kind().is_some() {
                    g.chips += 100
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::CleverJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_two_pair().is_some() {
                    g.chips += 80
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::DeviousJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_straight().is_some() {
                    g.chips += 100
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::CraftyJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_flush().is_some() {
                    g.chips += 80
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::HalfJoker(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.len() <= 3 {
                    g.mult += 20.0;
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::JokerStencil(_) => {
            // X(empty slots + Stencils held), only while a slot is empty.
            fn apply(g: &mut Game, _hand: MadeHand) {
                let empty = g.joker_slots().saturating_sub(g.jokers.len());
                if empty > 0 {
                    let stencils = g
                        .jokers
                        .iter()
                        .filter(|j| matches!(j, Jokers::JokerStencil(_)))
                        .count();
                    let factor = empty + stencils;
                    g.mult *= factor as f64;
                    g.note_xmult(factor as f64);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Banner(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.chips += 30 * g.discards();
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::MysticSummit(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                if g.discards() == 0 {
                    g.mult += 15.0;
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Supernova(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                g.mult += (g.planetarium.level(hand.rank).plays) as f64;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::RideTheBus(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                let has_face = g
                    .non_debuffed(hand.hand.cards().iter())
                    .iter()
                    .any(|c| g.is_face_card(c));
                if has_face {
                    g.consecutive_hands_without_face_card = 0;
                } else {
                    g.consecutive_hands_without_face_card += 1;
                    g.mult += (g.consecutive_hands_without_face_card) as f64;
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::CardSharp(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if g.hand_ranks_played_this_round.contains(&hand.rank) {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Obelisk(_) => {
            // Per-card X0.2 per hand that isn't strictly the most played;
            // back to X1 when it is. Counts include the current hand.
            let id = joker.instance_id();
            let apply = move |g: &mut Game, hand: MadeHand| {
                let rank = if hand.rank == HandRank::RoyalFlush {
                    HandRank::StraightFlush
                } else {
                    hand.rank
                };
                let plays = g.planetarium.level(rank).plays;
                let most_played = HandRank::iter()
                    .filter(|r| *r != rank && *r != HandRank::RoyalFlush)
                    .all(|r| g.planetarium.level(r).plays < plays);
                let Some(state) = g.joker_state_mut(id) else {
                    return;
                };
                if most_played {
                    state.counter = 0.0;
                } else {
                    state.counter += 0.2;
                }
                let extra = state.counter as f64;
                if extra > 0.0 {
                    g.mult *= 1.0 + extra;
                    g.note_xmult(1.0 + extra);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::WeeJoker(wee) => {
            let id = wee.instance_id;
            let apply = move |g: &mut Game, hand: MadeHand| {
                let twos = hand
                    .hand
                    .cards()
                    .iter()
                    .filter(|c| c.value == Value::Two)
                    .count();
                let counter = match g.joker_state_mut(id) {
                    Some(state) => {
                        state.counter += 8.0 * twos as f32;
                        state.counter
                    }
                    None => return,
                };
                g.chips += counter as usize;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::SpareTrousers(st) => {
            let id = st.instance_id;
            let apply = move |g: &mut Game, hand: MadeHand| {
                let has_two_pair = hand.hand.is_two_pair().is_some();
                let counter = match g.joker_state_mut(id) {
                    Some(state) => {
                        if has_two_pair {
                            state.counter += 2.0;
                        }
                        state.counter
                    }
                    None => return,
                };
                g.mult += (counter as usize) as f64;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        _ => return None,
    })
}
