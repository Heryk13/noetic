//! What an animal feels like doing: the scores of the Utility choice and the daily
//! rhythm that weighs them (ALG-10).
use crate::model::{Curve, Routine};

// ==== Config =================================================================

/// Need, 0–1, at which a Step curve jumps from no urge to full urge (ALG-10).
const STEP_CURVE_THRESHOLD: f32 = 0.6;
/// How much more resting weighs outside the active hours (API-COR-018).
const OFF_HOURS_REST_FACTOR: f32 = 1.6;
/// How much every other activity weighs outside them.
const OFF_HOURS_OTHER_FACTOR: f32 = 0.35;

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

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        let (noon, night) = (12.0, 2.0);
        // the factors are the constants themselves, so they compare exactly
        assert_eq!(routine_factor(Routine::Diurnal, noon, true), 1.0);
        assert_eq!(routine_factor(Routine::Diurnal, noon, false), 1.0);
        assert_eq!(
            routine_factor(Routine::Diurnal, night, true),
            OFF_HOURS_REST_FACTOR
        );
        assert_eq!(
            routine_factor(Routine::Diurnal, night, false),
            OFF_HOURS_OTHER_FACTOR
        );
    }
}
