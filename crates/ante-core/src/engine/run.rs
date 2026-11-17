use super::*;

pub(super) fn phase_from(stage: Stage) -> Phase {
    match stage {
        Stage::PreBlind() => Phase::BeforeBlind,
        Stage::Blind(_) => Phase::Playing,
        Stage::PostBlind() => Phase::Reward,
        Stage::Shop() => Phase::Shop,
        Stage::End(End::Win) => Phase::Won,
        Stage::End(End::Lose) => Phase::Lost,
        Stage::PackOpen() => Phase::Pack,
    }
}

pub(super) const STARTING_MONEY: usize = 4;

/// The engine's deck for a contract deck.
pub(super) fn engine_deck(deck: Deck) -> balatro_types::DeckVariant {
    use balatro_types::DeckVariant as D;
    match deck {
        Deck::Red => D::Red,
        Deck::Blue => D::Blue,
        Deck::Yellow => D::Yellow,
        Deck::Green => D::Green,
        Deck::Black => D::Black,
        Deck::Magic => D::Magic,
        Deck::Nebula => D::Nebula,
        Deck::Ghost => D::Ghost,
        Deck::Abandoned => D::Abandoned,
        Deck::Checkered => D::Checkered,
        Deck::Zodiac => D::Zodiac,
        Deck::Painted => D::Painted,
        Deck::Anaglyph => D::Anaglyph,
        Deck::Plasma => D::Plasma,
        Deck::Erratic => D::Erratic,
    }
}

pub struct EngineRun {
    pub(super) game: Game,
    pub(super) sort_by_suit: bool,
    pub(super) last_hand_score: Option<usize>,
    pub(super) revision: u64,
    pub(super) skipped_blinds: Vec<Blind>,
    pub(super) stats: RunStats,
    pub(super) debug: bool,
}

impl EngineRun {
    pub fn new(seed: u64) -> Self {
        Self::with_config(RunConfig {
            seed,
            deck: Deck::Red,
            goal: CampaignGoal::Standard,
            starting_money: None,
            stake: 1,
        })
    }

    pub fn with_config(run: RunConfig) -> Self {
        // The base hands and discards; the deck adjusts them (`Back:apply_to_run`).
        let (plays, discards) = (4, 3);
        let config = Config {
            seed: Some(run.seed),
            money_start: run.starting_money.unwrap_or(STARTING_MONEY),
            plays,
            discards,
            ante_end: if run.goal == CampaignGoal::SingleAnte {
                1
            } else {
                8
            },
            rng_mode: RngMode::Fast,
            ..Config::default()
        };
        let mut game = Game::new(config);
        game.apply_deck(engine_deck(run.deck));
        game.apply_stake(run.stake);
        game.start();
        Self {
            game,
            sort_by_suit: false,
            last_hand_score: None,
            revision: 0,
            skipped_blinds: Vec::new(),
            stats: RunStats::default(),
            debug: false,
        }
    }

    /// The engine's events since the last call, as notes.
    pub fn take_notes(&mut self) -> Vec<Note> {
        self.game.take_events().into_iter().map(note).collect()
    }

    pub fn phase(&self) -> Phase {
        phase_from(self.game.stage)
    }

    pub fn snapshot(&self) -> Result<String, serde_json::Error> {
        self.game.to_json()
    }

    pub fn apply(&mut self, action: Command) -> Result<(), RunError> {
        self.resolve(action, None).map(|_| ())
    }

    pub fn apply_traced(
        &mut self,
        action: Command,
        profile_trace: bool,
    ) -> Result<Outcome, RunError> {
        self.resolve(action, Some(profile_trace))
    }
}
