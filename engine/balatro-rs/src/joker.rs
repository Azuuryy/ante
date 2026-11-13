use crate::card::{Card, Edition, Enhancement, Suit, Value};
use crate::effect::{Effects, RuleFlag};
use crate::game::Game;
use crate::hand::{MadeHand, SelectHand};
use crate::rank::HandRank;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

pub use balatro_types::joker::*;

static JOKER_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub(crate) fn mint_joker_id() -> usize {
    JOKER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub(crate) fn ensure_joker_id_counter_past(max_seen: usize) {
    JOKER_ID_COUNTER.fetch_max(max_seen + 1, Ordering::SeqCst);
}

/// Assigns a fresh rotating selector to joker if it's a selector-type joker.
pub(crate) fn roll_discard_selector<R: Rng + ?Sized>(rng: &mut R, j: &mut Jokers) {
    const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];
    const VALUES: [Value; 13] = [
        Value::Two,
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
        Value::Ace,
    ];
    match j {
        Jokers::Castle(_) => {
            let suit = SUITS[rng.gen_range(0..SUITS.len())];
            j.state_mut().selector = Some(SelectorValue::Suit(suit));
        }
        Jokers::MailInRebate(_) => {
            let value = VALUES[rng.gen_range(0..VALUES.len())];
            j.state_mut().selector = Some(SelectorValue::Value(value));
        }
        _ => {}
    }
}

/// `balatro_types::Jokers` already supplies all static joker data
/// (name/rarity/cost/desc/category/etc.) as inherent methods.
/// The one thing that can't live there is game behavior.
pub trait JokerEffects {
    fn effects(&self, game: &Game) -> Vec<Effects>;

    /// Whether this joker's `effects()` has real logic behind it.
    /// Keeps unimplemented jokers out of shop/pack generation.
    fn is_implemented(&self) -> bool;
}

