//! The needs of one animal and how they rise with time, the hour and the place (ALG-02).
use enum_map::EnumMap;

use crate::agent::Agent;
use crate::model::{Need, Species, Trait};

// ==== Config =================================================================

/// Hunger factor of an animal with appetite 0; appetite 50 brings it to 1.0 (ALG-02).
const HUNGER_FACTOR_MIN: f32 = 0.6;
/// Appetite points that add 1.0 to the hunger factor (ALG-02).
const APPETITE_PER_HUNGER_FACTOR: f32 = 125.0;
/// Lowest value of any need.
const NEED_MIN: f32 = 0.0;
/// Highest value of any need.
const NEED_MAX: f32 = 100.0;

// ==== Needs ==================================================================

/// The current value of every need of one animal, 0–100, read by `Need`.
pub type Needs = EnumMap<Need, f32>;

// ==== Systems ================================================================

/// API-COR-002 Raises hunger and thirst with the game hours that passed; hunger also
/// follows the animal's appetite.
pub fn update_needs(agent: &mut Agent, species: &Species, delta_hours: f32) {
    agent.needs[Need::Thirst] += species.needs.thirst_per_h * delta_hours;
    let appetite = f32::from(agent.traits[Trait::Appetite]);
    agent.needs[Need::Hunger] += species.needs.hunger_per_h
        * (HUNGER_FACTOR_MIN + appetite / APPETITE_PER_HUNGER_FACTOR)
        * delta_hours;
    for v in agent.needs.values_mut() {
        *v = v.clamp(NEED_MIN, NEED_MAX);
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::TraitMap;
    use crate::model::NeedRates;

    const RATE_PER_H: f32 = 10.0;
    const TOLERANCE: f32 = 1e-4;

    fn calm_agent() -> Agent {
        Agent {
            needs: Needs::default(),
            traits: TraitMap::default(),
            limping: false,
        }
    }

    fn species() -> Species {
        Species {
            needs: NeedRates {
                hunger_per_h: RATE_PER_H,
                thirst_per_h: RATE_PER_H,
                fatigue_per_h: RATE_PER_H,
                fear_threshold: NEED_MAX / 2.0,
            },
        }
    }

    // ---- update_needs -------------------------------------------------------

    /// UT-039 Thirst rises by the species rate for every game hour that passes.
    #[test]
    fn thirst_rises_with_the_hours() {
        let mut agent = calm_agent();
        let hours = 2.0;
        update_needs(&mut agent, &species(), hours);
        let expected = RATE_PER_H * hours;
        assert!((agent.needs[Need::Thirst] - expected).abs() <= TOLERANCE);
    }

    /// UT-040 An animal of average appetite gets hungry exactly at the species rate.
    #[test]
    fn average_appetite_keeps_the_species_hunger_rate() {
        let mut agent = calm_agent();
        // the appetite whose factor is exactly 1.0
        let average = (1.0 - HUNGER_FACTOR_MIN) * APPETITE_PER_HUNGER_FACTOR;
        agent.traits[Trait::Appetite] = average.round() as u8;
        update_needs(&mut agent, &species(), 1.0);
        assert!((agent.needs[Need::Hunger] - RATE_PER_H).abs() <= TOLERANCE);
    }

    /// UT-041 A need never goes past its maximum, however long the animal waits.
    #[test]
    fn needs_never_pass_their_maximum() {
        let mut agent = calm_agent();
        agent.needs[Need::Hunger] = NEED_MAX - RATE_PER_H / 2.0;
        update_needs(&mut agent, &species(), 10.0);
        assert!((agent.needs[Need::Hunger] - NEED_MAX).abs() <= TOLERANCE);
        assert!((agent.needs[Need::Thirst] - NEED_MAX).abs() <= TOLERANCE);
    }
}
