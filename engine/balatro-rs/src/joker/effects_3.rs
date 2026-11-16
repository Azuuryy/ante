use super::*;

/// Effects of the Jokers from TheFamily to IceCream, in collection order.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    Some(match joker {
        Jokers::TheFamily(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.is_four_of_kind().is_some() {
                    g.mult *= 4.0;
                    g.note_xmult(4.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::TheOrder(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.straight_with(g.hand_rules()).is_some() {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::TheTribe(_) => {
            fn apply(g: &mut Game, hand: MadeHand) {
                if hand.hand.flush_with(g.hand_rules()).is_some() {
                    g.mult *= 2.0;
                    g.note_xmult(2.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::StoneJoker(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let count = g
                    .full_deck()
                    .iter()
                    .filter(|c| c.enhancement == Some(Enhancement::Stone))
                    .count();
                g.chips += count * 25;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::SteelJoker(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let count = g
                    .full_deck()
                    .iter()
                    .filter(|c| c.enhancement == Some(Enhancement::Steel))
                    .count();
                g.mult *= 1.0 + 0.2 * count as f64;
                g.note_xmult(1.0 + 0.2 * count as f64);
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::BlueJoker(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.chips += g.deck.len() * 2;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Erosion(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let deficit = 52usize.saturating_sub(g.full_deck().len());
                g.mult += (deficit * 4) as f64;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::DriversLicense(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let enhanced = g
                    .full_deck()
                    .iter()
                    .filter(|c| c.enhancement.is_some())
                    .count();
                if enhanced >= 16 {
                    g.mult *= 3.0;
                    g.note_xmult(3.0);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Dusk(_) => {
            fn extra(g: &mut Game, _card: Card, _is_first: bool) -> usize {
                if g.plays == 0 {
                    1
                } else {
                    0
                }
            }
            vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
        }
        Jokers::Hack(_) => {
            fn extra(_g: &mut Game, card: Card, _is_first: bool) -> usize {
                if matches!(
                    card.value,
                    Value::Two | Value::Three | Value::Four | Value::Five
                ) {
                    1
                } else {
                    0
                }
            }
            vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
        }
        Jokers::SockAndBuskin(_) => {
            fn extra(g: &mut Game, card: Card, _is_first: bool) -> usize {
                if g.is_face_card(&card) {
                    1
                } else {
                    0
                }
            }
            vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
        }
        Jokers::HangingChad(_) => {
            fn extra(_g: &mut Game, _card: Card, is_first: bool) -> usize {
                if is_first {
                    2
                } else {
                    0
                }
            }
            vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
        }
        Jokers::Mime(_) => {
            fn extra(_g: &mut Game, _card: Card, _is_first: bool) -> usize {
                1
            }
            vec![Effects::TriggerCountHeld(Arc::new(Mutex::new(extra)))]
        }
        Jokers::TradingCard(_) => {
            // "First discard of round" == joker.discards already
            // decremented to config.discards - 1 by the time this fires
            // (discard_selected decrements before invoking OnDiscard).
            let id = joker.instance_id();
            let apply = move |g: &mut Game, hand: MadeHand| {
                if hand.all.len() == 1
                    && g.discards_remaining == g.config.discards.saturating_sub(1)
                {
                    g.destroy_card(hand.all[0].id);
                    g.money += 3;
                    g.joker_message(id, JokerNote::Dollars(3));
                }
            };
            vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
        }
        Jokers::GreenJoker(_) => {
            // +1 Mult per hand played, -1 per discard, kept for the whole run.
            let id = joker.instance_id();
            let played = move |g: &mut Game, _hand: MadeHand| {
                let Some(state) = g.joker_state_mut(id) else {
                    return;
                };
                state.counter += 1.0;
                let mult = state.counter as usize;
                g.mult += (mult) as f64;
            };
            let discarded = move |g: &mut Game, _hand: MadeHand| {
                let Some(state) = g.joker_state_mut(id) else {
                    return;
                };
                let before = state.counter;
                state.counter = (state.counter - 1.0).max(0.0);
                if state.counter != before {
                    g.joker_message(id, JokerNote::MultMinus(1));
                }
            };
            vec![
                Effects::OnScore(Arc::new(Mutex::new(played))),
                Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
            ]
        }
        Jokers::HitTheRoad(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                let jacks = g
                    .non_debuffed(g.discarded_this_round.iter())
                    .iter()
                    .filter(|c| c.value == Value::Jack)
                    .count();
                g.mult *= 1.0 + 0.5 * jacks as f64;
                g.note_xmult(1.0 + 0.5 * jacks as f64);
            }
            // "X1.5", "X2"… on the Joker for each Jack discarded, counting the round's.
            let id = joker.instance_id();
            let discarded = move |g: &mut Game, hand: MadeHand| {
                let total = g
                    .non_debuffed(g.discarded_this_round.iter())
                    .iter()
                    .filter(|c| c.value == Value::Jack)
                    .count();
                let now = g
                    .non_debuffed(hand.all.iter())
                    .iter()
                    .filter(|c| c.value == Value::Jack)
                    .count();
                for jacks in total.saturating_sub(now) + 1..=total {
                    g.joker_message(id, JokerNote::XMult(100 + 50 * jacks));
                }
            };
            vec![
                Effects::OnScore(Arc::new(Mutex::new(apply))),
                Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
            ]
        }
        Jokers::Yorick(_) => {
            // X1 more every 23 cards discarded since this Yorick was acquired.
            let id = joker.instance_id();
            let played = move |g: &mut Game, _hand: MadeHand| {
                let Some(state) = g.joker_state_mut(id) else {
                    return;
                };
                let levels = (state.counter as usize) / 23;
                if levels > 0 {
                    g.mult *= 1.0 + levels as f64;
                    g.note_xmult(1.0 + levels as f64);
                }
            };
            let discarded = move |g: &mut Game, hand: MadeHand| {
                let Some(state) = g.joker_state_mut(id) else {
                    return;
                };
                let before = state.counter as usize / 23;
                state.counter += hand.all.len() as f32;
                let after = state.counter as usize / 23;
                for level in before + 1..=after {
                    g.joker_message(id, JokerNote::XMult(100 * (1 + level)));
                }
            };
            vec![
                Effects::OnScore(Arc::new(Mutex::new(played))),
                Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
            ]
        }
        Jokers::Castle(castle) => {
            // +3 Chips per discarded card of the round's suit, kept for the
            // whole run. The suit is picked in Game::clear_blind, so it is
            // looked up live.
            let id = castle.instance_id;
            let played = move |g: &mut Game, _hand: MadeHand| {
                if let Some(state) = g.joker_state_mut(id) {
                    let chips = state.counter as usize;
                    g.chips += chips;
                }
            };
            let discarded = move |g: &mut Game, hand: MadeHand| {
                let Some(SelectorValue::Suit(suit)) =
                    g.joker_state_mut(id).and_then(|s| s.selector)
                else {
                    return;
                };
                let count = g
                    .non_debuffed(hand.all.iter())
                    .iter()
                    .filter(|c| g.is_suit(c, suit))
                    .count();
                if let Some(state) = g.joker_state_mut(id) {
                    state.counter += 3.0 * count as f32;
                }
                for _ in 0..count {
                    g.joker_message(id, JokerNote::Upgrade);
                }
            };
            vec![
                Effects::OnScore(Arc::new(Mutex::new(played))),
                Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
            ]
        }
        Jokers::MailInRebate(mail) => {
            // Immediate payout on the triggering discard itself, not an
            // accumulated round total - unlike Castle/HitTheRoad, matches
            // real card text ("Earn $5 for each discarded [rank]"). Selector
            // looked up live for the same reason as Castle above.
            let id = mail.instance_id;
            let apply = move |g: &mut Game, hand: MadeHand| {
                let Some(SelectorValue::Value(value)) =
                    g.joker_state_mut(id).and_then(|s| s.selector)
                else {
                    return;
                };
                // A debuffed card does not pay.
                let count = g
                    .non_debuffed(hand.all.iter())
                    .iter()
                    .filter(|c| c.value == value)
                    .count();
                g.money += count * 5;
                for _ in 0..count {
                    g.joker_message(id, JokerNote::Dollars(5));
                }
            };
            vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Luchador(_) => {
            fn apply(g: &mut Game) {
                g.boss_disabled_by_luchador = true;
            }
            vec![Effects::OnSell(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Matador(_) => {
            fn apply(g: &mut Game, _hand: MadeHand) {
                if g.boss_triggered_this_hand {
                    g.money += 8;
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        #[cfg(feature = "ante-extra-jokers")]
        Jokers::Misprint(_) => vec![Effects::OnScore(Arc::new(Mutex::new(
            |g: &mut Game, _: MadeHand| g.mult += g.roll_misprint_mult() as f64,
        )))],
        // Paid at cash out on its own row: `reward::dollar_bonus`.
        #[cfg(feature = "ante-extra-jokers")]
        Jokers::GoldenJoker(_) => vec![],
        #[cfg(feature = "ante-extra-jokers")]
        Jokers::IceCream(_) => {
            let remaining = 100_usize.saturating_sub(5 * joker.state().counter as usize);
            vec![Effects::OnScore(Arc::new(Mutex::new(
                move |g: &mut Game, _: MadeHand| g.chips += remaining,
            )))]
        }
        _ => return None,
    })
}
