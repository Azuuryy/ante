//! The unlock condition of each locked item, ported from `check_for_unlock` and the
//! `unlock_condition` of `game.lua`. A condition is checked against what happened in the
//! last command ([`Moment`]), the state after it, and the profile's progress.

use ante_core::contract::{CardView, HandKind, RunView, SUITS};

use crate::services::profile::Progress;

/// What the last command did that a condition may look at.
#[derive(Default)]
pub(crate) struct Moment<'a> {
    /// The cards of a hand just played.
    pub(crate) played: Option<&'a [&'a CardView]>,
    /// The cards just discarded.
    pub(crate) discarded: Option<&'a [&'a CardView]>,
    /// The score of the hand just played.
    pub(crate) hand_score: Option<usize>,
    /// A blind just won: whether it was a Boss Blind, in one hand without a discard, and
    /// the last hand's type.
    pub(crate) round_won: Option<RoundWon>,
    /// The run was just won; the most Jokers held during it.
    pub(crate) run_won: Option<usize>,
    /// The ante just reached.
    pub(crate) ante_reached: Option<usize>,
    /// The player continued the run (Ante: endless mode; it has no saved runs).
    pub(crate) continued: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct RoundWon {
    pub(crate) boss: bool,
    pub(crate) one_hand_no_discard: bool,
    pub(crate) last_hand: Option<HandKind>,
}

/// What the profile has discovered, by kind (`G.DISCOVER_TALLIES`).
#[derive(Default, Clone, Copy)]
pub(crate) struct Discoveries {
    pub(crate) total: usize,
    pub(crate) tarots: usize,
    pub(crate) planets: usize,
    pub(crate) blinds: usize,
}

