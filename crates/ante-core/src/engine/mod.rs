//! Adapter from the vendored balatro-rs engine to the Ante contract.

use std::time::Duration;

use balatro_rs::{
    action::{Action, MoveDirection as EngineMoveDirection, SortBy},
    card::{Edition, Enhancement, Seal, Suit},
    config::{Config, RngMode},
    game::Game,
    hand::SelectHand,
    pack::PackContent,
    rank::HandRank,
    score::{ScoreSource as NativeSource, ScoreTrace as NativeTrace},
    stage::{Blind, BlindExt, End, Stage, blind_display},
    voucher::Voucher,
};

use crate::contract::*;

mod backend;
mod content;
mod convert;
mod describe;
mod resolve;
mod run;
mod view;

use content::*;
use convert::*;
use describe::*;
use run::*;
use view::*;

pub use backend::ExternalBackend;
pub use content::unlock_condition;
pub use describe::joker_description;
pub use run::EngineRun;

#[cfg(test)]
mod tests;
