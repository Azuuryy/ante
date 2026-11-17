use ante_core::{
    contract::{Command, Phase},
    engine::EngineRun,
};

const SEED: u64 = 0;
fn select(run: &mut EngineRun, positions: &[usize]) {
    let hand = run.view().cards;
    for &position in positions {
        run.apply(Command::ToggleCard(hand[position - 1].id))
            .unwrap();
    }
}

#[cfg(not(feature = "extra-jokers"))]
mod victory {
    use super::*;
    use ante_app::playback::{Playback, Speed};
    use ante_core::contract::{CampaignGoal, Deck, RunConfig, RunView};
    use std::time::Duration;

    // Scripts name cards as rank + suit initial because the run re-sorts the
    // hand after every deal, so positions do not identify cards.
    type Script = ([&'static str; 5], usize);
    const DISCARD: [&str; 1] = ["QC"];
    const SMALL: [Script; 4] = [
        (["6S", "5C", "3D", "AC", "5D"], 40),
        (["4S", "5S", "2S", "4H", "3S"], 36),
        (["7H", "3C", "7C", "AS", "7D"], 153),
        (["KC", "JC", "JH", "2C", "KS"], 120),
    ];
    const BIG: [Script; 2] = [
        (["8H", "8D", "2H", "2C", "2D"], 248),
        (["JC", "JD", "5S", "5C", "5D"], 300),
    ];
    const BOSS: [Script; 3] = [
        (["KC", "QS", "QH", "TS", "TH"], 360),
        (["AC", "JH", "JC", "9D", "7H"], 180),
        (["8D", "6D", "2S", "2H", "2D"], 252),
    ];

    fn select_cards(run: &mut EngineRun, names: &[&str]) {
        let hand = run.view().cards;
        for name in names {
            let card = hand
                .iter()
                .find(|card| format!("{}{}", card.rank, &card.suit[..1]) == *name)
                .unwrap_or_else(|| {
                    let held: Vec<String> = hand
                        .iter()
                        .map(|card| format!("{}{}", card.rank, &card.suit[..1]))
                        .collect();
                    panic!("{name} is not in hand {held:?}")
                });
            run.apply(Command::ToggleCard(card.id)).unwrap();
        }
    }

    fn record(run: &EngineRun, history: &mut Vec<RunView>) {
        let mut view = run.view();
        for card in &mut view.cards {
            card.id = Default::default();
        }
        for offer in &mut view.offers {
            offer.id = Default::default();
        }
        for joker in &mut view.held_jokers {
            joker.id = Default::default();
        }
        history.push(view);
    }

    fn play_blind(
        run: &mut EngineRun,
        history: &mut Vec<RunView>,
        plays: &[Script],
        playback: Option<Speed>,
    ) {
        let target = run.view().target;
        let mut expected_total = 0;
        for (cards, expected_score) in plays {
            select_cards(run, cards);
            assert!(run.view().can_play);
            if let Some(speed) = playback {
                let outcome = run.apply_traced(Command::Play, true).unwrap();
                let mut animation = Playback::new(outcome.trace.unwrap(), cards.len());
                let before = run.snapshot().unwrap();
                let finished =
                    (0..20_000).any(|_| animation.advance(Duration::from_millis(16), speed));
                assert!(finished);
                assert_eq!(run.snapshot().unwrap(), before);
            } else {
                run.apply(Command::Play).unwrap();
            }
            expected_total += expected_score;
            let view = run.view();
            assert_eq!(view.last_hand_score, Some(*expected_score));
            assert_eq!(view.score, expected_total);
            assert_eq!(view.target, target);
            record(run, history);
        }
    }

    fn winning_run(playback: Option<Speed>) -> Vec<RunView> {
        let mut run = EngineRun::with_config(RunConfig {
            seed: SEED,
            deck: Deck::Red,
            goal: CampaignGoal::SingleAnte,
            starting_money: None,
            stake: 1,
        });
        let mut history = Vec::new();
        let start = run.view();
        assert_eq!((start.phase, start.target), (Phase::BeforeBlind, 300));
        assert_eq!(start.blind, "Small Blind");
        run.apply(Command::StartBlind).unwrap();
        record(&run, &mut history);

        let discards = run.view().discards;
        select_cards(&mut run, &DISCARD);
        run.apply(Command::Discard).unwrap();
        assert_eq!(run.view().discards, discards - 1);
        assert!(run.view().cards.iter().all(|card| !card.selected));
        record(&run, &mut history);

        play_blind(&mut run, &mut history, &SMALL, playback);
        assert_eq!((run.view().phase, run.view().score), (Phase::Reward, 349));
        let cleared = run.view();
        run.apply(Command::CashOut).unwrap();
        assert_eq!(run.view().phase, Phase::Shop);
        assert_eq!(run.view().money, cleared.money + cleared.reward);
        record(&run, &mut history);

        run.apply(Command::LeaveShop).unwrap();
        let next = run.view();
        assert_eq!((next.phase, next.target), (Phase::BeforeBlind, 450));
        assert_eq!(next.blind, "Big Blind");
        run.apply(Command::StartBlind).unwrap();
        record(&run, &mut history);
        play_blind(&mut run, &mut history, &BIG, playback);
        assert_eq!((run.view().phase, run.view().score), (Phase::Reward, 548));
        run.apply(Command::CashOut).unwrap();

        let shop = run.view();
        assert_eq!(shop.phase, Phase::Shop);
        let offer = &shop.offers[1];
        assert_eq!(offer.name, "Joker");
        assert!(offer.enabled);
        run.apply(Command::BuyJoker(offer.id)).unwrap();
        assert_eq!(run.view().money, shop.money - offer.price);
        assert_eq!(run.view().jokers, ["Joker"]);
        assert!(!run.view().offers.iter().any(|other| other.id == offer.id));
        record(&run, &mut history);

        run.apply(Command::LeaveShop).unwrap();
        let boss = run.view();
        assert_eq!((boss.phase, boss.target), (Phase::BeforeBlind, 600));
        assert_eq!(boss.blind, "The Manacle");
        run.apply(Command::StartBlind).unwrap();
        record(&run, &mut history);
        play_blind(&mut run, &mut history, &BOSS, playback);
        assert_eq!((run.view().phase, run.view().score), (Phase::Won, 792));
        history
    }

    fn observe_deck(deck: ante_core::contract::Deck) -> Vec<RunView> {
        use ante_core::contract::RunConfig;
        let mut run = EngineRun::with_config(RunConfig {
            seed: SEED,
            deck,
            goal: CampaignGoal::SingleAnte,
            starting_money: None,
            stake: 1,
        });
        let expected = match deck {
            ante_core::contract::Deck::Red => (4, 4),
            ante_core::contract::Deck::Blue => (5, 3),
            _ => unreachable!("only Red and Blue are observed here"),
        };
        let mut history = Vec::new();
        run.apply(Command::StartBlind).unwrap();
        assert_eq!((run.view().hands, run.view().discards), expected);
        record(&run, &mut history);
        select_cards(&mut run, &DISCARD);
        run.apply(Command::Discard).unwrap();
        play_blind(&mut run, &mut history, &SMALL, None);
        assert_eq!(run.view().phase, Phase::Reward);
        run.apply(Command::CashOut).unwrap();
        run.apply(Command::LeaveShop).unwrap();
        run.apply(Command::StartBlind).unwrap();
        assert_eq!(run.view().blind, "Big Blind");
        assert_eq!((run.view().hands, run.view().discards), expected);
        record(&run, &mut history);
        history
    }

    #[test]
    fn both_decks_restore_limits_next_blind_and_repeat_with_seed() {
        use ante_core::contract::Deck;
        for deck in [Deck::Red, Deck::Blue] {
            assert_eq!(observe_deck(deck), observe_deck(deck));
        }
    }

    #[test]
    fn one_ante_win_is_reproducible() {
        let baseline = winning_run(None);
        for speed in [Speed::ALL[0], Speed::ALL[1], Speed::ALL[3], Speed::ALL[8]] {
            assert_eq!(winning_run(Some(speed)), baseline);
        }
    }
}

#[test]
fn empty_play_leaves_the_run_unchanged_and_running_out_of_hands_loses() {
    let mut run = EngineRun::new(SEED);
    run.apply(Command::StartBlind).unwrap();
    let before = run.view();
    assert_eq!(before.phase, Phase::Playing);
    assert!(!before.can_play);
    assert!(run.apply(Command::Play).is_err());
    assert_eq!(run.view(), before);

    for hands_left in (0..before.hands).rev() {
        select(&mut run, &[1]);
        run.apply(Command::Play).unwrap();
        assert_eq!(run.view().hands, hands_left);
    }
    let lost = run.view();
    assert_eq!(lost.phase, Phase::Lost);
    assert!(lost.score < lost.target);
    let restarted = EngineRun::new(lost.seed).view();
    assert_eq!(
        (restarted.phase, restarted.target),
        (Phase::BeforeBlind, 300)
    );
}

#[test]
fn blind_board_tracks_progress_and_rejects_stale_or_boss_skips() {
    let mut run = EngineRun::new(0);
    let initial = run.view();
    assert_eq!((initial.deck_cards, initial.deck_total), (52, 52));
    assert_eq!(initial.blinds.len(), 3);
    assert_eq!(initial.blinds[0].target, 300);
    assert_eq!(initial.blinds[1].target, 450);
    assert!(initial.blinds[0].selectable && initial.blinds[0].can_skip);
    assert!(initial.blinds[0].tag.is_some());
    let skip = Command::SkipBlind {
        revision: initial.revision,
    };
    run.apply(skip).unwrap();
    assert!(run.apply(skip).is_err());
    while run.view().phase == Phase::Pack {
        run.apply(Command::SkipPack).unwrap();
    }
    let big = run.view();
    assert!(big.blinds[0].passed && big.blinds[0].skipped);
    assert!(big.blinds[1].selectable && big.blinds[1].can_skip);
    run.apply(Command::SkipBlind {
        revision: big.revision,
    })
    .unwrap();
    while run.view().phase == Phase::Pack {
        run.apply(Command::SkipPack).unwrap();
    }
    let boss = run.view();
    assert!(boss.blinds[1].skipped && boss.blinds[2].selectable);
    assert!(!boss.blinds[2].description.is_empty());
    assert!(
        run.apply(Command::SkipBlind {
            revision: boss.revision
        })
        .is_err()
    );
    run.apply(Command::StartBlind).unwrap();
    assert_eq!(run.view().phase, Phase::Playing);
}
