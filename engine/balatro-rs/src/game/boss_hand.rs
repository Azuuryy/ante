//! `Blind:debuff_hand`: bosses that refuse a played hand. The Psychic refuses a hand of
//! fewer than 5 cards, The Eye a hand type already played this round, and The Mouth any
//! hand type other than the round's first. A refused hand ("Not Allowed!") scores
//! nothing; only the Jokers of `context.debuffed_hand` act on it, which is Matador's $8
//! when the boss's ability triggered.

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
}
