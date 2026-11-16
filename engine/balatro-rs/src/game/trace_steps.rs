//! The score trace: each step of a scoring pass as `calc_score_traced` records it.

use super::*;

impl Game {
    // Multiplicative steps note their factor so the trace can tell XMult from +Mult.
    pub(crate) fn note_xmult(&mut self, factor: f64) {
        self.trace_xmult = Some(self.trace_xmult.unwrap_or(1.0) * factor);
    }

    pub(super) fn record_step(
        &mut self,
        trace: &mut Option<&mut TraceRecorder>,
        source: ScoreSource,
        chips_before: usize,
        mult_before: f64,
        retrigger: bool,
    ) {
        let xmult = self.trace_xmult.take();
        let Some(t) = trace else {
            return;
        };
        let money_before = t.money.replace(self.money).unwrap_or(self.money);
        let money = self.money as i64 - money_before as i64;
        // skip no-op checks (e.g. a Base-edition joker, a conditional
        // effect that didn't fire); a refused hand's step changes nothing but is shown.
        if chips_before == self.chips
            && mult_before == self.mult
            && money == 0
            && !matches!(
                source,
                ScoreSource::NotAllowed | ScoreSource::JokerUpgrade(_)
            )
        {
            return;
        }
        let xmult = xmult.filter(|_| mult_before != self.mult);
        let start = t.profile.then(std::time::Instant::now);
        t.trace.0.push(ScoreStep {
            source,
            chips_before,
            chips_after: self.chips,
            mult_before,
            mult_after: self.mult,
            retrigger,
            xmult,
            money,
        });
        if let Some(start) = start {
            t.elapsed += start.elapsed();
        }
    }

    /// The value a Joker carries (`JokerState.counter`), read again after its effect.
    pub(super) fn joker_counter(&self, joker: &Jokers) -> f32 {
        self.jokers
            .iter()
            .find(|held| held.instance_id() == joker.instance_id())
            .map_or(0.0, |held| held.state().counter)
    }

    /// A Joker upgrade in the trace. It changes no value and leaves the pending XMult
    /// note to the step that follows.
    pub(super) fn record_upgrade(
        &mut self,
        trace: &mut Option<&mut TraceRecorder>,
        joker: &Jokers,
    ) {
        let Some(t) = trace else {
            return;
        };
        t.trace.0.push(ScoreStep {
            source: ScoreSource::JokerUpgrade(joker.clone()),
            chips_before: self.chips,
            chips_after: self.chips,
            mult_before: self.mult,
            mult_after: self.mult,
            retrigger: false,
            xmult: None,
            money: 0,
        });
    }
}
