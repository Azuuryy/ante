//! Whole runs against a chosen Boss Blind: the Small and Big blinds are cleared through
//! real plays, then each boss round is played hand by hand with known cards and Jokers,
//! and the run moves on to the next ante. They check the boss rules, the Jokers that
//! react to them and the hands they concern, end to end through `handle_action`.

use super::*;
use crate::joker::{JollyJoker, Matador};

/// A run driven through the engine's actions.
struct Run {
    g: Game,
}

impl Run {
    fn new(jokers: Vec<Jokers>) -> Self {
        let mut g = Game::default();
        g.start();
        g.jokers = jokers;
        Self { g }
    }

    /// Clears the Small and Big blinds of the current ante with a real play, as the client
    /// does once the score is reached, and stops before the Boss Blind.
    fn clear_small_and_big(&mut self) {
        for blind in [Blind::Small, Blind::Big] {
            self.g
                .handle_action(Action::SelectBlind(blind))
                .expect("select");
            self.g.score = self.g.required_score();
            self.select(&[0]);
            self.g.handle_action(Action::Play()).expect("play");
            self.leave_shop();
        }
    }

    fn leave_shop(&mut self) {
        assert_eq!(self.g.stage, Stage::PostBlind());
        self.g.handle_action(Action::CashOut(0)).expect("cash out");
        self.g
            .handle_action(Action::NextRound())
            .expect("next round");
    }

    /// Starts the Boss Blind of the current ante against `boss`.
    fn start_boss(&mut self, boss: BossBlind) {
        self.g.current_boss = Some(boss);
        self.g
            .handle_action(Action::SelectBlind(Blind::Boss))
            .expect("select boss");
        assert_eq!(self.g.active_boss(), Some(boss));
    }

    /// Replaces the hand with `cards`.
    fn deal(&mut self, cards: Vec<Card>) {
        self.g.available.empty();
        self.g.available.extend(cards);
    }

    fn select(&mut self, indices: &[usize]) {
        for &index in indices {
            let card = self.g.available.cards()[index];
            self.g
                .handle_action(Action::SelectCard(card))
                .expect("select card");
        }
    }

    /// Plays the cards at `indices`; returns the round score added and the trace.
    fn play(&mut self, indices: &[usize]) -> (usize, ScoreTrace) {
        self.select(indices);
        let before = self.g.score;
        let (trace, _) = self
            .g
            .handle_action_traced(Action::Play(), false)
            .expect("play");
        // A play that clears the blind resets the round score.
        (self.g.score.saturating_sub(before), trace)
    }

    /// Ends the boss round with a winning play.
    fn win_round(&mut self) {
        self.g.score = self.g.required_score();
        self.play(&[0]);
        self.leave_shop();
    }
}

fn cards(spec: &[(Value, Suit)]) -> Vec<Card> {
    spec.iter()
        .map(|&(value, suit)| Card::new(value, suit))
        .collect()
}

fn heart_flush() -> Vec<Card> {
    use Value::*;
    cards(&[
        (Two, Suit::Heart),
        (Five, Suit::Heart),
        (Seven, Suit::Heart),
        (Nine, Suit::Heart),
        (King, Suit::Heart),
        (Three, Suit::Club),
        (Four, Suit::Spade),
        (Six, Suit::Diamond),
    ])
}

fn pair_hand() -> Vec<Card> {
    use Value::*;
    cards(&[
        (Queen, Suit::Heart),
        (Queen, Suit::Club),
        (Two, Suit::Spade),
        (Five, Suit::Diamond),
        (Eight, Suit::Club),
        (Nine, Suit::Heart),
        (Jack, Suit::Spade),
        (Three, Suit::Diamond),
    ])
}

