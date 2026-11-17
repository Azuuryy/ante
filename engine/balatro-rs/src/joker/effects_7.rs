use super::*;

/// Effects of the Jokers that create cards or level hands while a hand scores (Lot 3,
/// batch 5a): 8 Ball, Superposition, Seance, Vagabond and Loyalty Card. Space Joker acts
/// in `Game::jokers_before_scoring`; Hallucination, Cartomancer, Riff-raff, Marble Joker,
/// Diet Cola and Burnt Joker in `game/joker_hooks.rs` and `game/joker_cards.rs`.
pub(super) fn effects(joker: &Jokers) -> Option<Vec<Effects>> {
    let id = joker.instance_id();
    Some(match joker {
        Jokers::EightBall(_) => {
            // Each scored 8: 1 in 4 to create a Tarot card if there is room.
            let apply = move |g: &mut Game, card: Card, _hand: &MadeHand| {
                if card.value == Value::Eight && g.has_consumable_room() && g.prob_roll(1, 4) {
                    g.create_tarot(id);
                }
            };
            vec![Effects::OnCardScored(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Superposition(_) => {
            // A scoring Ace in a hand containing a Straight creates a Tarot card.
            let apply = move |g: &mut Game, hand: MadeHand| {
                let ace = hand.hand.cards().iter().any(|c| c.value == Value::Ace);
                let straight = hand.hand.straight_with(g.hand_rules()).is_some();
                if ace && straight && g.has_consumable_room() {
                    g.create_tarot(id);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Seance(_) => {
            // A hand containing a Straight Flush creates a Spectral card.
            let apply = move |g: &mut Game, hand: MadeHand| {
                let rules = g.hand_rules();
                if SelectHand::new(hand.all.clone())
                    .straight_flush_with(rules)
                    .is_some()
                    && g.has_consumable_room()
                {
                    g.create_spectral(id);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::Vagabond(_) => {
            // A hand played with $4 or less creates a Tarot card.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                if g.money <= 4 && g.has_consumable_room() {
                    g.create_tarot(id);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::LoyaltyCard(_) => {
            // X4 Mult every 6 hands played since it was acquired.
            let apply = move |g: &mut Game, _hand: MadeHand| {
                let played = g.joker_state_mut(id).map_or(0.0, |state| state.counter) as usize;
                if played % 6 == 5 {
                    g.mult *= 4.0;
                    g.note_xmult(4.0);
                }
            };
            vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
        }
        Jokers::DietCola(_) => {
            // Selling it creates a free Double Tag.
            fn sold(g: &mut Game) {
                g.acquire_double_tag();
            }
            vec![Effects::OnSell(Arc::new(Mutex::new(sold)))]
        }
        Jokers::SpaceJoker(_)
        | Jokers::Hallucination(_)
        | Jokers::Cartomancer(_)
        | Jokers::RiffRaff(_)
        | Jokers::MarbleJoker(_)
        | Jokers::BurntJoker(_)
        | Jokers::SixthSense(_)
        | Jokers::Dna(_)
        | Jokers::Certificate(_)
        | Jokers::Showman(_)
        | Jokers::Astronomer(_)
        | Jokers::InvisibleJoker(_) => vec![],
        _ => return None,
    })
}
