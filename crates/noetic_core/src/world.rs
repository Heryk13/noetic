//! What the core asks of the world an animal lives in. The editor answers it from its
//! `SimWorld` and a game from its own scene, so the core never depends on either.
use std::collections::BTreeMap;

use crate::agent::Agent;
use crate::model::Species;

// ==== World View =============================================================

/// TYP-14 The read-only questions an animal asks its world; only the hour and the
/// length of a game hour so far, the rest of TYP-14 joins as the code that asks it does.
pub trait WorldView {
    /// The hour of the game day, from 0.0 up to 24.0.
    fn hour(&self) -> f32;
    /// How many real seconds one game hour lasts, to turn a step's seconds into the
    /// game hours the needs rise by.
    fn secs_per_game_hour(&self) -> f32;
}

// ==== Clock ==================================================================

/// The world's clock: the hour of the game day now and how long a game hour lasts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clock {
    pub hour: f32,
    pub secs_per_game_hour: f32,
}

impl WorldView for Clock {
    fn hour(&self) -> f32 {
        self.hour
    }

    fn secs_per_game_hour(&self) -> f32 {
        self.secs_per_game_hour
    }
}

/// The game hours that `dt` real seconds make in this world; none when a game hour has
/// no length.
pub fn game_hours(view: &dyn WorldView, dt: f32) -> f32 {
    // a game hour of no length means game time stands still, not a division by zero
    let secs = view.secs_per_game_hour();
    if secs > 0.0 { dt / secs } else { 0.0 }
}

// ==== Sim World ==============================================================

/// TYP-13 The world the editor simulates: its clock, its animals in id order and the
/// species they belong to, by name. Wind, terrain, marks, stimuli and the spatial grid
/// join with perception.
#[derive(Debug, Clone, PartialEq)]
pub struct SimWorld {
    pub clock: Clock,
    pub agents: Vec<Agent>,
    pub species: BTreeMap<String, Species>,
}
