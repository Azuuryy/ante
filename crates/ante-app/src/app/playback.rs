use std::time::Duration;

use ante_core::contract::{CardKey, ScoreSource, ScoreStep, ScoreTrace};

/// Scoring delays in game seconds, taken from the original event sequence. The game
/// clock runs `speed` times faster than real time, so 1x lasts as long as the original.
pub mod timing {
    /// `play_cards_from_highlighted`: pause after spending the hand.
    pub const PLAY_START: f32 = 0.4;
    /// `draw_card` hand to play area, per card.
    pub const MOVE_PER_CARD: f32 = 0.1;
    /// `evaluate_play`: pause before highlighting the scoring cards.
    pub const EVALUATE_START: f32 = 0.2;
    /// `highlight_card` per scoring card, up and down.
    pub const HIGHLIGHT_PER_CARD: f32 = 0.1;
    /// Pause before the first scoring popup when the hand is not debuffed.
    pub const BEFORE_SCORING: f32 = 0.4;
    /// `card_eval_status_text` blocks for its base delay times 1.25.
    pub const CHIPS_POPUP: f32 = 0.6 * 1.25;
    pub const MULT_POPUP: f32 = 0.65 * 1.25;
    pub const JOKER_POPUP: f32 = 0.75 * 1.25;
    /// Money uses the default status delay.
    pub const MONEY_POPUP: f32 = 0.65 * 1.25;
    /// Total shown, then the chip count, the score ease and the final pause.
    pub const TOTAL_SHOWN: f32 = 0.4;
    pub const TOTAL_HOLD: f32 = 0.8;
    pub const SCORE_EASE: f32 = 0.5;
    pub const AFTER_SCORE: f32 = 0.3;
    /// `draw_from_play_to_discard`, per card.
    pub const DISCARD_PER_CARD: f32 = 0.1;

    /// Game speeds offered in the options, doubling up to 128x.
    pub const SPEEDS: [f32; 9] = [0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0];
    /// Long scoring sequences speed up: the accumulator grows by 0.2 x speed per second
    /// up to 16 and adds `acc - 2` to the clock rate once above 2.
    pub const ACCELERATION_RATE: f32 = 0.2;
    pub const ACCELERATION_CAP: f32 = 16.0;
    pub const ACCELERATION_FLOOR: f32 = 2.0;
}

use timing::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speed(u8);

impl Default for Speed {
    fn default() -> Self {
        Self(1)
    }
}

impl Speed {
    pub const ALL: [Self; SPEEDS.len()] = {
        let mut all = [Self(0); SPEEDS.len()];
        let mut index = 0;
        while index < SPEEDS.len() {
            all[index] = Self(index as u8);
            index += 1;
        }
        all
    };

    pub fn factor(self) -> f32 {
        SPEEDS[self.0 as usize]
    }

    pub fn label(self) -> String {
        let factor = self.factor();
        if factor.fract() == 0.0 {
            format!("x{factor:.0}")
        } else {
            format!("x{factor}")
        }
    }

    pub fn next(self) -> Self {
        Self((self.0 + 1) % SPEEDS.len() as u8)
    }

    /// Speed for a saved factor, like the original `GAMESPEED` setting; unknown values are ignored.
    pub fn from_factor(factor: f32) -> Option<Self> {
        SPEEDS
            .iter()
            .position(|&known| known == factor)
            .map(|index| Self(index as u8))
    }
}

/// Phase of the scoring sequence, used to place and lift the played cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Cards travel to the play area and the scoring ones lift.
    Intro,
    Scoring,
    /// The total is shown and eased into the round score.
    Total,
    /// Played cards travel to the discard pile.
    Discard,
}

pub struct Playback {
    trace: ScoreTrace,
    /// Start time of each trace step in game seconds.
    starts: Vec<f32>,
    scoring_start: f32,
    scoring_end: f32,
    total_end: f32,
    end: f32,
    clock: f32,
    acceleration: f32,
}

impl Playback {
    pub fn new(trace: ScoreTrace, played: usize) -> Self {
        let played = played as f32;
        let scoring_start = PLAY_START
            + MOVE_PER_CARD * played
            + EVALUATE_START
            + HIGHLIGHT_PER_CARD * played
            + BEFORE_SCORING;
        let mut starts = Vec::with_capacity(trace.0.len());
        let mut clock = scoring_start;
        let mut previous: Option<&ScoreStep> = None;
        for step in &trace.0 {
            starts.push(clock);
            clock += step_duration(step, previous);
            previous = Some(step);
        }
        let scoring_end = clock;
        let total_end =
            scoring_end + HIGHLIGHT_PER_CARD * played + TOTAL_SHOWN + TOTAL_HOLD + SCORE_EASE;
        let end = total_end + AFTER_SCORE + DISCARD_PER_CARD * played;
        Self {
            trace,
            starts,
            scoring_start,
            scoring_end,
            total_end,
            end,
            clock: 0.0,
            acceleration: 0.0,
        }
    }

    /// Advances the game clock; returns true once the whole sequence has played.
    pub fn advance(&mut self, delta: Duration, speed: Speed) -> bool {
        let real = delta.as_secs_f32();
        let factor = speed.factor();
        self.acceleration =
            (self.acceleration + real * ACCELERATION_RATE * factor).min(ACCELERATION_CAP);
        let rate = factor + (self.acceleration - ACCELERATION_FLOOR).max(0.0);
        self.clock = (self.clock + real * rate).min(self.end);
        self.clock >= self.end
    }

    /// Length of the whole sequence in game seconds.
    pub fn duration(&self) -> f32 {
        self.end
    }

