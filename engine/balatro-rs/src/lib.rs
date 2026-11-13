pub mod action;
pub mod ante;

pub fn seed_from_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
pub mod available;
pub mod card;
pub mod config;
pub mod consumable;
pub mod deck;
pub mod effect;
pub mod error;
pub mod game;
pub mod generator;
pub mod hand;
pub mod joker;
pub mod pack;
pub mod planet;
pub mod rank;
#[cfg(feature = "serde")]
pub mod replay_log;
mod rng;
pub mod score;
pub mod shop;
pub mod space;
pub mod spectral;
pub mod stage;
pub mod tag;
pub mod tarot;
pub mod voucher;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod consumable_use_tests;

#[cfg(test)]
mod consumable_highlight_tests;
#[cfg(test)]
mod consumable_room_tests;
#[cfg(test)]
mod debug_tests;
#[cfg(test)]
mod trace_effect_tests;
