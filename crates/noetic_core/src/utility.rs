//! What an animal feels like doing: the scores of the Utility choice and the daily
//! rhythm that weighs them (ALG-10).
use crate::agent::Agent;
use crate::agent::needs::NEED_MAX;
use crate::model::{Curve, Need, Routine, Species, Trait};

// ==== Config =================================================================

/// Need, 0–1, at which a Step curve jumps from no urge to full urge (ALG-10).
const STEP_CURVE_THRESHOLD: f32 = 0.6;
/// How much more resting weighs outside the active hours (API-COR-018).
const OFF_HOURS_REST_FACTOR: f32 = 1.6;
/// How much every other activity weighs outside them.
const OFF_HOURS_OTHER_FACTOR: f32 = 0.35;
/// Pull added to fatigue outside the active hours, so an animal sleeps by day or by
/// night as its routine says (ALG-10).
const OFF_HOURS_FATIGUE_DRIVE: f32 = 55.0;
/// Pull added to fatigue while the animal limps.
const LIMPING_FATIGUE_DRIVE: f32 = 35.0;
/// Pull of fear with no fear at all; the fear felt now adds to it.
const FEAR_DRIVE_BASE: f32 = 25.0;
/// Pull added per point of fear felt now.
const FEAR_DRIVE_PER_FEAR: f32 = 0.8;
/// Pull of calm per point of calm, calm being what fear leaves of 100.
const CALM_DRIVE_PER_CALM: f32 = 0.55;
/// Pull of curiosity in an animal born with none.
const CURIOSITY_DRIVE_BASE: f32 = 20.0;
/// Pull added per point of the curiosity trait.
const CURIOSITY_DRIVE_PER_TRAIT: f32 = 0.6;
/// Pull of boredom in an animal born with no energy.
const BOREDOM_DRIVE_BASE: f32 = 30.0;
/// Pull added per point of the energy trait.
const BOREDOM_DRIVE_PER_ENERGY: f32 = 0.2;

// ==== Systems ================================================================

/// API-COR-019 The hours a routine is active, as (start, end) pairs; a window whose
/// start is after its end runs past midnight (Nocturnal 20–6).
pub fn active_windows(routine: Routine) -> &'static [(f32, f32)] {
    // the table of API-COR-018 itself, kept whole instead of split into constants
    match routine {
        Routine::Diurnal => &[(6.0, 19.0)],
        Routine::Crepuscular => &[(5.0, 9.0), (17.0, 21.0)],
        Routine::Nocturnal => &[(20.0, 6.0)],
        Routine::MorningAfternoon => &[(6.0, 10.0), (15.0, 19.0)],
    }
}

/// Whether `hour` falls inside one of the routine's active windows, a window that runs
/// past midnight included.
pub fn is_active_hour(routine: Routine, hour: f32) -> bool {
    for &(start, end) in active_windows(routine) {
        if start <= end {
            if hour >= start && hour < end {
                return true;
            }
        } else {
            if hour >= start || hour < end {
                return true;
            }
        }
    }
    false
}

/// API-COR-010 The urge a need gives through a curve, both 0–1: `Linear` x,
/// `Accelerating` x², `FastStart` √x, `Step` 0 below the threshold and 1 from it
/// (ALG-10).
pub fn curve_eval(curve: Curve, x: f32) -> f32 {
    match curve {
        Curve::Linear => x,
        Curve::Accelerating => x * x,
        Curve::FastStart => x.sqrt(),
        Curve::Step => {
            if x < STEP_CURVE_THRESHOLD {
                0.0
            } else {
                1.0
            }
        }
    }
}

/// API-COR-018 What the daily rhythm multiplies an activity's score by: 1.0 inside
/// the active hours; outside them, resting weighs more and everything else less.
pub fn routine_factor(routine: Routine, hour: f32, is_rest: bool) -> f32 {
    if is_active_hour(routine, hour) {
        1.0
    } else if is_rest {
        OFF_HOURS_REST_FACTOR
    } else {
        OFF_HOURS_OTHER_FACTOR
    }
}

