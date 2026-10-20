//! Needs, perception, memory, fear, the reaction graph and activity choice (Utility) and
//! execution (behavior tree), as a pure `tick` with no Bevy dependency.
// no panic in core (EP-01): an unwrap is a panic waiting for a bad file
#![deny(clippy::unwrap_used)]

pub mod agent;
pub mod model;
pub mod rng;
pub mod sim;
pub mod steps;
pub mod utility;
pub mod world;
