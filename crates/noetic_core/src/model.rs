//! The species as a `.noetic.ron` file describes it: needs, brain, activities and the
//! rest of what an animal of that species is.
use std::collections::BTreeMap;

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

/// TYP-10 A personality trait, 0–100: how fearful, curious, social, hungry,
/// energetic, stubborn or aggressive one animal is born (ALG-09).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Enum)]
pub enum Trait {
    Fear,
    Curiosity,
    Social,
    Appetite,
    Energy,
    Stubborn,
    Aggro,
}

// ==== Species ================================================================

/// TYP-01 A species as its `.noetic.ron` file describes it; only the needs, routine,
/// brain and activities so far, the rest of the schema joins as the code that reads it
/// does.
#[derive(Debug, Clone, PartialEq)]
pub struct Species {
    pub needs: NeedRates,
    pub routine: Routine,
    pub brain: Brain,
    pub activities: BTreeMap<String, Activity>,
}

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

/// TYP-08 Something a species knows how to do, scored by the need it lowers; its name
/// is its key in the species' activity map. Its steps join with TYP-09.
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    pub need: Need,
    pub curve: Curve,
    pub weight: f32,
}

/// TYP-07 Which activities the Utility choice weighs, and how much noise it adds to
/// their scores so that animals of one species do not move in step (ALG-10).
#[derive(Debug, Clone, PartialEq)]
pub struct ChooseSpec {
    pub noise: f32,
    pub options: Vec<String>,
}

/// TYP-02 How a species decides: what to do when all is calm (`choose`); how it reacts
/// to danger joins with the reaction graph.
#[derive(Debug, Clone, PartialEq)]
pub struct Brain {
    pub choose: ChooseSpec,
}