/// How hard a need pulls the animal now, before its curve (fatigue can pull past 100):
/// most needs pull by their value; fatigue more outside the active hours or when
/// limping, safety by what is missing, fear and calm by the fear felt now, curiosity
/// and boredom by the animal's traits (ALG-10).
pub fn drive(agent: &Agent, species: &Species, need: Need, hour: f32) -> f32 {
    match need {
        Need::Hunger | Need::Thirst | Need::Heat | Need::Sing | Need::Company => agent.needs[need],
        Need::Safety => NEED_MAX - agent.needs[Need::Safety],
        // no scent marks (M8) or fights (M7) yet: their urge stays 0, as with marks off
        Need::Mark | Need::Fight => 0.0,
        Need::Fatigue => {
            let off_hours = if is_active_hour(species.routine, hour) {
                0.0
            } else {
                OFF_HOURS_FATIGUE_DRIVE
            };
            let limping = if agent.limping {
                LIMPING_FATIGUE_DRIVE
            } else {
                0.0
            };
            agent.needs[Need::Fatigue] + off_hours + limping
        }
        Need::Fear => FEAR_DRIVE_BASE + agent.fear * FEAR_DRIVE_PER_FEAR,
        Need::Calm => (NEED_MAX - agent.fear) * CALM_DRIVE_PER_CALM,
        Need::Curiosity => {
            CURIOSITY_DRIVE_BASE
                + f32::from(agent.traits[Trait::Curiosity]) * CURIOSITY_DRIVE_PER_TRAIT
        }
        Need::Boredom => {
            BOREDOM_DRIVE_BASE + f32::from(agent.traits[Trait::Energy]) * BOREDOM_DRIVE_PER_ENERGY
        }
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::TraitMap;
    use crate::agent::needs::Needs;
    use crate::model::NeedRates;

    const TOLERANCE: f32 = 1e-4;
    const NOON_H: f32 = 12.0;
    /// Outside the active hours of a diurnal species.
    const NIGHT_H: f32 = 2.0;

    fn calm_agent() -> Agent {
        Agent {
            needs: Needs::default(),
            traits: TraitMap::default(),
            limping: false,
            fear: 0.0,
        }
    }

    fn diurnal() -> Species {
        Species {
            needs: NeedRates {
                hunger_per_h: 0.0,
                thirst_per_h: 0.0,
                fatigue_per_h: 0.0,
                fear_threshold: NEED_MAX / 2.0,
            },
            routine: Routine::Diurnal,
        }
    }

    fn close(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() <= TOLERANCE
    }

    // ---- active_windows -----------------------------------------------------

    /// UT-046 A window that runs past midnight holds the hours on both sides of it.
    #[test]
    fn a_nocturnal_window_runs_past_midnight() {
        assert!(is_active_hour(Routine::Nocturnal, 23.0));
        assert!(is_active_hour(Routine::Nocturnal, 3.0));
        assert!(!is_active_hour(Routine::Nocturnal, 12.0));
    }

    // ---- curve_eval ---------------------------------------------------------

    /// UT-021 Every curve gives no urge at an empty need and full urge at a full one.
    #[test]
    fn every_curve_runs_from_zero_to_one() {
        for curve in [
            Curve::Linear,
            Curve::Accelerating,
            Curve::FastStart,
            Curve::Step,
        ] {
            // 0 and 1 are exact in every formula, so no tolerance is needed
            assert_eq!(curve_eval(curve, 0.0), 0.0, "{curve:?} at 0");
            assert_eq!(curve_eval(curve, 1.0), 1.0, "{curve:?} at 1");
        }
    }

    // ---- routine_factor -----------------------------------------------------

    /// UT-049 Outside the active hours resting weighs more and the rest less; inside
    /// them nothing changes.
    #[test]
    fn the_daily_rhythm_favors_rest_outside_the_active_hours() {
        // the factors are the constants themselves, so they compare exactly
        assert_eq!(routine_factor(Routine::Diurnal, NOON_H, true), 1.0);
        assert_eq!(routine_factor(Routine::Diurnal, NOON_H, false), 1.0);
        assert_eq!(
            routine_factor(Routine::Diurnal, NIGHT_H, true),
            OFF_HOURS_REST_FACTOR
        );
        assert_eq!(
            routine_factor(Routine::Diurnal, NIGHT_H, false),
            OFF_HOURS_OTHER_FACTOR
        );
    }

    // ---- drive --------------------------------------------------------------

    /// UT-050 Most needs pull by their value, and safety by what is missing of it.
    #[test]
    fn a_need_pulls_by_its_value_and_safety_by_its_lack() {
        let mut agent = calm_agent();
        agent.needs[Need::Hunger] = 70.0;
        agent.needs[Need::Safety] = 30.0;
        assert!(close(drive(&agent, &diurnal(), Need::Hunger, NOON_H), 70.0));
        assert!(close(
            drive(&agent, &diurnal(), Need::Safety, NOON_H),
            NEED_MAX - 30.0
        ));
    }

    /// UT-051 Fatigue pulls harder outside the active hours, and harder still when the
    /// animal limps.
    #[test]
    fn fatigue_pulls_harder_off_hours_and_when_limping() {
        let mut agent = calm_agent();
        let fatigue = 40.0;
        agent.needs[Need::Fatigue] = fatigue;
        let pull = |agent: &Agent, hour| drive(agent, &diurnal(), Need::Fatigue, hour);
        assert!(close(pull(&agent, NOON_H), fatigue));
        assert!(close(
            pull(&agent, NIGHT_H),
            fatigue + OFF_HOURS_FATIGUE_DRIVE
        ));
        agent.limping = true;
        let both = fatigue + OFF_HOURS_FATIGUE_DRIVE + LIMPING_FATIGUE_DRIVE;
        assert!(close(pull(&agent, NIGHT_H), both));
    }

    /// UT-052 Fear and calm follow the fear the animal feels now.
    #[test]
    fn fear_and_calm_follow_the_fear_felt_now() {
        let mut agent = calm_agent();
        let felt = 50.0;
        agent.fear = felt;
        let fear = FEAR_DRIVE_BASE + felt * FEAR_DRIVE_PER_FEAR;
        let calm = (NEED_MAX - felt) * CALM_DRIVE_PER_CALM;
        assert!(close(drive(&agent, &diurnal(), Need::Fear, NOON_H), fear));
        assert!(close(drive(&agent, &diurnal(), Need::Calm, NOON_H), calm));
    }

    /// UT-053 Curiosity and boredom follow the traits the animal was born with.
    #[test]
    fn curiosity_and_boredom_follow_the_traits() {
        let mut agent = calm_agent();
        agent.traits[Trait::Curiosity] = 50;
        agent.traits[Trait::Energy] = 50;
        let curiosity = CURIOSITY_DRIVE_BASE + 50.0 * CURIOSITY_DRIVE_PER_TRAIT;
        let boredom = BOREDOM_DRIVE_BASE + 50.0 * BOREDOM_DRIVE_PER_ENERGY;
        assert!(close(
            drive(&agent, &diurnal(), Need::Curiosity, NOON_H),
            curiosity
        ));
        assert!(close(
            drive(&agent, &diurnal(), Need::Boredom, NOON_H),
            boredom
        ));
    }

    /// UT-054 Marking and fighting pull nothing until scent marks and fights exist.
    #[test]
    fn marking_and_fighting_pull_nothing_yet() {
        let mut agent = calm_agent();
        agent.needs[Need::Mark] = NEED_MAX;
        agent.needs[Need::Fight] = NEED_MAX;
        assert!(close(drive(&agent, &diurnal(), Need::Mark, NOON_H), 0.0));
        assert!(close(drive(&agent, &diurnal(), Need::Fight, NOON_H), 0.0));
    }
}