impl JokerEffects for Jokers {
    fn effects(&self, game: &Game) -> Vec<Effects> {
        let _ = game;
        match self {
            Self::TheJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.mult += 4.0;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GreedyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let diamonds = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Diamond))
                        .count();
                    g.mult += (diamonds * 3) as f64
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::LustyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let hearts = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Heart))
                        .count();
                    g.mult += (hearts * 3) as f64
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WrathfulJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let spades = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Spade))
                        .count();
                    g.mult += (spades * 3) as f64
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GluttonousJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    let clubs = g
                        .non_debuffed(hand.hand.cards().iter())
                        .iter()
                        .filter(|c| c.matches_suit(Suit::Club))
                        .count();
                    g.mult += (clubs * 3) as f64
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::JollyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_pair().is_some() {
                        g.mult += (8) as f64
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ZanyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_three_of_kind().is_some() {
                        g.mult += (12) as f64
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MadJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_two_pair().is_some() {
                        g.mult += (10) as f64
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CrazyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_straight().is_some() {
                        g.mult += (12) as f64
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::DrollJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_flush().is_some() {
                        g.mult += (10) as f64
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::SlyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_pair().is_some() {
                        g.chips += 50
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WilyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_three_of_kind().is_some() {
                        g.chips += 100
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CleverJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_two_pair().is_some() {
                        g.chips += 80
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::DeviousJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_straight().is_some() {
                        g.chips += 100
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::CraftyJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_flush().is_some() {
                        g.chips += 80
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::HalfJoker(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.len() <= 3 {
                        g.mult += 20.0;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::JokerStencil(_) => {
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
            Self::Banner(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.chips += 30 * g.discards();
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::MysticSummit(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.discards() == 0 {
                        g.mult += 15.0;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Supernova(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    g.mult += (g.planetarium.level(hand.rank).plays) as f64;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::RideTheBus(_) => {
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
            Self::CardSharp(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if g.hand_ranks_played_this_round.contains(&hand.rank) {
                        g.mult *= 3.0;
                        g.note_xmult(3.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Obelisk(_) => {
                // Per-card X0.2 per hand that isn't strictly the most played;
                // back to X1 when it is. Counts include the current hand.
                let id = self.instance_id();
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
            Self::WeeJoker(wee) => {
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
            Self::SpareTrousers(st) => {
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
            Self::Fibonacci(_) => {
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
            Self::ScaryFace(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) {
                            g.chips += 30;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::AbstractJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.mult += (g.jokers.len() * 3) as f64;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Pareidolia(_) => vec![Effects::RuleFlag(RuleFlag::AllCardsAreFace)],
            Self::EvenSteven(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if g.is_even(&card) {
                            g.mult += 4.0;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::OddTodd(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if g.is_odd(&card) {
                            g.chips += 31;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Scholar(_) => {
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
            Self::BusinessCard(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) && g.prob_roll(1, 2) {
                            g.money += 2;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::FacelessJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if _hand.all.len() >= 5 {
                        g.money += 5;
                    }
                }
                vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
            }
            Self::Baron(_) => {
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
            Self::MidasMask(_) => {
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
            Self::Photograph(_) => {
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
            Self::ReservedParking(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(g.available.not_selected().iter()) {
                        if g.is_face_card(&card) && g.prob_roll(1, 2) {
                            g.money += 1;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::BaseballCard(_) => {
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
            Self::Bull(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.chips += g.money * 2;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::WalkieTalkie(_) => {
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
            Self::SmileyFace(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if g.is_face_card(&card) {
                            g.mult += 5.0;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::GoldenTicket(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in _hand.hand.cards() {
                        if card.enhancement == Some(Enhancement::Gold) {
                            g.money += 4;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Acrobat(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.plays == 0 {
                        g.mult *= 3.0;
                        g.note_xmult(3.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::RoughGem(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.non_debuffed(_hand.hand.cards().iter()) {
                        if card.matches_suit(Suit::Diamond) {
                            g.money += 1;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Bloodstone(_) => {
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
            Self::Arrowhead(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    for card in g.non_debuffed(hand.hand.cards().iter()) {
                        if card.matches_suit(Suit::Spade) {
                            g.chips += 50;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::OnyxAgate(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    for card in g.non_debuffed(hand.hand.cards().iter()) {
                        if card.matches_suit(Suit::Club) {
                            g.mult += 7.0;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::ShootTheMoon(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    for card in g.available.not_selected() {
                        if card.value == Value::Queen {
                            g.mult += 13.0;
                        }
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::RaisedFist(_) => {
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
            Self::TheDuo(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_pair().is_some() {
                        g.mult *= 2.0;
                        g.note_xmult(2.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheTrio(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_three_of_kind().is_some() {
                        g.mult *= 3.0;
                        g.note_xmult(3.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheFamily(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_four_of_kind().is_some() {
                        g.mult *= 4.0;
                        g.note_xmult(4.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheOrder(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_straight().is_some() {
                        g.mult *= 3.0;
                        g.note_xmult(3.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::TheTribe(_) => {
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.hand.is_flush().is_some() {
                        g.mult *= 2.0;
                        g.note_xmult(2.0);
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::StoneJoker(_) => {
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
            Self::SteelJoker(_) => {
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
            Self::BlueJoker(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    g.chips += g.deck.len() * 2;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Erosion(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let deficit = 52usize.saturating_sub(g.full_deck().len());
                    g.mult += (deficit * 4) as f64;
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::DriversLicense(_) => {
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
            Self::Dusk(_) => {
                fn extra(g: &mut Game, _card: Card, _is_first: bool) -> usize {
                    if g.plays == 0 {
                        1
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::Hack(_) => {
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
            Self::SockAndBuskin(_) => {
                fn extra(g: &mut Game, card: Card, _is_first: bool) -> usize {
                    if g.is_face_card(&card) {
                        1
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::HangingChad(_) => {
                fn extra(_g: &mut Game, _card: Card, is_first: bool) -> usize {
                    if is_first {
                        2
                    } else {
                        0
                    }
                }
                vec![Effects::TriggerCountPlayed(Arc::new(Mutex::new(extra)))]
            }
            Self::Mime(_) => {
                fn extra(_g: &mut Game, _card: Card, _is_first: bool) -> usize {
                    1
                }
                vec![Effects::TriggerCountHeld(Arc::new(Mutex::new(extra)))]
            }
            Self::TradingCard(_) => {
                // "First discard of round" == self.discards already
                // decremented to config.discards - 1 by the time this fires
                // (discard_selected decrements before invoking OnDiscard).
                fn apply(g: &mut Game, hand: MadeHand) {
                    if hand.all.len() == 1
                        && g.discards_remaining == g.config.discards.saturating_sub(1)
                    {
                        g.destroy_card(hand.all[0].id);
                        g.money += 3;
                    }
                }
                vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
            }
            Self::GreenJoker(_) => {
                // +1 Mult per hand played, -1 per discard, kept for the whole run.
                let id = self.instance_id();
                let played = move |g: &mut Game, _hand: MadeHand| {
                    let Some(state) = g.joker_state_mut(id) else {
                        return;
                    };
                    state.counter += 1.0;
                    let mult = state.counter as usize;
                    g.mult += (mult) as f64;
                };
                let discarded = move |g: &mut Game, _hand: MadeHand| {
                    if let Some(state) = g.joker_state_mut(id) {
                        state.counter = (state.counter - 1.0).max(0.0);
                    }
                };
                vec![
                    Effects::OnScore(Arc::new(Mutex::new(played))),
                    Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
                ]
            }
            Self::HitTheRoad(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    let jacks = g
                        .non_debuffed(g.discarded_this_round.iter())
                        .iter()
                        .filter(|c| c.value == Value::Jack)
                        .count();
                    g.mult *= 1.0 + 0.5 * jacks as f64;
                    g.note_xmult(1.0 + 0.5 * jacks as f64);
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            Self::Yorick(_) => {
                // X1 more every 23 cards discarded since this Yorick was acquired.
                let id = self.instance_id();
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
                    if let Some(state) = g.joker_state_mut(id) {
                        state.counter += hand.all.len() as f32;
                    }
                };
                vec![
                    Effects::OnScore(Arc::new(Mutex::new(played))),
                    Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
                ]
            }
            Self::Castle(castle) => {
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
                        .filter(|c| c.matches_suit(suit))
                        .count();
                    if let Some(state) = g.joker_state_mut(id) {
                        state.counter += 3.0 * count as f32;
                    }
                };
                vec![
                    Effects::OnScore(Arc::new(Mutex::new(played))),
                    Effects::OnDiscard(Arc::new(Mutex::new(discarded))),
                ]
            }
            Self::MailInRebate(mail) => {
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
                    let count = hand.all.iter().filter(|c| c.value == value).count();
                    g.money += count * 5;
                };
                vec![Effects::OnDiscard(Arc::new(Mutex::new(apply)))]
            }
            Self::Luchador(_) => {
                fn apply(g: &mut Game) {
                    g.boss_disabled_by_luchador = true;
                }
                vec![Effects::OnSell(Arc::new(Mutex::new(apply)))]
            }
            Self::Matador(_) => {
                fn apply(g: &mut Game, _hand: MadeHand) {
                    if g.boss_triggered_this_hand {
                        g.money += 8;
                    }
                }
                vec![Effects::OnScore(Arc::new(Mutex::new(apply)))]
            }
            #[cfg(feature = "ante-extra-jokers")]
            Self::Misprint(_) => vec![Effects::OnScore(Arc::new(Mutex::new(
                |g: &mut Game, _: MadeHand| g.mult += g.roll_misprint_mult() as f64,
            )))],
            #[cfg(feature = "ante-extra-jokers")]
            Self::GoldenJoker(_) => {
                vec![Effects::OnRoundEnd(Arc::new(Mutex::new(|g: &mut Game| {
                    g.money += 4
                })))]
            }
            #[cfg(feature = "ante-extra-jokers")]
            Self::IceCream(_) => {
                let remaining = 100_usize.saturating_sub(5 * self.state().counter as usize);
                vec![Effects::OnScore(Arc::new(Mutex::new(
                    move |g: &mut Game, _: MadeHand| g.chips += remaining,
                )))]
            }
            _ => vec![],
        }
    }

    fn is_implemented(&self) -> bool {
        matches!(
            self,
            Self::TheJoker(_)
                | Self::GreedyJoker(_)
                | Self::LustyJoker(_)
                | Self::WrathfulJoker(_)
                | Self::GluttonousJoker(_)
                | Self::JollyJoker(_)
                | Self::ZanyJoker(_)
                | Self::MadJoker(_)
                | Self::CrazyJoker(_)
                | Self::DrollJoker(_)
                | Self::SlyJoker(_)
                | Self::WilyJoker(_)
                | Self::CleverJoker(_)
                | Self::DeviousJoker(_)
                | Self::CraftyJoker(_)
                | Self::HalfJoker(_)
                | Self::JokerStencil(_)
                | Self::Banner(_)
                | Self::MysticSummit(_)
                | Self::Supernova(_)
                | Self::RideTheBus(_)
                | Self::CardSharp(_)
                | Self::Obelisk(_)
                | Self::WeeJoker(_)
                | Self::SpareTrousers(_)
                | Self::Fibonacci(_)
                | Self::ScaryFace(_)
                | Self::AbstractJoker(_)
                | Self::Pareidolia(_)
                | Self::EvenSteven(_)
                | Self::OddTodd(_)
                | Self::Scholar(_)
                | Self::BusinessCard(_)
                | Self::FacelessJoker(_)
                | Self::Baron(_)
                | Self::MidasMask(_)
                | Self::Photograph(_)
                | Self::ReservedParking(_)
                | Self::BaseballCard(_)
                | Self::Bull(_)
                | Self::WalkieTalkie(_)
                | Self::SmileyFace(_)
                | Self::GoldenTicket(_)
                | Self::Acrobat(_)
                | Self::RoughGem(_)
                | Self::Bloodstone(_)
                | Self::Arrowhead(_)
                | Self::OnyxAgate(_)
                | Self::ShootTheMoon(_)
                | Self::RaisedFist(_)
                | Self::TheDuo(_)
                | Self::TheTrio(_)
                | Self::TheFamily(_)
                | Self::TheOrder(_)
                | Self::TheTribe(_)
                | Self::StoneJoker(_)
                | Self::SteelJoker(_)
                | Self::BlueJoker(_)
                | Self::Erosion(_)
                | Self::DriversLicense(_)
                | Self::Mime(_)
                | Self::Dusk(_)
                | Self::Hack(_)
                | Self::SockAndBuskin(_)
                | Self::HangingChad(_)
                | Self::TradingCard(_)
                | Self::GreenJoker(_)
                | Self::HitTheRoad(_)
                | Self::Yorick(_)
                | Self::Castle(_)
                | Self::MailInRebate(_)
                | Self::Luchador(_)
                | Self::Chicot(_)
                | Self::Matador(_)
        ) || cfg!(feature = "ante-extra-jokers")
            && matches!(
                self,
                Self::Misprint(_) | Self::GoldenJoker(_) | Self::IceCream(_)
            )
    }
}

pub fn joker_catalog() -> impl Iterator<Item = Jokers> {
    Jokers::iter()
}

/// Only returns jokers with real `effects()` behavior implemented - callers
/// (shop/pack generation) must never offer a not-yet-implemented joker to
/// the player. A free function rather than an inherent method since
/// `Jokers` is now a foreign type (orphan rule blocks inherent impls on it).
pub(crate) fn jokers_by_rarity(rarity: Rarity) -> Vec<Jokers> {
    Jokers::iter()
        .filter(|j| j.rarity() == rarity && j.is_implemented())
        .collect()
}

/// `Display` can't be implemented on `Jokers` directly (foreign trait +
/// foreign type both fail the orphan rule), so this free function stands in
/// for the old `impl fmt::Display for Jokers`.
pub fn joker_display(j: &Jokers) -> String {
    let edition = j.edition();
    let edition_str = if edition == Edition::Base {
        String::new()
    } else {
        format!(" [{:?}]", edition)
    };
    let stickers = j.stickers();
    let mut sticker_flags = Vec::new();
    if stickers.eternal {
        sticker_flags.push("Eternal");
    }
    if stickers.perishable {
        sticker_flags.push("Perishable");
    }
    if stickers.rental {
        sticker_flags.push("Rental");
    }
    let sticker_str = if sticker_flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", sticker_flags.join(","))
    };
    format!(
        "{} [{}]{}{} {}",
        j.name(),
        j.rarity(),
        edition_str,
        sticker_str,
        j.desc()
    )
}

#[cfg(test)]
mod tests;