#[test]
fn the_psychic_then_the_ox_over_two_antes_with_matador() {
    let mut run = Run::new(vec![Jokers::Matador(Matador::default())]);
    run.clear_small_and_big();
    run.start_boss(BossBlind::Psychic);
    run.g.money = 4;

    // One card against The Psychic: "Not Allowed!", no score, Matador pays.
    run.deal(heart_flush());
    let (score, trace) = run.play(&[0]);
    assert_eq!(score, 0);
    assert_eq!(trace.0[0].source, ScoreSource::NotAllowed);
    assert_eq!(run.g.money, 12);

    // Five cards: a Flush scores and the boss does not trigger.
    run.deal(heart_flush());
    let (score, trace) = run.play(&[0, 1, 2, 3, 4]);
    assert!(score > 0);
    assert!(!trace
        .0
        .iter()
        .any(|step| step.source == ScoreSource::NotAllowed));
    assert_eq!(run.g.money, 12);

    run.deal(heart_flush());
    run.play(&[0, 1, 2, 3, 4]);
    run.win_round();
    // The whole run counts: the one-card plays of the Small and Big blinds, the refused
    // card and the winning card make High Card (4 plays) the most played, ahead of the
    // two Flushes.
    assert_eq!(run.g.ox_hand, Some(HandRank::HighCard));
    assert_eq!(run.g.ante_current, crate::ante::Ante::Two);

    run.clear_small_and_big();
    run.start_boss(BossBlind::Ox);
    run.g.money = 20;
    run.deal(pair_hand());
    run.play(&[0, 1]);
    assert_eq!(run.g.money, 20, "a Pair is not the most played hand");
    run.deal(pair_hand());
    run.play(&[3]);
    assert_eq!(
        run.g.money, 8,
        "the Ox empties the wallet, then Matador pays"
    );
}

#[test]
fn the_eye_and_the_mouth_refuse_hands_and_skip_the_jokers() {
    let mut run = Run::new(vec![Jokers::JollyJoker(JollyJoker::default())]);
    run.clear_small_and_big();
    run.start_boss(BossBlind::Eye);
    run.deal(pair_hand());
    let (first_pair, _) = run.play(&[0, 1]);
    // Pair level (10 + 2) plus the two Queens, with Jolly Joker's +8 Mult on a Pair.
    assert_eq!(first_pair, (10 + 10 + 10) * (2 + 8));
    run.deal(pair_hand());
    let (second_pair, trace) = run.play(&[0, 1]);
    assert_eq!(second_pair, 0, "The Eye refuses a hand type already played");
    assert_eq!(trace.0.len(), 1, "no Joker acts on a refused hand");
    run.deal(pair_hand());
    let (high_card, _) = run.play(&[3]);
    assert!(high_card > 0);
    run.win_round();

    run.clear_small_and_big();
    run.start_boss(BossBlind::Mouth);
    run.deal(pair_hand());
    assert!(
        run.play(&[3]).0 > 0,
        "the first hand sets the only allowed type"
    );
    run.deal(pair_hand());
    assert_eq!(
        run.play(&[0, 1]).0,
        0,
        "a Pair after a High Card is refused"
    );
    run.deal(pair_hand());
    assert!(run.play(&[4]).0 > 0);
}

#[test]
fn the_hook_discards_held_purple_seals_and_pays_matador() {
    let mut run = Run::new(vec![Jokers::Matador(Matador::default())]);
    run.clear_small_and_big();
    run.start_boss(BossBlind::Hook);
    run.g.money = 0;
    run.g.consumables.clear();
    let mut hand = pair_hand();
    for card in hand.iter_mut().skip(2) {
        card.seal = Some(Seal::Purple);
    }
    run.deal(hand);
    let discards = run.g.discards();
    run.play(&[0, 1]);
    assert_eq!(
        run.g.discarded_this_round.len(),
        2,
        "two held cards are hooked"
    );
    assert_eq!(run.g.discards(), discards, "the Hook spends no discard");
    assert_eq!(
        run.g.consumables.len(),
        2,
        "each hooked Purple seal makes a Tarot"
    );
    assert_eq!(run.g.money, 8);
    assert_eq!(run.g.available.cards().len(), run.g.hand_size());
}

#[test]
fn the_tooth_charges_each_card_played() {
    let mut run = Run::new(vec![Jokers::Matador(Matador::default())]);
    run.clear_small_and_big();
    run.start_boss(BossBlind::Tooth);
    run.g.money = 10;
    run.deal(heart_flush());
    run.play(&[0, 1, 2, 3, 4]);
    assert_eq!(run.g.money, 10 - 5 + 8);
    run.deal(pair_hand());
    run.g.jokers.clear();
    run.play(&[0, 1]);
    assert_eq!(run.g.money, 11);
    run.win_round();
    assert_eq!(run.g.stage, Stage::PreBlind());
}
