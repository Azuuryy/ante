use crate::card::{Card, Edition, Enhancement, Suit, Value};
use crate::effect::{Effects, RuleFlag};
use crate::event::JokerNote;
use crate::game::Game;
use crate::hand::{MadeHand, SelectHand};
use crate::rank::HandRank;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use strum::IntoEnumIterator;

pub use balatro_types::joker::*;

mod effects_1;
mod effects_2;
mod effects_3;
mod effects_4;
mod effects_5;
mod effects_6;
mod effects_7;

static JOKER_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

pub(crate) fn mint_joker_id() -> usize {
    JOKER_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub(crate) fn ensure_joker_id_counter_past(max_seen: usize) {
    JOKER_ID_COUNTER.fetch_max(max_seen + 1, Ordering::SeqCst);
}

/// Assigns a fresh rotating selector to joker if it's a selector-type joker.
pub(crate) fn roll_discard_selector<R: Rng + ?Sized>(rng: &mut R, j: &mut Jokers) {
    const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];
    const VALUES: [Value; 13] = [
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
    match j {
        Jokers::Castle(_) => {
            let suit = SUITS[rng.gen_range(0..SUITS.len())];
            j.state_mut().selector = Some(SelectorValue::Suit(suit));
        }
        Jokers::MailInRebate(_) => {
            let value = VALUES[rng.gen_range(0..VALUES.len())];
            j.state_mut().selector = Some(SelectorValue::Value(value));
        }
        _ => {}
    }
}

/// `balatro_types::Jokers` already supplies all static joker data
/// (name/rarity/cost/desc/category/etc.) as inherent methods.
/// The one thing that can't live there is game behavior.
pub trait JokerEffects {
    fn effects(&self, game: &Game) -> Vec<Effects>;

    /// Whether this joker's `effects()` has real logic behind it.
    /// Keeps unimplemented jokers out of shop/pack generation.
    fn is_implemented(&self) -> bool;
}

impl JokerEffects for Jokers {
    fn effects(&self, game: &Game) -> Vec<Effects> {
        let _ = game;
        effects_1::effects(self)
            .or_else(|| effects_2::effects(self))
            .or_else(|| effects_3::effects(self))
            .or_else(|| effects_4::effects(self))
            .or_else(|| effects_5::effects(self))
            .or_else(|| effects_6::effects(self))
            .or_else(|| effects_7::effects(self))
            .unwrap_or_default()
    }

    fn is_implemented(&self) -> bool {
        matches!(
            self,
            Self::TheJoker(_)
                | Self::GreedyJoker(_)
                | Self::LustyJoker(_)
                | Self::WrathfulJoker(_)
                | Self::GluttonousJoker(_)
                | Self::JollyJoker(_)
                | Self::ZanyJoker(_)
                | Self::MadJoker(_)
                | Self::CrazyJoker(_)
                | Self::DrollJoker(_)
                | Self::SlyJoker(_)
                | Self::WilyJoker(_)
                | Self::CleverJoker(_)
                | Self::DeviousJoker(_)
                | Self::CraftyJoker(_)
                | Self::HalfJoker(_)
                | Self::JokerStencil(_)
                | Self::Banner(_)
                | Self::MysticSummit(_)
                | Self::Supernova(_)
                | Self::RideTheBus(_)
                | Self::CardSharp(_)
                | Self::Obelisk(_)
                | Self::WeeJoker(_)
                | Self::SpareTrousers(_)
                | Self::Fibonacci(_)
                | Self::ScaryFace(_)
                | Self::AbstractJoker(_)
                | Self::Pareidolia(_)
                | Self::EvenSteven(_)
                | Self::OddTodd(_)
                | Self::Scholar(_)
                | Self::BusinessCard(_)
                | Self::FacelessJoker(_)
                | Self::Baron(_)
                | Self::MidasMask(_)
                | Self::Photograph(_)
                | Self::ReservedParking(_)
                | Self::BaseballCard(_)
                | Self::Bull(_)
                | Self::WalkieTalkie(_)
                | Self::SmileyFace(_)
                | Self::GoldenTicket(_)
                | Self::Acrobat(_)
                | Self::RoughGem(_)
                | Self::Bloodstone(_)
                | Self::Arrowhead(_)
                | Self::OnyxAgate(_)
                | Self::ShootTheMoon(_)
                | Self::RaisedFist(_)
                | Self::TheDuo(_)
                | Self::TheTrio(_)
                | Self::TheFamily(_)
                | Self::TheOrder(_)
                | Self::TheTribe(_)
                | Self::StoneJoker(_)
                | Self::SteelJoker(_)
                | Self::BlueJoker(_)
                | Self::Erosion(_)
                | Self::DriversLicense(_)
                | Self::Mime(_)
                | Self::Dusk(_)
                | Self::Hack(_)
                | Self::SockAndBuskin(_)
                | Self::HangingChad(_)
                | Self::TradingCard(_)
                | Self::GreenJoker(_)
                | Self::HitTheRoad(_)
                | Self::Yorick(_)
                | Self::Castle(_)
                | Self::MailInRebate(_)
                | Self::Luchador(_)
                | Self::Chicot(_)
                | Self::Matador(_)
                | Self::Swashbuckler(_)
                | Self::Blackboard(_)
                | Self::FlowerPot(_)
                | Self::SeeingDouble(_)
                | Self::Stuntman(_)
                | Self::Bootstraps(_)
                | Self::GrosMichel(_)
                | Self::Cavendish(_)
                | Self::AncientJoker(_)
                | Self::TheIdol(_)
                | Self::SquareJoker(_)
                | Self::Runner(_)
                | Self::Throwback(_)
                | Self::FortuneTeller(_)
                | Self::FlashCard(_)
                | Self::RedCard(_)
                | Self::Constellation(_)
                | Self::Popcorn(_)
                | Self::TurtleBean(_)
                | Self::Ramen(_)
                | Self::LuckyCat(_)
                | Self::GlassJoker(_)
                | Self::Hiker(_)
                | Self::Vampire(_)
                | Self::Seltzer(_)
                | Self::Hologram(_)
                | Self::Campfire(_)
                | Self::CeremonialDagger(_)
                | Self::Madness(_)
                | Self::DelayedGratification(_)
                | Self::Egg(_)
                | Self::ToTheMoon(_)
                | Self::Cloud9(_)
                | Self::Rocket(_)
                | Self::Satellite(_)
                | Self::ToDoList(_)
                | Self::OopsAllSixes(_)
                | Self::ChaosTheClown(_)
                | Self::Burglar(_)
                | Self::Juggler(_)
                | Self::Drunkard(_)
                | Self::Troubadour(_)
                | Self::MerryAndy(_)
                | Self::MrBones(_)
                | Self::Splash(_)
                | Self::FourFingers(_)
                | Self::Shortcut(_)
                | Self::SmearedJoker(_)
                | Self::EightBall(_)
                | Self::Superposition(_)
                | Self::Seance(_)
                | Self::Vagabond(_)
                | Self::LoyaltyCard(_)
                | Self::DietCola(_)
                | Self::SpaceJoker(_)
                | Self::Hallucination(_)
                | Self::Cartomancer(_)
                | Self::RiffRaff(_)
                | Self::MarbleJoker(_)
                | Self::BurntJoker(_)
        ) || cfg!(feature = "ante-extra-jokers")
            && matches!(
                self,
                Self::Misprint(_) | Self::GoldenJoker(_) | Self::IceCream(_)
            )
    }
}

pub fn joker_catalog() -> impl Iterator<Item = Jokers> {
    Jokers::iter()
}

/// Only returns jokers with real `effects()` behavior implemented - callers
/// (shop/pack generation) must never offer a not-yet-implemented joker to
/// the player. A free function rather than an inherent method since
/// `Jokers` is now a foreign type (orphan rule blocks inherent impls on it).
pub(crate) fn jokers_by_rarity(rarity: Rarity) -> Vec<Jokers> {
    Jokers::iter()
        .filter(|j| j.rarity() == rarity && j.is_implemented())
        .collect()
}

/// `Display` can't be implemented on `Jokers` directly (foreign trait +
/// foreign type both fail the orphan rule), so this free function stands in
/// for the old `impl fmt::Display for Jokers`.
pub fn joker_display(j: &Jokers) -> String {
    let edition = j.edition();
    let edition_str = if edition == Edition::Base {
        String::new()
    } else {
        format!(" [{:?}]", edition)
    };
    let stickers = j.stickers();
    let mut sticker_flags = Vec::new();
    if stickers.eternal {
        sticker_flags.push("Eternal");
    }
    if stickers.perishable {
        sticker_flags.push("Perishable");
    }
    if stickers.rental {
        sticker_flags.push("Rental");
    }
    let sticker_str = if sticker_flags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", sticker_flags.join(","))
    };
    format!(
        "{} [{}]{}{} {}",
        j.name(),
        j.rarity(),
        edition_str,
        sticker_str,
        j.desc()
    )
}

#[cfg(test)]
mod tests;
