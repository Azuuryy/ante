use super::*;

/// Effects of the Jokers that grow or wear out (Lot 3, batch 2a): Square Joker, Runner,
/// Throwback, Fortune Teller, Flash Card, Red Card, Constellation, Popcorn, Turtle Bean,
/// Ramen, Lucky Cat and Glass Joker. Their value is `JokerState.counter`; the events that
/// move it (reroll, skipped pack, consumable used, round end, discard, shattered Glass)
/// live in `Game::grow_jokers` and its callers.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    let id = joker.instance_id();
    let counter = move |g: &mut Game| g.joker_state_mut(id).map_or(0.0, |state| state.counter);
    Some(match joker {
        Jokers::SquareJoker(_) => {
            // `before`: +4 Chips for a hand of exactly 4 cards, then its Chips.
            let apply = move |g: &mut Game, hand: MadeHand| {
                if hand.all.len() == 4 {
                    if let Some(state) = g.joker_state_mut(id) {
                        state.counter += 4.0;
                    }
                }
                g.chips += counter(g) as usize;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Runner(_) => {
            // `before`: +15 Chips when the hand contains a Straight, then its Chips.
            let apply = move |g: &mut Game, hand: MadeHand| {
                if hand.hand.straight_with(g.hand_rules()).is_some() {
                    if let Some(state) = g.joker_state_mut(id) {
                        state.counter += 15.0;
                    }
                }
                g.chips += counter(g) as usize;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Throwback(_) => {
            // X0.25 Mult per blind skipped this run.
            fn apply(g: &mut Game, _hand: MadeHand) {
                let factor = 1.0 + 0.25 * g.skipped_count as f64;
                if factor > 1.0 {
                    g.mult *= factor;
                    g.note_xmult(factor);
                }
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::FortuneTeller(_) => {
            // +1 Mult per Tarot used this run.
            fn apply(g: &mut Game, _hand: MadeHand) {
                g.mult += g.tarots_used as f64;
            }
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::FlashCard(_) | Jokers::RedCard(_) => {
            // +2 Mult per reroll, +3 Mult per Booster Pack skipped.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                g.mult += counter(g) as f64;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Popcorn(_) => {
            // +20 Mult, 4 less after each round.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                g.mult += (20.0 - counter(g)).max(0.0) as f64;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Constellation(_)
        | Jokers::LuckyCat(_)
        | Jokers::GlassJoker(_)
        | Jokers::Ramen(_) => {
            // XMult from 1 (2 for Ramen) moved by its counter.
            let base = if matches!(joker, Jokers::Ramen(_)) {
                2.0
            } else {
                1.0
            };
            let sign = if matches!(joker, Jokers::Ramen(_)) {
                -1.0
            } else {
                1.0
            };
            let apply = move |g: &mut Game, _hand: MadeHand| {
                let factor = base + sign * counter(g) as f64;
                if factor > 1.0 {
                    g.mult *= factor;
                    g.note_xmult(factor);
                }
            };
            let mut effects = vec![Effects::OnScore(Arc::new(Mutex::new(apply)))];
            if matches!(joker, Jokers::LuckyCat(_)) {
                // X0.25 more each time a Lucky card triggers.
                let lucky = move |g: &mut Game, card: Card, _hand: &MadeHand| {
                    if g.lucky_trigger == Some(card.id) {
                        if let Some(state) = g.joker_state_mut(id) {
                            state.counter += 0.25;
                        }
                    }
                };
                effects.insert(0, Effects::OnCardScored(Arc::new(Mutex::new(lucky))));
            }
            effects
        }
        // Its hand size is in `Game::hand_size`, its wear in `Game::wear_out_jokers`.
        Jokers::TurtleBean(_) => vec![],
        _ => return None,
    })
}