    pub fn stage(&self) -> Stage {
        if self.clock < self.scoring_start {
            Stage::Intro
        } else if self.clock < self.scoring_end {
            Stage::Scoring
        } else if self.clock < self.total_end {
            Stage::Total
        } else {
            Stage::Discard
        }
    }

    /// Number of trace steps already revealed.
    fn cursor(&self) -> usize {
        self.starts.partition_point(|start| *start <= self.clock)
    }

    fn current(&self) -> Option<&ScoreStep> {
        self.trace.0.get(self.cursor().checked_sub(1)?)
    }

    /// Played cards that score; they stay lifted while the hand is evaluated.
    pub fn scoring_cards(&self) -> Vec<CardKey> {
        let mut cards = Vec::new();
        for step in &self.trace.0 {
            if let ScoreSource::Card {
                trigger: ante_core::contract::CardTrigger::Played,
                id,
                ..
            } = step.source
                && !cards.contains(&id)
            {
                cards.push(id);
            }
        }
        cards
    }

    /// Index and content of the step whose popup is showing.
    pub fn current_step(&self) -> Option<(usize, &ScoreStep)> {
        if self.stage() != Stage::Scoring {
            return None;
        }
        let index = self.cursor().checked_sub(1)?;
        Some((index, &self.trace.0[index]))
    }

    pub fn focused_card(&self) -> Option<CardKey> {
        if self.stage() != Stage::Scoring {
            return None;
        }
        match self.current()?.source {
            ScoreSource::Card { id, .. } => Some(id),
            _ => None,
        }
    }

    pub fn score_values(&self) -> Option<(usize, f64)> {
        let step = self.current()?;
        Some((step.chips_after, step.mult_after))
    }

    /// Progress through the current popup, 0 when it appears and 1 when the next starts.
    /// Step index, popup index within the step and the popup currently shown.
    pub fn current_popup(&self) -> Option<(usize, usize, Popup)> {
        let (index, step) = self.current_step()?;
        let previous = index
            .checked_sub(1)
            .and_then(|previous| self.trace.0.get(previous));
        let mut end = self.starts[index];
        let popups = popups(step, previous);
        for (position, &(popup, delay)) in popups.iter().enumerate() {
            end += delay;
            if self.clock < end || position + 1 == popups.len() {
                return Some((index, position, popup));
            }
        }
        None
    }

    pub fn step_progress(&self) -> f32 {
        let cursor = self.cursor();
        let Some(index) = cursor.checked_sub(1) else {
            return 1.0;
        };
        let start = self.starts[index];
        let end = self.starts.get(cursor).copied().unwrap_or(self.scoring_end);
        if end <= start {
            return 1.0;
        }
        ((self.clock - start) / (end - start)).clamp(0.0, 1.0)
    }

    pub fn description(&self) -> String {
        let Some(step) = self.current() else {
            return "Scoring...".into();
        };
        let source = match &step.source {
            ScoreSource::Card { rank, suit, .. } => format!("{rank} {suit}"),
            ScoreSource::Joker { name, .. } | ScoreSource::Voucher(_, name) => (*name).to_owned(),
            ScoreSource::HandLevel(rank) => format!("{rank:?}"),
        };
        let effect = if step.retrigger {
            format!("{source} retrigger")
        } else {
            source
        };
        format!(
            "{effect}   |   {} chips x {} mult   |   {}/{}",
            step.chips_after,
            step.mult_after,
            self.cursor(),
            self.trace.0.len()
        )
    }
}

/// One status popup of a scoring step, shown one after the other like the original
/// `card_eval_status_text` calls: "Again!", then chips, mult, xmult and money.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Popup {
    Again,
    Chips(i64),
    Mult(i64),
    /// Factor in hundredths: 150 is X1.5 Mult.
    XMult(usize),
    Money(i64),
}

/// Popups of a step and their delays in game seconds. Cards use the chips and mult
/// delays; Jokers and vouchers use the longer Joker delay for each popup.
pub fn popups(step: &ScoreStep, previous: Option<&ScoreStep>) -> Vec<(Popup, f32)> {
    let joker = match step.source {
        ScoreSource::HandLevel(_) => return Vec::new(),
        ScoreSource::Joker { .. } | ScoreSource::Voucher(..) => true,
        ScoreSource::Card { .. } => false,
    };
    let delay = |card: f32| if joker { JOKER_POPUP } else { card };
    let mut popups = Vec::new();
    if step.retrigger
        && !previous.is_some_and(|previous| previous.retrigger && previous.source == step.source)
    {
        popups.push((Popup::Again, JOKER_POPUP));
    }
    let chips = step.chips_after as i64 - step.chips_before as i64;
    if chips != 0 {
        popups.push((Popup::Chips(chips), delay(CHIPS_POPUP)));
    }
    // An additive Mult applies before a multiplicative one, as with a Mult card in Polychrome.
    let added_mult = match step.xmult {
        Some(factor) if factor > 0 => {
            (step.mult_after * 100.0 / factor as f64 - step.mult_before).round() as i64
        }
        _ => (step.mult_after - step.mult_before).round() as i64,
    };
    if added_mult != 0 {
        popups.push((Popup::Mult(added_mult), delay(MULT_POPUP)));
    }
    if let Some(factor) = step.xmult {
        popups.push((Popup::XMult(factor), delay(MULT_POPUP)));
    }
    if step.money != 0 {
        popups.push((Popup::Money(step.money), delay(MONEY_POPUP)));
    }
    popups
}

fn step_duration(step: &ScoreStep, previous: Option<&ScoreStep>) -> f32 {
    popups(step, previous).iter().map(|(_, delay)| delay).sum()
}

#[cfg(test)]
mod tests;
