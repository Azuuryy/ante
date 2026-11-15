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
        })
    }

    pub fn with_config(run: RunConfig) -> Self {
        let (plays, discards) = match run.deck {
            Deck::Red => (4, 4),
            Deck::Blue => (5, 3),
        };
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
