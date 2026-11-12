use crate::{
    Card, GameRng, Hand, Rank, SortMode, Suit, contract::*, evaluate, score::Score, sort_cards,
    standard_deck,
};

const TARGET: usize = 300;
const HAND_SIZE: usize = 8;
const REPLAY_VERSION: u32 = 1;

pub struct NativeBackend;

impl GameBackend for NativeBackend {
    fn start(&self, config: RunConfig) -> Box<dyn RunDriver> {
        Box::new(NativeRun::new(config))
    }

    fn catalog(&self, _: ContentGroup) -> &'static [Content] {
        &[]
    }

    fn parse_seed(&self, seed: &str) -> u64 {
        seed.bytes().fold(0xcbf29ce484222325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRun {
    config: RunConfig,
    rng: GameRng,
    phase: Phase,
    deck: Vec<Card>,
    hand: Vec<Card>,
    discarded: Vec<Card>,
    selected: Vec<CardKey>,
    plays: usize,
    discards: usize,
    money: usize,
    reward: usize,
    score: Score,
    last_hand: Option<usize>,
    revision: u64,
    levels: [usize; 12],
}

pub struct NativeReplay {
    pub version: u32,
    pub rng_version: u32,
    pub config: RunConfig,
    pub commands: Vec<Command>,
}

impl NativeReplay {
    pub fn new(config: RunConfig, commands: Vec<Command>) -> Self {
        Self {
            version: REPLAY_VERSION,
            rng_version: GameRng::VERSION,
            config,
            commands,
        }
    }

    pub fn replay(&self) -> Result<NativeRun, RunError> {
        if (self.version, self.rng_version) != (REPLAY_VERSION, GameRng::VERSION) {
            return Err(RunError::Unavailable);
        }
        let mut run = NativeRun::new(self.config);
        for &command in &self.commands {
            run.apply(command, TraceMode::Off)?;
        }
        Ok(run)
    }
}

impl NativeRun {
    pub fn new(config: RunConfig) -> Self {
        let (plays, discards) = match config.deck {
            Deck::Red => (4, 4),
            Deck::Blue => (5, 3),
        };
        let mut rng = GameRng::new(config.seed);
        let mut deck = standard_deck();
        for i in (1..deck.len()).rev() {
            let bound = (i + 1) as u64;
            let threshold = bound.wrapping_neg() % bound;
            let draw = loop {
                let draw = rng.next_u64();
                if draw >= threshold {
                    break draw;
                }
            };
            deck.swap(i, (draw % bound) as usize);
        }
        Self {
            config,
            rng,
            phase: Phase::BeforeBlind,
            deck,
            hand: Vec::with_capacity(HAND_SIZE),
            discarded: Vec::with_capacity(52),
            selected: Vec::with_capacity(5),
            plays,
            discards,
            money: 4,
            reward: 0,
            score: Score::default(),
            last_hand: None,
            revision: 0,
            levels: [0; 12],
        }
    }

    fn draw(&mut self) {
        while self.hand.len() < HAND_SIZE {
            let Some(card) = self.deck.pop() else { break };
            self.hand.push(card);
        }
        self.hand.sort_by_key(|card| std::cmp::Reverse(card.rank));
    }

    fn remove_selected(&mut self) {
        self.hand.retain(|card| {
            if self.selected.contains(&key(*card)) {
                self.discarded.push(*card);
                false
            } else {
                true
            }
        });
        self.selected.clear();
        self.draw();
    }

    fn selection(&self) -> ([Card; 5], usize) {
        let mut cards = [Card {
            id: crate::CardId(0),
            rank: Rank::Two,
            suit: Suit::Clubs,
        }; 5];
        let mut len = 0;
        for &card in &self.hand {
            if self.selected.contains(&key(card)) {
                cards[len] = card;
                len += 1;
            }
        }
        (cards, len)
    }

    fn play(&mut self, mode: TraceMode) -> Result<Option<ScoreTrace>, RunError> {
        let (cards, len) = self.selection();
        let cards = &cards[..len];
        let (hand, gain, trace) = score_hand(cards, mode)?;
        let index = hand as usize;
        let score = self
            .score
            .checked_add(gain)
            .ok_or_else(|| RunError::Simulation("Score limit reached.".into()))?;
        self.last_hand = Some((score.get() - self.score.get()) as usize);
        self.score = score;
        self.levels[index] += 1;
        self.plays -= 1;
        self.remove_selected();
        if self.score.get() >= TARGET as u64 {
            self.reward = 3 + self.plays;
            self.phase = Phase::Reward;
        } else if self.plays == 0 {
            self.phase = Phase::Lost;
        }
        Ok(trace)
    }
}

impl RunDriver for NativeRun {
    fn view(&self) -> RunView {
        let mut deck_view = self
            .deck
            .iter()
            .map(|c| deck_card(*c, true))
            .chain(
                self.hand
                    .iter()
                    .chain(&self.discarded)
                    .map(|c| deck_card(*c, false)),
            )
            .collect::<Vec<_>>();
        deck_view.sort_unstable_by_key(|c| (c.suit, RANKS.find(c.rank), !c.remaining));
        let hand_levels = HANDS
            .iter()
            .enumerate()
            .rev()
            .map(|(i, &(rank, chips, mult))| HandView {
                rank,
                level: HandLevel {
                    level: 1,
                    chips,
                    mult,
                    plays: self.levels[i],
                },
            })
            .collect::<Vec<_>>();
        let (selection, len) = self.selection();
        let selected_hand = evaluate(&selection[..len])
            .ok()
            .map(|hand| hand_levels[11 - hand as usize]);
        RunView {
            seed: self.config.seed,
            revision: self.revision,
            ante: 1,
            ante_end: 1,
            round: 1,
            phase: self.phase,
            blind: "Small Blind".into(),
            blind_chip: Some("bl_small"),
            target: TARGET,
            blinds: vec![BlindView {
                id: "bl_small",
                name: "Small Blind".into(),
                target: TARGET,
                reward: 3,
                description: String::new(),
                selectable: self.phase == Phase::BeforeBlind,
                passed: self.phase == Phase::Won,
                skipped: false,
                tag: None,
                can_skip: false,
            }],
            deck_total: 52,
            deck_cards: self.deck.len(),
            discarded_cards: self.discarded.len(),
            hands: self.plays,
            discards: self.discards,
            money: self.money,
            reward: self.reward,
            score: self.score.get() as usize,
            last_hand_score: self.last_hand,
            campaign_won: self.phase == Phase::Won,
            hand_levels,
            selected_hand,
            deck_view,
            can_sort: self.phase == Phase::Playing,
            can_play: self.phase == Phase::Playing && len > 0,
            can_discard: self.phase == Phase::Playing && len > 0 && self.discards > 0,
            cards: self
                .hand
                .iter()
                .map(|&card| {
                    let selected = self.selected.contains(&key(card));
                    CardView {
                        id: key(card),
                        rank: rank_char(card.rank),
                        suit: suit_name(card.suit),
                        red: matches!(card.suit, Suit::Hearts | Suit::Diamonds),
                        selected,
                        enabled: selected || len < 5,
                        debuffed: false,
                        chips: rank_chips(card.rank),
                        enhancement: String::new(),
                        edition: "Base".into(),
                        seal: String::new(),
                        effect_summary: String::new(),
                    }
                })
                .collect(),
            ..Default::default()
        }
    }

    fn apply(&mut self, command: Command, mode: TraceMode) -> Result<RunUpdate, RunError> {
        let before = self.phase;
        let mut trace = None;
        match (self.phase, command) {
            (Phase::BeforeBlind, Command::StartBlind) => {
                self.phase = Phase::Playing;
                self.draw();
            }
            (Phase::Playing, Command::ToggleCard(id))
                if self.hand.iter().any(|&c| key(c) == id) =>
            {
                if let Some(index) = self.selected.iter().position(|&card| card == id) {
                    self.selected.remove(index);
                } else if self.selected.len() < 5 {
                    self.selected.push(id);
                } else {
                    return Err(RunError::Unavailable);
                }
            }
            (Phase::Playing, Command::Play) => trace = self.play(mode)?,
            (Phase::Playing, Command::Discard)
                if self.discards > 0 && !self.selected.is_empty() =>
            {
                self.discards -= 1;
                self.remove_selected();
            }
            (Phase::Playing, Command::SortRank) => sort_cards(&mut self.hand, SortMode::Rank),
            (Phase::Playing, Command::SortSuit) => sort_cards(&mut self.hand, SortMode::Suit),
            (Phase::Playing, Command::MoveCard { id, direction }) => {
                let Some(index) = self.hand.iter().position(|&card| key(card) == id) else {
                    return Err(RunError::Unavailable);
                };
                let target = match direction {
                    MoveDirection::Left => index.checked_sub(1),
                    MoveDirection::Right => (index + 1 < self.hand.len()).then_some(index + 1),
                }
                .ok_or(RunError::Unavailable)?;
                self.hand.swap(index, target);
            }
            (Phase::Reward, Command::CashOut) => {
                self.money += std::mem::take(&mut self.reward);
                self.phase = Phase::Won;
            }
            _ => return Err(RunError::Unavailable),
        }
        self.revision += 1;
        Ok(RunUpdate::confirmed(
            before,
            before == Phase::Won,
            command,
            Outcome {
                phase: self.phase,
                round_score: self.score.get() as usize,
                campaign_won: self.phase == Phase::Won,
                hand_score: self.last_hand,
                trace,
                trace_time: Default::default(),
                discovered_item: None,
            },
            self.view(),
        ))
    }
}

const HANDS: [(HandKind, usize, usize); 12] = [
    (HandKind::HighCard, 5, 1),
    (HandKind::OnePair, 10, 2),
    (HandKind::TwoPair, 20, 2),
    (HandKind::ThreeOfAKind, 30, 3),
    (HandKind::Straight, 30, 4),
    (HandKind::Flush, 35, 4),
    (HandKind::FullHouse, 40, 4),
    (HandKind::FourOfAKind, 60, 7),
    (HandKind::StraightFlush, 100, 8),
    (HandKind::FiveOfAKind, 120, 12),
    (HandKind::FlushHouse, 140, 14),
    (HandKind::FlushFive, 160, 16),
];

pub fn score_hand(
    cards: &[Card],
    mode: TraceMode,
) -> Result<(Hand, Score, Option<ScoreTrace>), RunError> {
    let hand = evaluate(cards).map_err(|_| RunError::Unavailable)?;
    let index = hand as usize;
    let (kind, chips, mult) = HANDS[index];
    let mut total_chips = chips;
    let mut trace =
        (mode != TraceMode::Off).then(|| ScoreTrace(Vec::with_capacity(cards.len() + 1)));
    if let Some(trace) = &mut trace {
        trace.0.push(ScoreStep {
            source: ScoreSource::HandLevel(kind),
            chips_before: 0,
            chips_after: chips,
            mult_before: 0.0,
            mult_after: mult as f64,
            retrigger: false,
            xmult: None,
            money: 0,
        });
    }
    for card in scoring_cards(hand, cards) {
        let before = total_chips;
        total_chips += rank_chips(card.rank);
        if let Some(trace) = &mut trace {
            trace.0.push(ScoreStep {
                source: ScoreSource::Card {
                    trigger: CardTrigger::Played,
                    id: key(card),
                    rank: rank_char(card.rank),
                    suit: suit_name(card.suit),
                },
                chips_before: before,
                chips_after: total_chips,
                mult_before: mult as f64,
                mult_after: mult as f64,
                retrigger: false,
                xmult: None,
                money: 0,
            });
        }
    }
    let score = Score::new(total_chips as u64)
        .checked_scale(mult as u64, 1)
        .ok_or_else(|| RunError::Simulation("Score limit reached.".into()))?;
    Ok((hand, score, trace))
}

pub fn scoring_cards(hand: Hand, cards: &[Card]) -> impl Iterator<Item = Card> + '_ {
    let mut counts = [0; 15];
    let mut highest = None;
    for &card in cards {
        counts[card.rank as usize] += 1;
        if highest.is_none_or(|c: Card| c.rank < card.rank) {
            highest = Some(card);
        }
    }
    cards.iter().copied().filter(move |&card| {
        let count = counts[card.rank as usize];
        match hand {
            Hand::HighCard => Some(card) == highest,
            Hand::Pair | Hand::TwoPair => count == 2,
            Hand::ThreeOfAKind => count == 3,
            Hand::FourOfAKind => count == 4,
            _ => true,
        }
    })
}

fn key(card: Card) -> CardKey {
    CardKey(card.id.0 as usize)
}

fn rank_chips(rank: Rank) -> usize {
    match rank {
        Rank::Ace => 11,
        _ => (rank as usize).min(10),
    }
}

fn rank_char(rank: Rank) -> char {
    RANKS.as_bytes()[(Rank::Ace as u8 - rank as u8) as usize] as char
}

fn suit_index(suit: Suit) -> usize {
    match suit {
        Suit::Spades => 0,
        Suit::Hearts => 1,
        Suit::Clubs => 2,
        Suit::Diamonds => 3,
    }
}

fn suit_name(suit: Suit) -> &'static str {
    SUITS[suit_index(suit)]
}

