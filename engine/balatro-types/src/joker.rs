use crate::card::{Edition, Suit, Value};
use crate::rank::HandRank;
use strum::EnumIter;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Self::Common => "Common",
            Self::Uncommon => "Uncommon",
            Self::Rare => "Rare",
            Self::Legendary => "Legendary",
        };
        write!(f, "{s}")
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Categories {
    Chips,
    AdditiveMult,
    MultiplicativeMult,
    ChipsAndAdditiveMult,
    Effect,
    Retrigger,
    Economy,
}

/// Stake-gated per-instance debuff flags, as tracked in real save files.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub struct Stickers {
    pub eternal: bool,
    pub perishable: bool,
    pub rental: bool,
}

/// A rotating-selector value some jokers (MailInRebate, Castle, AncientJoker,
/// TheIdol) lock onto for the run rather than accumulating a number.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectorValue {
    Suit(Suit),
    Value(Value),
    /// A card's rank and suit (The Idol's `idol_card`).
    Card(Value, Suit),
    /// A poker hand (To Do List's `to_do_poker_hand`).
    Hand(HandRank),
}

/// Per-instance persistent state slot, uniform across all jokers and mostly
/// inert for the ones that don't need it.
/// `counter` covers accumulator-style jokers (Hologram, Ramen, LuckyCat, ...).
/// `selector` covers rotating-selector jokers.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct JokerState {
    pub counter: f32,
    pub selector: Option<SelectorValue>,
    /// Sell value gained (`ability.extra_value`: Egg, Gift Card).
    #[cfg_attr(feature = "serde", serde(default))]
    pub extra_value: usize,
    /// Rounds a Perishable Joker still works (`perish_tally`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub perish_rounds: u8,
}

/// `Jokers` is the definitive static repr of all jokers in the game.
// We could pass around `Box<dyn Joker>` but it does not work well with serde.
// Since we know all variants (one for each joker), we define an enum that implements
// our `Joker` trait. This macro just reduces the amount of boilerplate we have to copy
// to match each joker and call its methods. It ends up creating an enum `Jokers` that
// contains each joker struct (where each struct impl `Joker`), and we impl `Joker` for
// `Jokers` enum by matching each case and calling underlying methods.
// https://stackoverflow.com/questions/63848427/using-enums-for-dynamic-polymorphism-in-rust/63849405#63849405
macro_rules! make_jokers {
    ($($x:ident),* $(,)?) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, Eq, PartialEq, Hash, EnumIter)]
        pub enum Jokers {
            $($x($x),)*
        }

        $(
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $x {
            pub edition: Edition,
            pub stickers: Stickers,
            #[cfg_attr(feature = "serde", serde(default))]
            pub instance_id: usize,
            #[cfg_attr(feature = "serde", serde(default))]
            pub state: JokerState,
        }

        impl PartialEq for $x {
            fn eq(&self, other: &Self) -> bool {
                self.instance_id == other.instance_id
                    && self.edition == other.edition
                    && self.stickers == other.stickers
                    && self.state.selector == other.state.selector
            }
        }
        impl Eq for $x {}
        impl std::hash::Hash for $x {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.instance_id.hash(state);
                self.edition.hash(state);
                self.stickers.hash(state);
                self.state.selector.hash(state);
            }
        }
        )*

        impl Jokers {
            fn inner_ref(&self) -> (&Edition, &Stickers, &usize, &JokerState) {
                match self {
                    $(Self::$x(j) => (&j.edition, &j.stickers, &j.instance_id, &j.state),)*
                }
            }

            fn inner_mut(&mut self) -> (&mut Edition, &mut Stickers, &mut usize, &mut JokerState) {
                match self {
                    $(Self::$x(j) => (&mut j.edition, &mut j.stickers, &mut j.instance_id, &mut j.state),)*
                }
            }

            pub fn edition(&self) -> Edition { *self.inner_ref().0 }
            /// `Card:set_cost` before discounts: the base cost plus the edition's
            /// `extra_cost` (Foil 2, Holographic 3, Polychrome and Negative 5).
            pub fn full_cost(&self) -> usize {
                self.cost()
                    + match self.edition() {
                        Edition::Base => 0,
                        Edition::Foil => 2,
                        Edition::Holographic => 3,
                        Edition::Polychrome | Edition::Negative => 5,
                    }
            }
            pub fn stickers(&self) -> Stickers { *self.inner_ref().1 }
            pub fn instance_id(&self) -> usize { *self.inner_ref().2 }
            pub fn state(&self) -> JokerState { *self.inner_ref().3 }
            pub fn state_mut(&mut self) -> &mut JokerState { self.inner_mut().3 }

            pub fn set_edition(&mut self, edition: Edition) { *self.inner_mut().0 = edition; }
            pub fn set_stickers(&mut self, stickers: Stickers) { *self.inner_mut().1 = stickers; }
            pub fn set_instance_id(&mut self, id: usize) { *self.inner_mut().2 = id; }
            pub fn set_state(&mut self, state: JokerState) { *self.inner_mut().3 = state; }
            /// A Perishable Joker past its last round is debuffed: it does nothing.
            pub fn is_disabled(&self) -> bool {
                self.stickers().perishable && self.state().perish_rounds == 0
            }
        }

        impl std::str::FromStr for Jokers {
            type Err = strum::ParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(if s.eq_ignore_ascii_case(stringify!($x)) {
                    return Ok(Self::$x($x::default()));
                })*
                Err(strum::ParseError::VariantNotFound)
            }
        }
    };
}

