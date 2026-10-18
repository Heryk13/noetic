//! What an animal feels like doing: the scores of the Utility choice and the daily
//! rhythm that weighs them (ALG-10).
use crate::model::Routine;

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
}