fn deck_card(card: Card, remaining: bool) -> DeckCardView {
    DeckCardView {
        rank: rank_char(card.rank),
        suit: suit_index(card.suit),
        effective_suits: 1 << suit_index(card.suit),
        edition: "Base",
        enhancement: None,
        seal: None,
        debuffed: false,
        remaining,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_and_trace_are_independent() {
        let config = RunConfig {
            seed: 42,
            ..Default::default()
        };
        let mut run = NativeRun::new(config);
        let mut commands = vec![Command::StartBlind];
        run.apply(commands[0], TraceMode::Off).unwrap();
        let id = run.view().cards[0].id;
        assert!(run.apply(Command::Play, TraceMode::Off).is_err());
        commands.extend([Command::ToggleCard(id), Command::Play]);
        run.apply(commands[1], TraceMode::Off).unwrap();
        assert!(
            run.apply(Command::Play, TraceMode::Capture)
                .unwrap()
                .outcome
                .trace
                .is_some()
        );
        assert_eq!(run, NativeReplay::new(config, commands).replay().unwrap());
    }

    #[test]
    fn clears_a_blind_and_pays_once() {
        let mut run = NativeRun::new(RunConfig::default());
        run.apply(Command::StartBlind, TraceMode::Off).unwrap();
        run.hand = [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Ten]
            .into_iter()
            .enumerate()
            .map(|(i, rank)| Card {
                id: crate::CardId(100 + i as u32),
                rank,
                suit: Suit::Spades,
            })
            .collect();
        let selected = run.hand.iter().map(|&card| key(card)).collect::<Vec<_>>();
        for id in selected {
            run.apply(Command::ToggleCard(id), TraceMode::Off).unwrap();
        }
        let cleared = run.apply(Command::Play, TraceMode::Capture).unwrap();
        assert_eq!(cleared.view.phase, Phase::Reward);
        let expected_money = cleared.view.money + cleared.view.reward;
        let settled = run.apply(Command::CashOut, TraceMode::Off).unwrap();
        assert_eq!(
            (settled.view.phase, settled.view.money),
            (Phase::Won, expected_money)
        );
        assert!(run.apply(Command::CashOut, TraceMode::Off).is_err());
        assert_eq!(run.view().money, expected_money);
    }

    #[test]
    fn two_pair_scoring_ignores_kicker_and_preserves_order() {
        let cards = [Rank::Five, Rank::Nine, Rank::Six, Rank::Five, Rank::Six].map(|rank| Card {
            id: crate::CardId(0),
            rank,
            suit: if rank == Rank::Nine {
                Suit::Clubs
            } else {
                Suit::Hearts
            },
        });
        assert_eq!(evaluate(&cards), Ok(Hand::TwoPair));
        assert_eq!(
            scoring_cards(Hand::TwoPair, &cards)
                .map(|c| c.rank)
                .collect::<Vec<_>>(),
            [Rank::Five, Rank::Six, Rank::Five, Rank::Six]
        );
    }
}
