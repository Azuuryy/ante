use balatro_rs::{
    action::Action,
    config::{Config, RngMode},
    game::Game,
    hand::SelectHand,
    score::ScoreTrace,
};

pub struct ExperimentalEngine(Game);
impl ExperimentalEngine {
    pub fn new(seed: u64) -> Self {
        let config = Config {
            seed: Some(seed),
            rng_mode: RngMode::Fast,
            ..Config::default()
        };
        Self(Game::new(config))
    }
    pub fn state(&self) -> &Game {
        &self.0
    }
    pub fn apply(&mut self, action: Action) -> Result<(), balatro_rs::error::GameError> {
        self.0.handle_action(action)
    }
    pub fn score_selected(&self) -> Result<(usize, ScoreTrace), balatro_rs::error::PlayHandError> {
        let hand = SelectHand::new(self.0.available.selected()).best_hand()?;
        Ok(self.0.clone().calc_score_traced(hand))
    }
    pub fn save(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.0)
    }
    pub fn load(snapshot: &str) -> Result<Self, serde_json::Error> {
        Game::from_json(snapshot).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use balatro_rs::stage::Blind;
    fn start(seed: u64) -> ExperimentalEngine {
        let mut engine = ExperimentalEngine::new(seed);
        engine.apply(Action::SelectBlind(Blind::Small)).unwrap();
        engine
    }
    #[test]
    fn deterministic_deal_discard_and_snapshot() {
        let mut a = start(12345);
        let mut b = ExperimentalEngine::load(&a.save().unwrap()).unwrap();
        assert_eq!(
            a.state().available.not_selected(),
            b.state().available.not_selected()
        );
        let card = a.state().available.not_selected()[0];
        for engine in [&mut a, &mut b] {
            engine.apply(Action::SelectCard(card)).unwrap();
            engine.apply(Action::Discard()).unwrap();
        }
        assert_eq!(
            a.state().available.not_selected(),
            b.state().available.not_selected()
        );
    }
    #[test]
    fn load_restores_active_joker_effects() {
        use balatro_rs::joker::Jokers;
        let mut game = start(42);
        game.0.jokers.push(Jokers::HangingChad(Default::default()));
        let snapshot = game.0.clone();
        game.0
            .effect_registry
            .register_jokers(snapshot.jokers.clone(), &snapshot);
        assert!(!game.0.effect_registry.trigger_count_played.is_empty());
        let loaded = ExperimentalEngine::load(&game.save().unwrap()).unwrap();
        assert_eq!(
            loaded.0.effect_registry.trigger_count_played.len(),
            game.0.effect_registry.trigger_count_played.len()
        );
    }
    #[test]
    fn trace_can_be_read_without_mutating_authoritative_state() {
        let mut game = start(7);
        let card = game.state().available.not_selected()[0];
        game.apply(Action::SelectCard(card)).unwrap();
        let before = game.save().unwrap();
        let (_, trace) = game.score_selected().unwrap();
        assert!(!trace.0.is_empty());
        assert_eq!(game.save().unwrap(), before);
        game.apply(Action::Play()).unwrap();
    }
}

