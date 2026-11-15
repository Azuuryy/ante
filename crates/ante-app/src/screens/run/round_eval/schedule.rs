//! When each part of the round evaluation appears, in game seconds, following the event
//! queue of `add_round_eval_row`: `delay(x)` waits, an event with `trigger = 'after'`
//! waits then runs, and one with `trigger = 'before'` runs then holds the queue.

use ante_core::contract::{RewardKind, RewardRow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reveal {
    /// The left part of row `row` (its icon and text), with the `cancel` sound.
    Row(usize),
    /// The dotted line between the Blind and the bonus rows.
    Divider,
    /// Dollar sign `index` of row `row`, with a `coin3` sound.
    Dollar { row: usize, index: usize },
    /// The single "$N" of a row worth more than $60.
    DollarSum(usize),
    /// The "Cash Out: $N" button.
    CashOut,
}

/// Dollar signs above this many become one "$N" (`num_dollars > 60`).
pub(crate) const DOLLAR_SUM_ABOVE: usize = 60;

/// Seconds between two dollar signs: 0.18, faster for longer rows.
fn dollar_step(dollars: usize) -> f32 {
    0.18 - if dollars > 20 {
        0.13
    } else if dollars > 9 {
        0.1
    } else {
        0.0
    }
}

pub(crate) fn schedule(rows: &[RewardRow]) -> Vec<(f32, Reveal)> {
    let mut at = 0.0;
    let mut reveals = Vec::new();
    let mut divider = false;
    for (row, reward) in rows.iter().enumerate() {
        if reward.kind == RewardKind::Blind {
            at += 0.2;
        } else if !divider {
            at += 0.25;
            reveals.push((at, Reveal::Divider));
            at += 0.6;
            divider = true;
        }
        at += 0.2;
        reveals.push((at, Reveal::Row(row)));
        at += 0.5;
        if reward.dollars > DOLLAR_SUM_ABOVE {
            reveals.push((at, Reveal::DollarSum(row)));
            at += 0.38;
        } else {
            for index in 0..reward.dollars {
                reveals.push((at, Reveal::Dollar { row, index }));
                at += dollar_step(reward.dollars);
            }
        }
    }
    at += 0.4;
    reveals.push((at, Reveal::CashOut));
    reveals
}
