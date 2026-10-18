//! One animal of a species, as the simulation keeps it: its needs, fear, health, memory
//! and what it is doing.
use enum_map::EnumMap;

use crate::model::Trait;
use crate::rng::Rng;
use needs::Needs;

pub mod needs;

// ==== Agent ==================================================================

/// The personality one animal is born with, 0–100 per trait (ALG-09).
pub type TraitMap = EnumMap<Trait, u8>;

/// TYP-11 One animal in the simulation; only the fields the needs tick and the
/// Utility choice read so far, the rest of TYP-11 joins as the code that uses it does.
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub needs: Needs,
    pub traits: TraitMap,
    pub limping: bool,
    pub fear: f32,
    pub rng: Rng,
}
