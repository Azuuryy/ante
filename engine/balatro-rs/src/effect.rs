use crate::card::Card;
use crate::game::Game;
use crate::hand::MadeHand;
use crate::joker::{JokerEffects, Jokers};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

type GameHandFn = Arc<Mutex<dyn Fn(&mut Game, MadeHand) + Send + 'static>>;
type GameFn = Arc<Mutex<dyn Fn(&mut Game) + Send + 'static>>;
type GameModifyFn = Arc<Mutex<dyn Fn(&mut Game, &mut MadeHand) + Send + 'static>>;
type CardTriggerFn = Arc<Mutex<dyn Fn(&mut Game, Card, bool) -> usize + Send + 'static>>;
type CardEffectFn = Arc<Mutex<dyn Fn(&mut Game, Card, &MadeHand) + Send + 'static>>;

// A joker's mere presence flips a named rule used elsewhere in scoring/hand
// evaluation, rather than contributing a chip/mult delta at a specific hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleFlag {
    AllCardsAreFace,
}

#[derive(Debug, Clone)]
pub struct EffectRegistry {
    pub on_play: Vec<Effects>,
    pub on_discard: Vec<Effects>,
    pub on_handrank: Vec<Effects>,
    pub on_modify_hand: Vec<Effects>,
    pub on_round_end: Vec<Effects>,
    pub trigger_count_played: Vec<Effects>,
    pub trigger_count_held: Vec<Effects>,
    pub rule_flags: HashSet<RuleFlag>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            on_play: Vec::new(),
            on_discard: Vec::new(),
            on_handrank: Vec::new(),
            on_modify_hand: Vec::new(),
            on_round_end: Vec::new(),
            trigger_count_played: Vec::new(),
            trigger_count_held: Vec::new(),
            rule_flags: HashSet::new(),
        }
    }
}

impl Default for EffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectRegistry {
    pub fn register_jokers(&mut self, jokers: Vec<Jokers>, game: &Game) {
        self.on_play.clear();
        self.on_discard.clear();
        self.on_handrank.clear();
        self.on_modify_hand.clear();
        self.on_round_end.clear();
        self.trigger_count_played.clear();
        self.trigger_count_held.clear();
        self.rule_flags.clear();
        for j in jokers.into_iter().filter(|j| !j.is_disabled()) {
            for e in j.effects(game) {
                match e {
                    Effects::OnPlay(_) => self.on_play.push(e),
                    Effects::OnDiscard(_) => self.on_discard.push(e),
                    Effects::OnScore(_) => {} // not cached, recomputed in score loop on purpose
                    Effects::OnCardScored(_) | Effects::OnCardHeld(_) => {} // same, per card
                    Effects::OnSell(_) => {} // not cached, computed on the sold joker directly in sell_joker
                    Effects::OnHandRank(_) => self.on_handrank.push(e),
                    Effects::OnModifyHand(_) => self.on_modify_hand.push(e),
                    Effects::OnRoundEnd(_) => self.on_round_end.push(e),
                    Effects::TriggerCountPlayed(_) => self.trigger_count_played.push(e),
                    Effects::TriggerCountHeld(_) => self.trigger_count_held.push(e),
                    Effects::RuleFlag(flag) => {
                        self.rule_flags.insert(flag);
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum Effects {
    OnPlay(GameHandFn),
    OnDiscard(GameHandFn),
    OnScore(GameHandFn),
    /// A scored card, once per repetition, after its own effects
    /// (`calculate_joker{individual = true, cardarea = G.play}`).
    OnCardScored(CardEffectFn),
    /// A card held in hand, once per repetition (`individual = true, cardarea = G.hand`).
    OnCardHeld(CardEffectFn),
    OnSell(GameFn),
    OnHandRank(GameFn),
    OnModifyHand(GameModifyFn),
    OnRoundEnd(GameFn),
    TriggerCountPlayed(CardTriggerFn),
    TriggerCountHeld(CardTriggerFn),
    RuleFlag(RuleFlag),
}

impl std::fmt::Debug for Effects {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::OnPlay(_) => write!(f, "OnPlay"),
            Self::OnDiscard(_) => write!(f, "OnDiscard"),
            Self::OnScore(_) => write!(f, "OnScore"),
            Self::OnCardScored(_) => write!(f, "OnCardScored"),
            Self::OnCardHeld(_) => write!(f, "OnCardHeld"),
            Self::OnSell(_) => write!(f, "OnSell"),
            Self::OnHandRank(_) => write!(f, "OnHandRank"),
            Self::OnModifyHand(_) => write!(f, "OnModifyHand"),
            Self::OnRoundEnd(_) => write!(f, "OnRoundEnd"),
            Self::TriggerCountPlayed(_) => write!(f, "TriggerCountPlayed"),
            Self::TriggerCountHeld(_) => write!(f, "TriggerCountHeld"),
            Self::RuleFlag(flag) => write!(f, "RuleFlag({:?})", flag),
        }
    }
}
