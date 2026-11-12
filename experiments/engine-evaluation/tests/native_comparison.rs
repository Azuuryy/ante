use ante_core::{Card, CardId, Rank, Suit, contract::TraceMode, native::score_hand};
use balatro_rs::{
    card::{Card as ExternalCard, Suit as ExternalSuit, Value},
    config::Config,
    game::Game,
    hand::SelectHand,
    score::ScoreSource,
};

#[test]
fn pair_and_high_card_score_the_same_controlled_cards() {
    for (values, ranks) in [
        (
            [Value::King, Value::Ace, Value::King],
            [Rank::King, Rank::Ace, Rank::King],
        ),
        (
            [Value::Queen, Value::Ace, Value::King],
            [Rank::Queen, Rank::Ace, Rank::King],
        ),
    ] {
        let native = ranks
            .into_iter()
            .enumerate()
            .map(|(i, rank)| Card {
                id: CardId(i as u32),
                rank,
                suit: [Suit::Clubs, Suit::Hearts, Suit::Spades][i],
            })
            .collect::<Vec<_>>();
        let external = values
            .into_iter()
            .zip([ExternalSuit::Club, ExternalSuit::Heart, ExternalSuit::Spade])
            .map(|(value, suit)| ExternalCard::new(value, suit))
            .collect::<Vec<_>>();
        let best = SelectHand::new(external.clone()).best_hand().unwrap();
        let (expected, trace) = Game::new(Config {
            seed: Some(0),
            ..Default::default()
        })
        .calc_score_traced(best);
        let (_, actual, native_trace) = score_hand(&native, TraceMode::Capture).unwrap();
        assert_eq!(actual.get(), expected as u64);
        let external_order = trace
            .0
            .iter()
            .filter_map(|step| match step.source {
                ScoreSource::PlayedCard(card) => {
                    external.iter().position(|other| other.id == card.id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let native_order = native_trace
            .unwrap()
            .0
            .into_iter()
            .filter_map(|step| match step.source {
                ante_core::contract::ScoreSource::Card { id, .. } => Some(id.0),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(native_order, external_order);
    }
}
