//! `Blind:debuff_hand`: bosses that refuse a played hand. The Psychic refuses a hand of
//! fewer than 5 cards, The Eye a hand type already played this round, and The Mouth any
//! hand type other than the round's first. A refused hand ("Not Allowed!") scores
//! nothing; only the Jokers of `context.debuffed_hand` act on it, which is Matador's $8
//! when the boss's ability triggered.
//!
//! The same file holds the other hand-time boss abilities: The Ox (`debuff_hand`) and
//! The Hook and The Tooth (`Blind:press_play`).

use super::*;

impl Game {
    /// Whether the active boss refuses `hand`, as `Blind:debuff_hand` decides it.
    pub(super) fn hand_not_allowed(&self, hand: &MadeHand) -> bool {
        match self.active_boss() {
            Some(BossBlind::Psychic) => hand.all.len() < 5,
            Some(BossBlind::Eye) => self.hand_ranks_played_this_round.contains(&hand.rank),
            Some(BossBlind::Mouth) => self.mouth_hand.is_some_and(|rank| rank != hand.rank),
            _ => false,
        }
    }

    /// The Mouth keeps the first hand type played in the round (`self.only_hand`).
    pub(super) fn remember_mouth_hand(&mut self, hand: &MadeHand) {
        if self.active_boss() == Some(BossBlind::Mouth) && self.mouth_hand.is_none() {
            self.mouth_hand = Some(hand.rank);
        }
    }

    /// Scores a refused hand: it still counts as played (`hands[text].played` and
    /// `played_this_round` rise before the check in `evaluate_play`), the boss's ability
    /// triggered, and each Matador earns $8. The score is 0.
    pub(super) fn score_not_allowed(
        &mut self,
        hand: &MadeHand,
        mut trace: Option<&mut TraceRecorder>,
    ) -> usize {
        self.boss_triggered_this_hand = true;
        self.planetarium.play(hand.rank);
        self.hand_ranks_played_this_round.insert(hand.rank);
        let (chips, mult) = (self.chips, self.mult);
        self.record_step(&mut trace, ScoreSource::NotAllowed, chips, mult, false);
        for joker in self.jokers.clone() {
            if matches!(joker, Jokers::Matador(_)) {
                let (chips_before, mult_before) = (self.chips, self.mult);
                self.money += 8;
                self.record_step(
                    &mut trace,
                    ScoreSource::Joker(joker.clone()),
                    chips_before,
                    mult_before,
                    false,
                );
            }
        }
        0
    }

    /// `Blind:press_play`, before the hand scores. The Hook discards two random cards
    /// held in hand (`pseudorandom_element`), through the same discard as the Discard
    /// action but without spending a discard; The Tooth costs $1 per card played. Both
    /// trigger the boss's ability. The hand is refilled after the play, so the cards
    /// The Hook discarded are replaced like the played ones.
    pub(super) fn press_play(&mut self, hand: &MadeHand) {
        match self.active_boss() {
            Some(BossBlind::Hook) => {
                let mut held = self.available.not_selected();
                let mut hooked = Vec::new();
                for _ in 0..2 {
                    if held.is_empty() {
                        break;
                    }
                    let card = self.backend.pick_random_card(held.clone());
                    held.retain(|c| c.id != card.id);
                    self.available.remove_by_id(card.id);
                    hooked.push(card);
                }
                self.record_discard(hooked);
                self.boss_play_triggered = true;
            }
            Some(BossBlind::Tooth) => {
                // Ante's money cannot go below $0, so The Tooth stops there.
                self.money = self.money.saturating_sub(hand.all.len());
                self.boss_play_triggered = true;
            }
            _ => {}
        }
    }

    /// The Ox: playing the most played hand sets money to $0 (`ease_dollars(-dollars)`).
    pub(super) fn apply_ox(&mut self, hand: &MadeHand) {
        let most_played = self.ox_hand.unwrap_or(HandRank::HighCard);
        if self.active_boss() == Some(BossBlind::Ox)
            && hand.rank.scoring_rank() == most_played.scoring_rank()
        {
            self.money = 0;
            self.boss_triggered_this_hand = true;
        }
    }

    /// After a Boss Blind, `end_round` records the most played hand for The Ox: the most
    /// plays, and on a tie the hand with the lower `order` (the stronger one).
    pub(super) fn remember_most_played_hand(&mut self) {
        use HandRank::*;
        let strongest_first = [
            FlushFive,
            FlushHouse,
            FiveOfAKind,
            StraightFlush,
            FourOfAKind,
            FullHouse,
            Flush,
            Straight,
            ThreeOfAKind,
            TwoPair,
            OnePair,
            HighCard,
        ];
        let mut best = (HighCard, None::<usize>);
        for rank in strongest_first {
            let plays = self.planetarium.level(rank).plays;
            if best.1.is_none_or(|most| plays > most) {
                best = (rank, Some(plays));
            }
        }
        self.ox_hand = Some(best.0);
    }
}