make_jokers!(
    TheJoker,
    GreedyJoker,
    LustyJoker,
    WrathfulJoker,
    GluttonousJoker,
    JollyJoker,
    ZanyJoker,
    MadJoker,
    CrazyJoker,
    DrollJoker,
    SlyJoker,
    WilyJoker,
    CleverJoker,
    DeviousJoker,
    CraftyJoker,
    HalfJoker,
    JokerStencil,
    FourFingers,
    Mime,
    CreditCard,
    CeremonialDagger,
    Banner,
    MysticSummit,
    MarbleJoker,
    LoyaltyCard,
    EightBall,
    Misprint,
    Dusk,
    RaisedFist,
    ChaosTheClown,
    Fibonacci,
    SteelJoker,
    ScaryFace,
    AbstractJoker,
    DelayedGratification,
    Hack,
    Pareidolia,
    GrosMichel,
    EvenSteven,
    OddTodd,
    Scholar,
    BusinessCard,
    Supernova,
    RideTheBus,
    SpaceJoker,
    Egg,
    Burglar,
    Blackboard,
    Runner,
    IceCream,
    Dna,
    Splash,
    BlueJoker,
    SixthSense,
    Constellation,
    Hiker,
    FacelessJoker,
    GreenJoker,
    Superposition,
    ToDoList,
    Cavendish,
    CardSharp,
    RedCard,
    Madness,
    SquareJoker,
    Seance,
    RiffRaff,
    Vampire,
    Shortcut,
    Hologram,
    Vagabond,
    Baron,
    Cloud9,
    Rocket,
    Obelisk,
    MidasMask,
    Luchador,
    Photograph,
    GiftCard,
    TurtleBean,
    Erosion,
    ReservedParking,
    MailInRebate,
    ToTheMoon,
    Hallucination,
    FortuneTeller,
    Juggler,
    Drunkard,
    StoneJoker,
    GoldenJoker,
    LuckyCat,
    BaseballCard,
    Bull,
    DietCola,
    TradingCard,
    FlashCard,
    Popcorn,
    SpareTrousers,
    AncientJoker,
    Ramen,
    WalkieTalkie,
    Seltzer,
    Castle,
    SmileyFace,
    Campfire,
    GoldenTicket,
    MrBones,
    Acrobat,
    SockAndBuskin,
    Swashbuckler,
    Troubadour,
    Certificate,
    SmearedJoker,
    Throwback,
    HangingChad,
    RoughGem,
    Bloodstone,
    Arrowhead,
    OnyxAgate,
    GlassJoker,
    Showman,
    FlowerPot,
    Blueprint,
    WeeJoker,
    MerryAndy,
    OopsAllSixes,
    TheIdol,
    SeeingDouble,
    Matador,
    HitTheRoad,
    TheDuo,
    TheTrio,
    TheFamily,
    TheOrder,
    TheTribe,
    Stuntman,
    InvisibleJoker,
    Brainstorm,
    Satellite,
    ShootTheMoon,
    DriversLicense,
    Cartomancer,
    Astronomer,
    BurntJoker,
    Bootstraps,
    Canio,
    Triboulet,
    Yorick,
    Chicot,
    Perkeo,
);

#[cfg(test)]
mod tests;

mod data;
