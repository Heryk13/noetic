//! The golden determinism test: the default world, run from one seed, always ends in the
//! same state (NFR-01).
use std::collections::BTreeMap;

use noetic_core::agent::needs::Needs;
use noetic_core::agent::{Agent, AgentState, TraitMap};
use noetic_core::model::{
    Activity, Brain, ChooseSpec, Curve, Need, NeedRates, Routine, Species, Step, StepKind, Trait,
};
use noetic_core::rng::Rng;
use noetic_core::sim::tick;
use noetic_core::world::{Clock, SimWorld};

// ==== Config =================================================================

const SEED: u64 = 42;
const TICKS: u32 = 1000;
const STEP_SECS: f32 = 0.5;
const SECS_PER_GAME_HOUR: f32 = 30.0;
/// Dawn, so the run crosses the active hours, dusk and the night.
const START_H: f32 = 6.0;
const AGENT_COUNT: u64 = 4;
/// The hash of the default world after `TICKS` steps. A change to the simulation, or a
/// `rand` upgrade that changes the draws of a seed, refreshes it on purpose
/// (Detailed Design 0.12).
const GOLDEN: u64 = 0x5a44_cdb4_b9da_a2fe;

// ---- FNV-1a 64 --------------------------------------------------------------

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

// ==== Helpers ================================================================

/// The default world: four rabbits that graze, drink and rest, each from its own
/// seed and with its own needs, so they drift apart.
fn default_world() -> SimWorld {
    let agents = (0..AGENT_COUNT)
        .map(|id| {
            let mut needs = Needs::default();
            needs[Need::Hunger] = 20.0 * id as f32;
            needs[Need::Thirst] = 60.0 - 15.0 * id as f32;
            let mut traits = TraitMap::default();
            traits[Trait::Appetite] = 50;
            traits[Trait::Energy] = 50;
            Agent {
                species: "rabbit".to_string(),
                needs,
                traits,
                limping: false,
                fear: 0.0,
                rng: Rng::seed(SEED + id),
                state: AgentState::default(),
            }
        })
        .collect();
    SimWorld {
        clock: Clock {
            hour: START_H,
            secs_per_game_hour: SECS_PER_GAME_HOUR,
        },
        agents,
        species: BTreeMap::from([("rabbit".to_string(), rabbit())]),
    }
}

/// A diurnal rabbit that grazes when hungry, drinks when thirsty and rests when tired.
fn rabbit() -> Species {
    let activities = BTreeMap::from([
        (
            "graze".to_string(),
            activity(
                Need::Hunger,
                Curve::Linear,
                &[(StepKind::Wander, (2.0, 4.0)), (StepKind::Eat, (3.0, 6.0))],
            ),
        ),
        (
            "drink".to_string(),
            activity(
                Need::Thirst,
                Curve::Accelerating,
                &[(StepKind::Drink, (2.0, 4.0))],
            ),
        ),
        (
            "rest".to_string(),
            activity(
                Need::Fatigue,
                Curve::FastStart,
                &[(StepKind::Rest, (5.0, 10.0))],
            ),
        ),
    ]);
    Species {
        needs: NeedRates {
            hunger_per_h: 12.0,
            thirst_per_h: 15.0,
            fatigue_per_h: 8.0,
            fear_threshold: 50.0,
        },
        routine: Routine::Diurnal,
        brain: Brain {
            choose: ChooseSpec {
                noise: 0.2,
                options: activities.keys().cloned().collect(),
            },
        },
        activities,
    }
}

/// An activity of weight 1 that lowers `need`, done as `steps` in order.
fn activity(need: Need, curve: Curve, steps: &[(StepKind, (f32, f32))]) -> Activity {
    Activity {
        need,
        curve,
        weight: 1.0,
        steps: steps
            .iter()
            .map(|&(do_, secs)| Step {
                do_,
                secs,
                anim: None,
            })
            .collect(),
    }
}

/// FNV-1a over the bytes: written here rather than `DefaultHasher`, whose algorithm
/// may change between Rust releases and would break the golden value with it.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
    })
}

// ==== Tests ==================================================================

/// UT-025 The default world run from seed 42 for 1000 ticks always ends in the golden
/// state, bit for bit on the same platform.
#[test]
fn the_default_world_always_ends_in_the_golden_state() {
    let mut world = default_world();
    for _ in 0..TICKS {
        tick(&mut world, STEP_SECS);
    }
    // `Debug` prints every f32 with the digits that read back to the same bits, so
    // the text holds the whole state exactly
    let hash = fnv1a(format!("{world:?}").as_bytes());
    assert_eq!(
        hash, GOLDEN,
        "state hash {hash:#018x} is not the golden one"
    );
}
