use super::schedule::{Reveal, schedule};
use ante_core::contract::{RewardKind, RewardRow};

fn row(kind: RewardKind, dollars: usize) -> RewardRow {
    RewardRow { kind, dollars }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn rows_and_dollars_follow_the_original_event_queue() {
    let reveals = schedule(&[
        row(RewardKind::Blind, 3),
        row(RewardKind::Hands { count: 2 }, 2),
    ]);
    let at = |what: Reveal| reveals.iter().find(|(_, r)| *r == what).unwrap().0;
    // Blind: delay(0.2) + delay(0.2), then the row holds 0.5 before its dollars.
    assert!(close(at(Reveal::Row(0)), 0.4));
    assert!(close(at(Reveal::Dollar { row: 0, index: 0 }), 0.9));
    assert!(close(at(Reveal::Dollar { row: 0, index: 2 }), 1.26));
    // The divider waits 0.25 after the last dollar's 0.18, then holds 0.6 + 0.2.
    assert!(close(at(Reveal::Divider), 1.44 + 0.25));
    assert!(close(at(Reveal::Row(1)), 1.69 + 0.8));
    // Cash Out comes 0.4 after the last dollar's hold.
    let last = at(Reveal::Dollar { row: 1, index: 1 });
    assert!(close(at(Reveal::CashOut), last + 0.18 + 0.4));
    // Every part appears in order.
    assert!(reveals.windows(2).all(|pair| pair[0].0 <= pair[1].0));
}

#[test]
fn long_rows_count_faster_and_huge_ones_show_one_sum() {
    let reveals = schedule(&[
        row(RewardKind::Blind, 25),
        row(RewardKind::Interest { max: 5 }, 70),
    ]);
    let first = reveals
        .iter()
        .find(|(_, r)| *r == Reveal::Dollar { row: 0, index: 0 })
        .unwrap()
        .0;
    let second = reveals
        .iter()
        .find(|(_, r)| *r == Reveal::Dollar { row: 0, index: 1 })
        .unwrap()
        .0;
    assert!(close(second - first, 0.05));
    assert!(reveals.iter().any(|(_, r)| *r == Reveal::DollarSum(1)));
    assert!(
        !reveals
            .iter()
            .any(|(_, r)| matches!(r, Reveal::Dollar { row: 1, .. }))
    );
}
