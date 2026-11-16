//! Lot 3, batch 1: Jokers that score from the board or the round's target.

use super::*;
use crate::event::{GameEvent, JokerNote};
use balatro_types::joker::SelectorValue;

/// A Small Blind with `jokers` bought in order and `held` cards in hand besides the played
/// ones; money back to `money`.
fn game_with(jokers: Vec<Jokers>, held: Vec<Card>, money: usize) -> Game {
    let mut g = Game {
        stage: Stage::Blind(Blind::Small),
        ..Default::default()
    };
    g.money += 1000;
    for mut joker in jokers {
        // Distinct instances, as the shop mints them.
        joker.set_instance_id(crate::joker::mint_joker_id());
        g.stage = Stage::Shop();
        g.shop.jokers.push(joker.clone());
        g.buy_joker(joker).unwrap();
    }
    g.stage = Stage::Blind(Blind::Small);
    g.money = money;
    g.available.extend(held);
    g
}

fn score(g: &mut Game, cards: Vec<Card>) -> usize {
    g.calc_score(SelectHand::new(cards).best_hand().unwrap())
}

fn ace() -> Card {
    Card::new(Value::Ace, Suit::Spade)
}

#[test]
fn swashbuckler_adds_the_other_jokers_sell_values() {
    // The Joker sells for $1: High Card Ace (16 chips) × (1 + 4 + 1).
    let mut g = game_with(
        vec![
            Jokers::TheJoker(Default::default()),
            Jokers::Swashbuckler(Default::default()),
        ],
        vec![],
        0,
    );
    assert_eq!(score(&mut g, vec![ace()]), 16 * 6);
}

#[test]
fn blackboard_needs_every_held_card_black() {
    let spades = vec![
        Card::new(Value::Two, Suit::Spade),
        Card::new(Value::Three, Suit::Club),
    ];
    let mut g = game_with(vec![Jokers::Blackboard(Default::default())], spades, 0);
    assert_eq!(score(&mut g, vec![ace()]), 16 * 3);
    let mixed = vec![Card::new(Value::Two, Suit::Heart)];
    let mut g = game_with(vec![Jokers::Blackboard(Default::default())], mixed, 0);
    assert_eq!(score(&mut g, vec![ace()]), 16);
}

#[test]
fn flower_pot_needs_four_suits_and_a_wild_card_fills_one() {
    let straight = |last: Card| {
        vec![
            Card::new(Value::Two, Suit::Heart),
            Card::new(Value::Three, Suit::Diamond),
            Card::new(Value::Four, Suit::Spade),
            Card::new(Value::Five, Suit::Heart),
            last,
        ]
    };
    let mut g = game_with(vec![Jokers::FlowerPot(Default::default())], vec![], 0);
    let plain = score(&mut g, straight(Card::new(Value::Six, Suit::Heart)));
    let all_four = score(&mut g, straight(Card::new(Value::Six, Suit::Club)));
    assert_eq!(all_four, plain * 3);
    let mut wild = Card::new(Value::Six, Suit::Heart);
    wild.enhancement = Some(Enhancement::Wild);
    assert_eq!(score(&mut g, straight(wild)), plain * 3);
}

#[test]
fn seeing_double_needs_a_club_and_another_suit() {
    let pair = |second: Suit| {
        vec![
            Card::new(Value::King, Suit::Club),
            Card::new(Value::King, second),
        ]
    };
    let mut g = game_with(vec![Jokers::SeeingDouble(Default::default())], vec![], 0);
    let clubs = score(&mut g, pair(Suit::Club));
    assert_eq!(score(&mut g, pair(Suit::Heart)), clubs * 2);
}

#[test]
fn stuntman_adds_chips_and_takes_two_hand_slots() {
    let g = game_with(vec![], vec![], 0);
    let size = g.hand_size();
    let mut g = game_with(vec![Jokers::Stuntman(Default::default())], vec![], 0);
    assert_eq!(g.hand_size(), size - 2);
    assert_eq!(score(&mut g, vec![ace()]), 16 + 250);
}

#[test]
fn bootstraps_adds_two_mult_per_five_dollars() {
    let mut g = game_with(vec![Jokers::Bootstraps(Default::default())], vec![], 14);
    assert_eq!(score(&mut g, vec![ace()]), 16 * 5);
}

#[test]
fn gros_michel_goes_extinct_and_lets_cavendish_in() {
    let mut g = game_with(vec![Jokers::GrosMichel(Default::default())], vec![], 0);
    assert_eq!(score(&mut g, vec![ace()]), 16 * 16);
    g.take_events();
    g.prob_mult = 6; // 6 in 6: the roll always hits.
    g.handle_round_end();
    assert!(g.jokers.is_empty());
    assert!(g.gros_michel_extinct);
    let notes: Vec<_> = g
        .take_events()
        .into_iter()
        .filter_map(|event| match event {
            GameEvent::JokerMessage { note, .. } => Some(note),
            _ => None,
        })
        .collect();
    assert_eq!(notes, [JokerNote::Extinct]);
}

#[test]
fn cavendish_only_appears_once_gros_michel_is_extinct() {
    use crate::shop::generators::JokerGenerator;
    let mut rng = rand::thread_rng();
    let draw = |extinct: bool, rng: &mut rand::rngs::ThreadRng| {
        let generator = JokerGenerator {
            gros_michel_extinct: extinct,
        };
        (0..400)
            .map(|_| generator.gen_joker_of_rarity(1, &[], Rarity::Common, rng))
            .collect::<Vec<_>>()
    };
    let before = draw(false, &mut rng);
    assert!(!before.iter().any(|j| matches!(j, Jokers::Cavendish(_))));
    let after = draw(true, &mut rng);
    assert!(!after.iter().any(|j| matches!(j, Jokers::GrosMichel(_))));
    assert!(after.iter().any(|j| matches!(j, Jokers::Cavendish(_))));
}

#[test]
fn ancient_joker_and_the_idol_follow_the_rounds_target() {
    let mut g = game_with(
        vec![
            Jokers::AncientJoker(Default::default()),
            Jokers::TheIdol(Default::default()),
        ],
        vec![],
        0,
    );
    g.jokers[0].state_mut().selector = Some(SelectorValue::Suit(Suit::Spade));
    g.jokers[1].state_mut().selector = Some(SelectorValue::Card(Value::Ace, Suit::Spade));
    // X1.5 then X2 on the Ace of Spades.
    assert_eq!(score(&mut g, vec![ace()]), 16 * 3);
    g.jokers[1].state_mut().selector = Some(SelectorValue::Card(Value::King, Suit::Spade));
    assert_eq!(score(&mut g, vec![ace()]), 24);
}

#[test]
fn round_targets_come_from_the_deck_and_the_suit_moves_on() {
    let mut g = Game::default();
    g.start();
    g.jokers.push(Jokers::AncientJoker(Default::default()));
    g.jokers.push(Jokers::TheIdol(Default::default()));
    g.roll_card_targets(false);
    let Some(SelectorValue::Suit(first)) = g.jokers[0].state().selector else {
        panic!("Ancient Joker got a suit")
    };
    let Some(SelectorValue::Card(value, suit)) = g.jokers[1].state().selector else {
        panic!("The Idol got a card")
    };
    assert!(g
        .full_deck()
        .iter()
        .any(|c| c.value == value && c.suit == suit));
    for _ in 0..20 {
        let before = g.jokers[0].state().selector;
        g.roll_card_targets(true);
        assert_ne!(
            g.jokers[0].state().selector,
            before,
            "a suit other than the last"
        );
    }
    let _ = first;
}
