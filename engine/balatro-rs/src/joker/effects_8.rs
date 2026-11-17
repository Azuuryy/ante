use super::*;

/// Effects of Blueprint, Brainstorm and two Legendary Jokers (Lot 3, batch 6).
///
/// Blueprint copies the Joker to its right and Brainstorm the leftmost one, when that
/// Joker is `blueprint_compatible`: their closures look the target up when they run, so
/// moving Jokers changes what they copy, like the original. A copy runs the target's own
/// effect against its state, then puts that state back: the target does not grow twice
/// (the original's `not context.blueprint`). `Game.copy_depth` stops copies of copies
/// from looping.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    let id = joker.instance_id();
    Some(match joker {
        Jokers::Blueprint(_) | Jokers::Brainstorm(_) => {
            let leftmost = matches!(joker, Jokers::Brainstorm(_));
            let score = move |g: &mut Game, hand: MadeHand| {
                g.run_copied(id, leftmost, |g, effect| {
                    if let Effects::OnScore(f) = effect {
                        f.lock().unwrap()(g, hand.clone());
                    }
                });
            };
            let scored = move |g: &mut Game, card: Card, hand: &MadeHand| {
                g.run_copied(id, leftmost, |g, effect| {
                    if let Effects::OnCardScored(f) = effect {
                        f.lock().unwrap()(g, card, hand);
                    }
                });
            };
            let held = move |g: &mut Game, card: Card, hand: &MadeHand| {
                g.run_copied(id, leftmost, |g, effect| {
                    if let Effects::OnCardHeld(f) = effect {
                        f.lock().unwrap()(g, card, hand);
                    }
                });
            };
            let discarded = move |g: &mut Game, hand: MadeHand| {
                g.run_copied(id, leftmost, |g, effect| {
                    if let Effects::OnDiscard(f) = effect {
                        f.lock().unwrap()(g, hand.clone());
                    }
                });
            };
            let played_again = move |g: &mut Game, card: Card, first: bool| -> usize {
                let mut extra = 0;
                g.run_copied(id, leftmost, |g, effect| {
                    if let Effects::TriggerCountPlayed(f) = effect {
                        extra += f.lock().unwrap()(g, card, first);
                    }
                });
                extra
            };
            let held_again = move |g: &mut Game, card: Card, first: bool| -> usize {
                let mut extra = 0;
                g.run_copied(id, leftmost, |g, effect| {
                    if let Effects::TriggerCountHeld(f) = effect {
                        extra += f.lock().unwrap()(g, card, first);
                    }
                });
                extra
            };
            vec![
                Effects::OnScore(Arc::new(Mutex::new(score))),
                Effects::OnCardScored(Arc::new(Mutex::new(scored))),
                Effects::OnCardHeld(Arc::new(Mutex::new(held))),
                Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
                Effects::TriggerCountPlayed(Arc::new(Mutex::new(played_again))),
                Effects::TriggerCountHeld(Arc::new(Mutex::new(held_again))),
            ]
        }
        Jokers::Triboulet(_) => {
            // Each scored King or Queen gives X2 Mult.
            fn apply(g: &mut Game, card: Card, _hand: &MadeHand) {
                if matches!(card.value, Value::King | Value::Queen) {
                    g.mult *= 2.0;
                    g.note_xmult(2.0);
                }
            }
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Canio(_) => {
            // X1 Mult more per face card destroyed (`Game::destroy_card`).
            let apply = move |g: &mut Game, _hand: MadeHand| {
                let factor = 1.0 + g.joker_state_mut(id).map_or(0.0, |s| s.counter) as f64;
                if factor > 1.0 {
                    g.mult *= factor;
                    g.note_xmult(factor);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        // Read by `Game::spendable`.
        Jokers::CreditCard(_) => vec![],
        // At the end of the round and of the shop (`game/consumable_meta.rs`).
        Jokers::GiftCard(_) | Jokers::Perkeo(_) => vec![],
        _ => return None,
    })
}
