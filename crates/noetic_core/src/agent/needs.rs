//! The needs of one animal and how they rise with time, the hour and the place (ALG-02).
use enum_map::EnumMap;

use crate::model::Need;

// ==== Needs ==================================================================

/// The current value of every need of one animal, 0–100, read by `Need`.
pub type Needs = EnumMap<Need, f32>;
