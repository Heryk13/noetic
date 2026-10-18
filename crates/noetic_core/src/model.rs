//! The species as a `.noetic.ron` file describes it: needs, brain, activities and the
//! rest of what an animal of that species is.
use enum_map::Enum;

// ==== Enums ==================================================================

/// TYP-10 What an animal can want; each activity lowers one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum)]
pub enum Need {
    Hunger,
    Thirst,
    Fatigue,
    Fear,
    Safety,
    Company,
    Curiosity,
    Boredom,
    Mark,
    Fight,
    Sing,
    Heat,
    Calm,
}

/// TYP-10 How the value of a need turns into the urge to do an activity (ALG-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    Linear,
    Accelerating,
    FastStart,
    Step,
}

/// TYP-10 The hours a species is active; outside them rest weighs more (API-COR-018).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Routine {
    #[default]
    Diurnal,
    Crepuscular,
    Nocturnal,
    MorningAfternoon,
}

// ==== Species ================================================================

/// How fast hunger, thirst and fatigue rise per game hour (0–60 each), and the fear
/// (0–100) above which the animal reacts; the other needs follow their driver
/// (PRC-04).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeedRates {
    pub hunger_per_h: f32,
    pub thirst_per_h: f32,
    pub fatigue_per_h: f32,
    pub fear_threshold: f32,
}