/// Whether locked item `id` meets its condition now.
pub(crate) fn met(
    id: &str,
    moment: &Moment,
    view: &RunView,
    progress: &Progress,
    found: Discoveries,
) -> bool {
    let stat = |name: &str, at_least: u64| progress.stat(name) >= at_least;
    let played = |test: &dyn Fn(&CardView) -> bool| {
        moment
            .played
            .map_or(0, |cards| cards.iter().filter(|c| test(c)).count())
    };
    let deck_count = |test: &dyn Fn(usize, Option<&str>) -> bool| {
        view.deck_view
            .iter()
            .filter(|card| test(card.suit, card.enhancement))
            .count()
    };
    let suit_count = |suit: &str| deck_count(&|s, _| SUITS[s] == suit);
    let enhanced = |name: &str| deck_count(&|_, e| e == Some(name));
    let won = moment.run_won;
    let never_played = |kind: HandKind| {
        won.is_some()
            && view
                .hand_levels
                .iter()
                .any(|hand| hand.rank == kind && hand.level.plays == 0)
    };
    match id {
        // Career statistics.
        "j_mr_bones" => stat("c_losses", 5),
        "j_acrobat" => stat("c_hands_played", 200),
        "j_sock_and_buskin" => stat("c_face_cards_played", 300),
        "j_swashbuckler" => stat("c_jokers_sold", 20),
        "j_burnt" => stat("c_cards_sold", 50),
        "j_troubadour" => stat("c_single_hand_round_streak", 5),
        "v_overstock_plus" => stat("c_shop_dollars_spent", 2500),
        "v_reroll_glut" => stat("c_shop_rerolls", 100),
        "v_omen_globe" => stat("c_tarot_reading_used", 25),
        "v_observatory" => stat("c_planetarium_used", 25),
        "v_nacho_tong" => stat("c_cards_played", 2500),
        "v_recyclomancy" => stat("c_cards_discarded", 2500),
        "v_tarot_tycoon" => stat("c_tarots_bought", 50),
        "v_planet_tycoon" => stat("c_planets_bought", 50),
        "v_money_tree" => stat("c_round_interest_cap_streak", 10),
        "v_antimatter" => stat("v_blank_redeemed", 10),
        "v_illusion" => stat("c_playing_cards_bought", 20),
        // The hand played or discarded.
        "j_ticket" => played(&|c| c.enhancement == "Gold") >= 5,
        "j_seeing_double" => {
            played(&|c| c.rank == '7' && (c.suit == "Clubs" || c.enhancement == "Wild")) >= 4
        }
        "j_oops" => moment.hand_score.is_some_and(|score| score >= 10_000),
        "j_idol" => moment.hand_score.is_some_and(|score| score >= 1_000_000),
        "j_stuntman" => moment.hand_score.is_some_and(|score| score >= 100_000_000),
        "j_hit_the_road" => moment
            .discarded
            .is_some_and(|cards| cards.iter().filter(|c| c.rank == 'J').count() >= 5),
        "j_brainstorm" => moment.discarded.is_some_and(royal_flush),
        "j_shoot_the_moon" => moment.played.is_some() && no_hearts_left(view),
        // Rounds and runs.
        "j_matador" => moment
            .round_won
            .is_some_and(|round| round.boss && round.one_hand_no_discard),
        "j_hanging_chad" => moment
            .round_won
            .is_some_and(|round| round.boss && round.last_hand == Some(HandKind::HighCard)),
        "j_wee" => won.is_some() && view.round <= 18,
        "j_merry_andy" => won.is_some() && view.round <= 12,
        "j_blueprint" => won.is_some(),
        "j_invisible" => won.is_some_and(|most| most <= 4),
        "j_duo" => never_played(HandKind::OnePair),
        "j_trio" => never_played(HandKind::ThreeOfAKind),
        "j_family" => never_played(HandKind::FourOfAKind),
        "j_order" => never_played(HandKind::Straight),
        "j_tribe" => never_played(HandKind::Flush),
        "j_ring_master" => moment.ante_reached == Some(4),
        "j_flower_pot" => moment.ante_reached == Some(8),
        "v_petroglyph" => moment.ante_reached == Some(12),
        "j_throwback" => moment.continued,
        "j_satellite" => !view.in_debt && view.money >= 400,
        // The deck and the Jokers held.
        "j_rough_gem" => suit_count("Diamonds") >= 30,
        "j_bloodstone" => suit_count("Hearts") >= 30,
        "j_arrowhead" => suit_count("Spades") >= 30,
        "j_onyx_agate" => suit_count("Clubs") >= 30,
        "j_smeared" => enhanced("Wild") >= 3,
        "j_glass" => enhanced("Glass") >= 5,
        "j_drivers_license" => deck_count(&|_, e| e.is_some()) >= 16,
        "j_certificate" => view
            .deck_view
            .iter()
            .any(|card| card.enhancement == Some("Gold") && card.seal == Some("Gold")),
        "j_bootstraps" => {
            view.held_jokers
                .iter()
                .filter(|j| j.edition == "Polychrome")
                .count()
                >= 2
        }
        "v_glow_up" => {
            view.held_jokers
                .iter()
                .filter(|j| !j.edition.is_empty() && j.edition != "Base")
                .count()
                >= 5
        }
        "v_liquidation" => view.vouchers.len() >= 10,
        "v_palette" => view.hand_size > 0 && view.hand_size <= 5,
        // Discoveries.
        "j_cartomancer" => found.tarots >= 22,
        "j_astronomer" => found.planets >= 12,
        "v_retcon" => found.blinds >= 25,
        "b_blue" => found.total >= 20,
        "b_yellow" => found.total >= 50,
        "b_green" => found.total >= 75,
        "b_black" => found.total >= 100,
        // Wins with decks and stakes.
        "b_magic" => progress.deck_win_stake("b_red") > 0,
        "b_nebula" => progress.deck_win_stake("b_blue") > 0,
        "b_ghost" => progress.deck_win_stake("b_yellow") > 0,
        "b_abandoned" => progress.deck_win_stake("b_green") > 0,
        "b_checkered" => progress.deck_win_stake("b_black") > 0,
        "b_zodiac" => progress.best_win_stake() >= 2,
        "b_painted" => progress.best_win_stake() >= 3,
        "b_anaglyph" => progress.best_win_stake() >= 4,
        "b_plasma" => progress.best_win_stake() >= 5,
        "b_erratic" => progress.best_win_stake() >= 7,
        _ => false,
    }
}

/// `discard_custom` for Brainstorm: a Straight Flush whose lowest card is a 10.
fn royal_flush(cards: &[&CardView]) -> bool {
    if cards.len() != 5 {
        return false;
    }
    let suit = cards[0].suit;
    let same_suit = cards
        .iter()
        .all(|c| c.suit == suit || c.enhancement == "Wild");
    let mut ranks: Vec<char> = cards.iter().map(|c| c.rank).collect();
    ranks.sort_unstable();
    let mut royal = ['A', 'J', 'K', 'Q', 'T'];
    royal.sort_unstable();
    same_suit && ranks == royal
}

/// `play_all_hearts`: no Heart left in the draw pile or the hand (Stone cards aside).
fn no_hearts_left(view: &RunView) -> bool {
    let in_pile = view.deck_view.iter().any(|card| {
        card.remaining && SUITS[card.suit] == "Hearts" && card.enhancement != Some("Stone")
    });
    let in_hand = view
        .cards
        .iter()
        .any(|card| card.suit == "Hearts" && card.enhancement != "Stone");
    !in_pile && !in_hand
}
