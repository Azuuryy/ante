//! The 15 starting decks (`Back:apply_to_run` and the rules they keep).

use super::*;
use balatro_types::DeckVariant;

fn game(deck: DeckVariant) -> Game {
    let mut g = Game::new(Config {
        plays: 4,
        discards: 3,
        ..Config::default()
    });
    g.apply_deck(deck);
    g.start();
    g
}

#[test]
fn limits_and_money_follow_the_deck() {
    let red = game(DeckVariant::Red);
    assert_eq!((red.plays_per_round(), red.discards_per_round()), (4, 4));
    let blue = game(DeckVariant::Blue);
    assert_eq!((blue.plays_per_round(), blue.discards_per_round()), (5, 3));
    let base = game(DeckVariant::Green);
    assert_eq!(game(DeckVariant::Yellow).money, base.money + 10);
    let black = game(DeckVariant::Black);
    assert_eq!(black.plays_per_round(), 3);
    assert_eq!(black.joker_slots(), base.joker_slots() + 1);
    let painted = game(DeckVariant::Painted);
    assert_eq!(painted.hand_size(), base.hand_size() + 2);
    assert_eq!(painted.joker_slots(), base.joker_slots() - 1);
    let nebula = game(DeckVariant::Nebula);
    assert!(nebula.vouchers.has(Voucher::Telescope));
    assert_eq!(nebula.consumable_slots(), base.consumable_slots() - 1);
}

#[test]
fn starting_vouchers_and_consumables() {
    let magic = game(DeckVariant::Magic);
    assert!(magic.vouchers.has(Voucher::CrystalBall));
    assert_eq!(magic.consumables, [Consumable::Tarot(Tarot::Fool); 2]);
    let ghost = game(DeckVariant::Ghost);
    assert!(matches!(
        ghost.consumables[..],
        [Consumable::Spectral(crate::spectral::Spectral::Hex)]
    ));
    let zodiac = game(DeckVariant::Zodiac);
    for voucher in [
        Voucher::TarotMerchant,
        Voucher::PlanetMerchant,
        Voucher::Overstock,
    ] {
        assert!(zodiac.vouchers.has(voucher));
    }
}

#[test]
fn the_cards_of_the_deck() {
    let abandoned = game(DeckVariant::Abandoned);
    assert_eq!(abandoned.full_deck().len(), 40);
    assert!(!abandoned.full_deck().iter().any(|c| c.is_face_card()));
    let checkered = game(DeckVariant::Checkered);
    let deck = checkered.full_deck();
    assert_eq!(deck.iter().filter(|c| c.suit == Suit::Spade).count(), 26);
    assert_eq!(deck.iter().filter(|c| c.suit == Suit::Heart).count(), 26);
    let erratic = game(DeckVariant::Erratic);
    assert_eq!(erratic.full_deck().len(), 52);
}

#[test]
fn plasma_doubles_blinds_and_balances_the_score() {
    let mut g = game(DeckVariant::Plasma);
    assert_eq!(g.required_score_for(Some(Blind::Small)), 600);
    // High Card Ace: 16 chips and 1 Mult average to 8 × 8.
    let hand = SelectHand::new(vec![Card::new(Value::Ace, Suit::Spade)])
        .best_hand()
        .unwrap();
    assert_eq!(g.calc_score(hand), 64);
}

#[test]
fn green_pays_hands_and_discards_without_interest() {
    let mut g = game(DeckVariant::Green);
    g.money = 25;
    g.plays = 2;
    g.discards_remaining = 3;
    let reward = g.calc_reward(Blind::Small).unwrap();
    assert_eq!((reward.hand_dollars, reward.discard_dollars), (4, 3));
    assert_eq!(reward.interest, 0);
}

#[test]
fn anaglyph_gives_a_double_tag_after_a_boss() {
    let mut g = game(DeckVariant::Anaglyph);
    g.anaglyph_after_boss();
    assert!(g.tags.contains(&Tag::Double));
}
