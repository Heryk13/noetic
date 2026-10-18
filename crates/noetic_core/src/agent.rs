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

/// TYP-12 What the animal is doing now: choosing, or in an activity at one of its steps
/// with `t_left` seconds to go; reacting, fighting and dead join with their code.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AgentState {
    #[default]
    Choosing,
    InActivity {
        act: String,
        step: usize,
        t_left: f32,
    },
}

/// TYP-11 One animal in the simulation; only the fields the needs tick, the Utility
/// choice and the steps read so far, the rest of TYP-11 joins as the code that uses it
/// does.
#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub needs: Needs,
    pub traits: TraitMap,
    pub limping: bool,
    pub fear: f32,
    pub rng: Rng,
    pub state: AgentState,
}

// ==== Tests ==================================================================

#[cfg(test)]
impl Agent {
    /// A calm animal for the tests of every file: every need and trait at 0, no fear,
    /// not limping, seed 1 and still choosing what to do.
    pub(crate) fn calm() -> Self {
        Agent {
            needs: Needs::default(),
            traits: TraitMap::default(),
            limping: false,
            fear: 0.0,
            rng: Rng::seed(1),
            state: AgentState::default(),
        }
    }
}
