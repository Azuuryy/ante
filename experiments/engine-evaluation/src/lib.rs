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
        serde_json::from_str(snapshot).map(Self)
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