#[cfg(test)]
mod comparison {
    use balatro_rs::{
        card::{Card, Enhancement, Suit, Value},
        hand::SelectHand,
    };
    use balatrust_core::{
        card::{Enhancement as TEnhancement, PlayingCard, Rank, Suit as TSuit},
        hand::detect_hand,
    };
    #[test]
    fn pair_scoring_order_and_invalid_sizes() {
        let cards = vec![
            Card::new(Value::Ace, Suit::Heart),
            Card::new(Value::King, Suit::Heart),
            Card::new(Value::King, Suit::Club),
        ];
        let result = SelectHand::new(cards.clone()).best_hand().unwrap();
        assert_eq!(format!("{:?}", result.rank), "OnePair");
        let t = vec![
            PlayingCard::new(Rank::Ace, TSuit::Hearts),
            PlayingCard::new(Rank::King, TSuit::Hearts),
            PlayingCard::new(Rank::King, TSuit::Clubs),
        ];
        let found = detect_hand(&t);
        assert_eq!(format!("{:?}", found.hand_type), "Pair");
        assert_eq!(found.scoring_indices, vec![1, 2]);
        assert!(SelectHand::new(vec![]).best_hand().is_err());
        assert!(SelectHand::new(vec![cards[0]; 6]).best_hand().is_err());
        assert!(detect_hand(&[]).scoring_indices.is_empty());
    }
    #[test]
    fn all_hand_categories_match_expected_results() {
        use Rank::*;
        let ranks = [
            Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
        ];
        let values = [
            Value::Two,
            Value::Three,
            Value::Four,
            Value::Five,
            Value::Six,
            Value::Seven,
            Value::Eight,
            Value::Nine,
            Value::Ten,
            Value::Jack,
            Value::Queen,
            Value::King,
            Value::Ace,
        ];
        let cases = [
            ([0, 2, 4, 6, 8], false, "HighCard", "HighCard"),
            ([0, 0, 4, 6, 8], false, "OnePair", "Pair"),
            ([0, 0, 4, 4, 8], false, "TwoPair", "TwoPair"),
            ([0, 0, 0, 6, 8], false, "ThreeOfAKind", "ThreeOfAKind"),
            ([0, 1, 2, 3, 4], false, "Straight", "Straight"),
            ([0, 2, 4, 6, 8], true, "Flush", "Flush"),
            ([0, 0, 0, 6, 6], false, "FullHouse", "FullHouse"),
            ([0, 0, 0, 0, 8], false, "FourOfAKind", "FourOfAKind"),
            ([0, 1, 2, 3, 4], true, "StraightFlush", "StraightFlush"),
            ([0, 0, 0, 0, 0], false, "FiveOfAKind", "FiveOfAKind"),
            ([0, 0, 0, 6, 6], true, "FlushHouse", "FlushHouse"),
            ([0, 0, 0, 0, 0], true, "FlushFive", "FlushFive"),
        ];
        let suits = [
            Suit::Heart,
            Suit::Club,
            Suit::Diamond,
            Suit::Spade,
            Suit::Heart,
        ];
        let tsuits = [
            TSuit::Hearts,
            TSuit::Clubs,
            TSuit::Diamonds,
            TSuit::Spades,
            TSuit::Hearts,
        ];
        for (indices, flush, expected, t_expected) in cases {
            let cards = indices
                .iter()
                .enumerate()
                .map(|(i, &r)| Card::new(values[r], if flush { Suit::Heart } else { suits[i] }))
                .collect();
            let other: Vec<_> = indices
                .iter()
                .enumerate()
                .map(|(i, &r)| {
                    PlayingCard::new(ranks[r], if flush { TSuit::Hearts } else { tsuits[i] })
                })
                .collect();
            assert_eq!(
                format!("{:?}", SelectHand::new(cards).best_hand().unwrap().rank),
                expected
            );
            assert_eq!(format!("{:?}", detect_hand(&other).hand_type), t_expected);
        }
    }
    #[test]
    fn wild_flush() {
        let values = [
            Value::Two,
            Value::Four,
            Value::Six,
            Value::Eight,
            Value::Ten,
        ];
        let mut cards: Vec<_> = values
            .into_iter()
            .map(|v| Card::new(v, Suit::Heart))
            .collect();
        cards[4].suit = Suit::Club;
        cards[4].enhancement = Some(Enhancement::Wild);
        assert_eq!(
            format!("{:?}", SelectHand::new(cards).best_hand().unwrap().rank),
            "Flush"
        );
        let ranks = [Rank::Two, Rank::Four, Rank::Six, Rank::Eight, Rank::Ten];
        let mut t: Vec<_> = ranks
            .into_iter()
            .map(|r| PlayingCard::new(r, TSuit::Hearts))
            .collect();
        t[4].suit = TSuit::Clubs;
        t[4].enhancement = Some(TEnhancement::Wild);
        assert_eq!(format!("{:?}", detect_hand(&t).hand_type), "Flush");
    }
}

pub mod mvp;

#[path = "../../../crates/ante-app/src/app/playback.rs"]
pub mod playback;
