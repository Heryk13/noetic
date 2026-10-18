//! The needs of one animal and how they rise with time, the hour and the place (ALG-02).
use std::ops::Range;

use enum_map::EnumMap;

use crate::agent::Agent;
use crate::model::{Need, Species, Trait};
use crate::utility::is_active_hour;
use crate::world::WorldView;

// ==== Config =================================================================

/// Hunger factor of an animal with appetite 0; appetite 50 brings it to 1.0 (ALG-02).
const HUNGER_FACTOR_MIN: f32 = 0.6;
/// Appetite points that add 1.0 to the hunger factor (ALG-02).
const APPETITE_PER_HUNGER_FACTOR: f32 = 125.0;
/// Lowest value of any need.
const NEED_MIN: f32 = 0.0;
/// Highest value of any need.
pub(crate) const NEED_MAX: f32 = 100.0;
/// Hours of the day when the urge to get out of the heat rises (ALG-02).
const HOT_HOURS: Range<f32> = 10.5..16.5;
/// How fast that urge rises per game hour inside the hot hours.
const HEAT_RISE_PER_H: f32 = 40.0;
/// How fast it drops per game hour outside them.
const HEAT_DROP_PER_H: f32 = 30.0;
/// Morning hours when the urge to sing rises, the dawn chorus (ALG-02).
const MORNING_SONG_HOURS: Range<f32> = 5.0..10.0;
/// Evening hours when it rises again.
const EVENING_SONG_HOURS: Range<f32> = 16.0..19.0;
/// How fast the urge to sing rises per game hour inside the song hours.
const SING_RISE_PER_H: f32 = 90.0;
/// How fast it drops per game hour outside them.
const SING_DROP_PER_H: f32 = 50.0;
/// How much faster fatigue rises outside the species' active hours (ALG-02).
const OFF_HOURS_FATIGUE_FACTOR: f32 = 1.6;
/// Fatigue factor of an animal with energy 0; energy 45 brings it to 1.0.
const FATIGUE_FACTOR_MAX: f32 = 1.3;
/// Energy points that take 1.0 off the fatigue factor.
const ENERGY_PER_FATIGUE_FACTOR: f32 = 150.0;

// ==== Needs ==================================================================

/// The current value of every need of one animal, 0–100, read by `Need`.
pub type Needs = EnumMap<Need, f32>;

// ==== Systems ================================================================

