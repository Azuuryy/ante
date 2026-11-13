use super::*;

/// Effects of the Jokers from Fibonacci to TheTrio, in collection order.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    Some(match joker {
        Jokers::Fibonacci(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in _hand.hand.cards() {
                    if card.value == Value::Ace
                        || card.value == Value::Two
                        || card.value == Value::Three
                        || card.value == Value::Five
                        || card.value == Value::Eight
                    {
                        g.mult += 8.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::ScaryFace(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(_hand.hand.cards().iter()) {
                    if g.is_face_card(&card) {
                        g.chips += 30;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::AbstractJoker(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.mult += (g.jokers.len() * 3) as f64;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Pareidolia(_) => vec![Effects::RuleFlag(RuleFlag::AllCardsAreFace)],
        Jokers::EvenSteven(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in _hand.hand.cards() {
                    if g.is_even(&card) {
                        g.mult += 4.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::OddTodd(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in _hand.hand.cards() {
                    if g.is_odd(&card) {
                        g.chips += 31;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Scholar(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in _hand.hand.cards() {
                    if card.value == Value::Ace {
                        g.chips += 20;
                        g.mult += 4.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::BusinessCard(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(_hand.hand.cards().iter()) {
                    if g.is_face_card(&card) && g.prob_roll(1, 2) {
                        g.money += 2;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::FacelessJoker(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                if _hand.all.len() >= 5 {
                    g.money += 5;
                }
            }
            vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Baron(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let kings = g
                    .non_debuffed(g.available.not_selected().iter())
                    .iter()
                    .filter(|c| c.value == Value::King)
                    .count();
                for _ in 0..kings {
                    g.mult *= 1.5;
                    g.note_xmult(1.5);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::MidasMask(_) => {
            fn apply(g: &mut Game, hand: &mut MadeHand) {
                // only cards that are actually scored (hand.hand), not
                // unscored kickers (hand.all), become Gold
                let cards: Vec<Card> = hand
                    .hand
                    .cards()
                    .into_iter()
                    .map(|mut c| {
                        if g.is_face_card(&c) && !g.is_card_debuffed(&c) {
                            c.enhancement = Some(Enhancement::Gold);
                            g.mutate_card(c.id, |c| c.enhancement = Some(Enhancement::Gold));
                        }
                        c
                    })
                    .collect();
                hand.hand = SelectHand::new(cards);
            }
            vec![Effects::OnModifyHand(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Photograph(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(_hand.hand.cards().iter()) {
                    if g.is_face_card(&card) {
                        g.mult *= 2.0;
                        g.note_xmult(2.0);
                        break;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::ReservedParking(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(g.available.not_selected().iter()) {
                    if g.is_face_card(&card) && g.prob_roll(1, 2) {
                        g.money += 1;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::BaseballCard(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let uncommon = g
                    .jokers
                    .iter()
                    .filter(|j| j.rarity() == Rarity::Uncommon)
                    .count();
                for _ in 0..uncommon {
                    g.mult *= 1.5;
                    g.note_xmult(1.5);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Bull(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.chips += g.money * 2;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::WalkieTalkie(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in _hand.hand.cards() {
                    if card.value == Value::Ten || card.value == Value::Four {
                        g.chips += 10;
                        g.mult += 4.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::SmileyFace(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(_hand.hand.cards().iter()) {
                    if g.is_face_card(&card) {
                        g.mult += 5.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::GoldenTicket(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in _hand.hand.cards() {
                    if card.enhancement == Some(Enhancement::Gold) {
                        g.money += 4;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Acrobat(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                if g.plays == 0 {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::RoughGem(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(_hand.hand.cards().iter()) {
                    if card.matches_suit(Suit::Diamond) {
                        g.money += 1;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Bloodstone(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.non_debuffed(_hand.hand.cards().iter()) {
                    if card.matches_suit(Suit::Heart) && g.prob_roll(1, 2) {
                        g.mult *= 1.5;
                        g.note_xmult(1.5);
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Arrowhead(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                for card in g.non_debuffed(hand.hand.cards().iter()) {
                    if card.matches_suit(Suit::Spade) {
                        g.chips += 50;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::OnyxAgate(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                for card in g.non_debuffed(hand.hand.cards().iter()) {
                    if card.matches_suit(Suit::Club) {
                        g.mult += 7.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::ShootTheMoon(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                for card in g.available.not_selected() {
                    if card.value == Value::Queen {
                        g.mult += 13.0;
                    }
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::RaisedFist(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                // no dedicated numeric "rank" accessor exists; chips() is the
                // closest analog to the card's face rank
                // TODO: need to do this with rank ord?
                let lowest = g.available.not_selected().iter().map(|c| c.chips()).min();
                if let Some(lowest) = lowest {
                    g.mult += (lowest * 2) as f64;
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::TheDuo(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_pair().is_some() {
                    g.mult *= 2.0;
                    g.note_xmult(2.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::TheTrio(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_three_of_kind().is_some() {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        _ => return None,
    })
}
