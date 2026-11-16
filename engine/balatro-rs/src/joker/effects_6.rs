use super::*;

/// Effects of the Jokers that need hooks around the hand or the blind (Lot 3, batch 2b):
/// Hiker, Seltzer, Hologram, Campfire, Ceremonial Dagger and Madness. Vampire acts in
/// `Game::jokers_before_scoring`; the blind-start and sale hooks live in
/// `game/joker_hooks.rs`.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    let id = joker.instance_id();
    let counter = move |g: &mut Game| g.joker_state_mut(id).map_or(0.0, |state| state.counter);
    Some(match joker {
        Jokers::Hiker(_) => {
            // Each scored card permanently gains 5 Chips (`perma_bonus`).
            fn apply(g: &mut Game, card: Card, _hand: &MadeHand) {
                g.mutate_card(card.id, |c| c.perma_bonus += 5);
            }
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Seltzer(_) => {
            // Retriggers every played card while it lasts; it wears in `wear_seltzer`.
            fn extra(_g: &mut Game, _card: Card, _is_first: bool) -> usize {
                1
            }
            vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
        }
        Jokers::CeremonialDagger(_) => {
            // +Mult from the Jokers it sliced.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                g.mult += counter(g) as f64;
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Hologram(_) | Jokers::Campfire(_) | Jokers::Madness(_) | Jokers::Vampire(_) => {
            // X1 plus its counter.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                let factor = 1.0 + counter(g) as f64;
                if factor > 1.0 {
                    g.mult *= factor;
                    g.note_xmult(factor);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::ToDoList(_) => {
            // $4 when the hand played is the listed one (`before`).
            let apply = move |g: &mut Game, hand: MadeHand| {
                let listed = g.joker_state_mut(id).and_then(|state| state.selector);
                let rank = if hand.rank == HandRank::RoyalFlush {
                    HandRank::StraightFlush
                } else {
                    hand.rank
                };
                if listed == Some(balatro_types::joker::SelectorValue::Hand(rank)) {
                    g.money += 4;
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        // Paid at cash out (`reward::dollar_bonus`) or through interest and sell value.
        Jokers::DelayedGratification(_)
        | Jokers::Egg(_)
        | Jokers::ToTheMoon(_)
        | Jokers::Cloud9(_)
        | Jokers::Rocket(_)
        | Jokers::Satellite(_) => vec![],
        // Rules read where they apply: probabilities, rerolls, hand size, hands and
        // discards per round, blind select, a lost round, the scoring hand.
        Jokers::OopsAllSixes(_)
        | Jokers::ChaosTheClown(_)
        | Jokers::Burglar(_)
        | Jokers::Juggler(_)
        | Jokers::Drunkard(_)
        | Jokers::Troubadour(_)
        | Jokers::MerryAndy(_)
        | Jokers::MrBones(_)
        | Jokers::Splash(_)
        // Read by hand evaluation (`Game::hand_rules`) and suit checks (`Game::is_suit`).
        | Jokers::FourFingers(_)
        | Jokers::Shortcut(_)
        | Jokers::SmearedJoker(_) => vec![],
        _ => return None,
    })
}