/// API-COR-002 Moves the needs with the game hours that passed: hunger, thirst and
/// fatigue rise at the species rate (hunger with appetite, fatigue with energy and
/// faster outside the active hours), heat and the urge to sing follow the hour; every
/// need stays within 0–100.
pub fn update_needs(agent: &mut Agent, species: &Species, view: &dyn WorldView, delta_hours: f32) {
    agent.needs[Need::Thirst] += species.needs.thirst_per_h * delta_hours;
    let appetite = f32::from(agent.traits[Trait::Appetite]);
    agent.needs[Need::Hunger] += species.needs.hunger_per_h
        * (HUNGER_FACTOR_MIN + appetite / APPETITE_PER_HUNGER_FACTOR)
        * delta_hours;
    let hour = view.hour();
    let energy = f32::from(agent.traits[Trait::Energy]);
    let hour_factor = if is_active_hour(species.routine, hour) {
        1.0
    } else {
        OFF_HOURS_FATIGUE_FACTOR
    };
    agent.needs[Need::Fatigue] += species.needs.fatigue_per_h
        * hour_factor
        * (FATIGUE_FACTOR_MAX - energy / ENERGY_PER_FATIGUE_FACTOR)
        * delta_hours;
    agent.needs[Need::Heat] += (if HOT_HOURS.contains(&hour) {
        HEAT_RISE_PER_H
    } else {
        -HEAT_DROP_PER_H
    }) * delta_hours;
    agent.needs[Need::Sing] +=
        (if MORNING_SONG_HOURS.contains(&hour) || EVENING_SONG_HOURS.contains(&hour) {
            SING_RISE_PER_H
        } else {
            -SING_DROP_PER_H
        }) * delta_hours;
    for v in agent.needs.values_mut() {
        *v = v.clamp(NEED_MIN, NEED_MAX);
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const RATE_PER_H: f32 = 10.0;
    const TOLERANCE: f32 = 1e-4;
    /// An hour these tests do not depend on.
    const NOON_H: f32 = 12.0;
    /// An hour outside every range of the needs that follow the hour.
    const NIGHT_H: f32 = 2.0;

    /// A world that only knows the hour.
    struct FakeWorld {
        hour: f32,
    }

    impl WorldView for FakeWorld {
        fn hour(&self) -> f32 {
            self.hour
        }
    }

    fn noon() -> FakeWorld {
        FakeWorld { hour: NOON_H }
    }

    fn at(hour: f32) -> FakeWorld {
        FakeWorld { hour }
    }

    /// The hour in the middle of a range, away from its edges.
    fn middle(hours: Range<f32>) -> f32 {
        (hours.start + hours.end) / 2.0
    }

    // ---- update_needs -------------------------------------------------------

    /// UT-039 Thirst rises by the species rate for every game hour that passes.
    #[test]
    fn thirst_rises_with_the_hours() {
        let mut agent = Agent::calm();
        let hours = 2.0;
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &noon(), hours);
        let expected = RATE_PER_H * hours;
        assert!((agent.needs[Need::Thirst] - expected).abs() <= TOLERANCE);
    }

    /// UT-040 An animal of average appetite gets hungry exactly at the species rate.
    #[test]
    fn average_appetite_keeps_the_species_hunger_rate() {
        let mut agent = Agent::calm();
        // the appetite whose factor is exactly 1.0
        let average = (1.0 - HUNGER_FACTOR_MIN) * APPETITE_PER_HUNGER_FACTOR;
        agent.traits[Trait::Appetite] = average.round() as u8;
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &noon(), 1.0);
        assert!((agent.needs[Need::Hunger] - RATE_PER_H).abs() <= TOLERANCE);
    }

    /// UT-041 A need never goes past its maximum, however long the animal waits.
    #[test]
    fn needs_never_pass_their_maximum() {
        let mut agent = Agent::calm();
        agent.needs[Need::Hunger] = NEED_MAX - RATE_PER_H / 2.0;
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &noon(), 10.0);
        assert!((agent.needs[Need::Hunger] - NEED_MAX).abs() <= TOLERANCE);
        assert!((agent.needs[Need::Thirst] - NEED_MAX).abs() <= TOLERANCE);
    }

    /// UT-042 The urge to get out of the heat rises during the hot hours.
    #[test]
    fn heat_rises_in_the_hot_hours() {
        let mut agent = Agent::calm();
        update_needs(
            &mut agent,
            &Species::diurnal(RATE_PER_H),
            &at(middle(HOT_HOURS)),
            1.0,
        );
        assert!((agent.needs[Need::Heat] - HEAT_RISE_PER_H).abs() <= TOLERANCE);
    }

    /// UT-043 Outside the hot hours that urge drops, and never below zero.
    #[test]
    fn heat_drops_at_night_and_stops_at_zero() {
        let mut agent = Agent::calm();
        agent.needs[Need::Heat] = HEAT_DROP_PER_H / 2.0;
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &at(NIGHT_H), 1.0);
        assert!((agent.needs[Need::Heat] - NEED_MIN).abs() <= TOLERANCE);
    }

    /// UT-044 The urge to sing rises in both the morning and the evening song hours.
    #[test]
    fn the_urge_to_sing_rises_at_dawn_and_dusk() {
        let hours = 0.5;
        for hour in [middle(MORNING_SONG_HOURS), middle(EVENING_SONG_HOURS)] {
            let mut agent = Agent::calm();
            update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &at(hour), hours);
            let expected = SING_RISE_PER_H * hours;
            assert!(
                (agent.needs[Need::Sing] - expected).abs() <= TOLERANCE,
                "at {hour} h the urge to sing is {}, expected {expected}",
                agent.needs[Need::Sing]
            );
        }
    }

    /// UT-045 Outside the song hours the urge to sing drops.
    #[test]
    fn the_urge_to_sing_drops_at_midday() {
        let mut agent = Agent::calm();
        agent.needs[Need::Sing] = NEED_MAX;
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &noon(), 1.0);
        let expected = NEED_MAX - SING_DROP_PER_H;
        assert!((agent.needs[Need::Sing] - expected).abs() <= TOLERANCE);
    }

    /// The energy whose fatigue factor is exactly 1.0.
    fn average_energy() -> u8 {
        ((FATIGUE_FACTOR_MAX - 1.0) * ENERGY_PER_FATIGUE_FACTOR).round() as u8
    }

    /// UT-047 During its active hours an animal of average energy tires at the species
    /// rate.
    #[test]
    fn average_energy_tires_at_the_species_rate() {
        let mut agent = Agent::calm();
        agent.traits[Trait::Energy] = average_energy();
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &noon(), 1.0);
        assert!((agent.needs[Need::Fatigue] - RATE_PER_H).abs() <= TOLERANCE);
    }

    /// UT-048 Outside its active hours the same animal tires faster.
    #[test]
    fn fatigue_rises_faster_outside_the_active_hours() {
        let mut agent = Agent::calm();
        agent.traits[Trait::Energy] = average_energy();
        update_needs(&mut agent, &Species::diurnal(RATE_PER_H), &at(NIGHT_H), 1.0);
        let expected = RATE_PER_H * OFF_HOURS_FATIGUE_FACTOR;
        assert!((agent.needs[Need::Fatigue] - expected).abs() <= TOLERANCE);
    }
}
