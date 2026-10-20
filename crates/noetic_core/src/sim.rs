//! One step of the simulation for each animal: it thinks (its needs rise and, while it
//! is choosing, it picks an activity) and it acts (it runs the step it is in).
use crate::agent::needs::update_needs;
use crate::agent::{Agent, AgentState};
use crate::model::Species;
use crate::steps::{do_activity, enter_step};
use crate::utility::choose_activity;
use crate::world::{SimWorld, WorldView, game_hours};

// ==== Config =================================================================

/// Game hours in one day; the clock wraps back to 0.0 when it reaches them.
const HOURS_PER_DAY: f32 = 24.0;

// ==== Systems ================================================================

/// API-COR-001 One step of the world for `dt` real seconds: the clock moves on
/// (PRC-01), then every animal in id order thinks and acts; an animal whose species is
/// missing stays idle (NFR-12). Every animal thinks on every step until `think_due`
/// (API-COR-023) joins; the returned events and the effects join with their code.
pub fn tick(world: &mut SimWorld, dt: f32) {
    let delta_hours = game_hours(&world.clock, dt);
    world.clock.hour = (world.clock.hour + delta_hours).rem_euclid(HOURS_PER_DAY);
    for agent in &mut world.agents {
        let Some(species) = world.species.get(&agent.species) else {
            continue;
        };
        think_agent(agent, species, &world.clock, dt);
        act_agent(agent, species, dt);
    }
}

/// API-COR-020 One animal thinks for `dt` real seconds: its needs rise by the game
/// hours they make, and while it is choosing it picks an activity and enters its first
/// step. Heal, perceive, memory, fear and the threat branch join in PRC order with
/// their code.
pub fn think_agent(agent: &mut Agent, species: &Species, view: &dyn WorldView, dt: f32) {
    let delta_hours = game_hours(view, dt);
    update_needs(agent, species, view, delta_hours);
    if agent.state == AgentState::Choosing
        && let Some((name, _scores)) = choose_activity(agent, species, view.hour())
        && let Some(activity) = species.activities.get(&name)
    {
        enter_step(agent, &name, activity, 0);
    }
}

/// API-COR-021 One animal acts for `dt` real seconds: it runs the step of its activity
/// (PRC-11). Following the group, fights, marks and locomotion join as PRC-12…15.
pub fn act_agent(agent: &mut Agent, species: &Species, dt: f32) {
    do_activity(agent, species, dt);
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::model::Need;
    use crate::world::Clock;

    const TOLERANCE: f32 = 1e-4;
    const RATE_PER_H: f32 = 10.0;
    const SECS_PER_GAME_HOUR: f32 = 60.0;
    /// Half an hour before midnight, so one game hour ends half an hour after it.
    const LATE_H: f32 = 23.5;
    const NOON_H: f32 = 12.0;

    /// A world at `hour` with one calm animal of the `test` species.
    fn world(hour: f32, secs_per_game_hour: f32) -> SimWorld {
        SimWorld {
            clock: Clock {
                hour,
                secs_per_game_hour,
            },
            agents: vec![Agent::calm()],
            species: BTreeMap::from([("test".to_string(), Species::diurnal(RATE_PER_H))]),
        }
    }

    // ---- tick ---------------------------------------------------------------

    /// UT-066 A game hour that passes midnight brings the clock back to the start of
    /// the day.
    #[test]
    fn the_clock_wraps_past_midnight() {
        let mut world = world(LATE_H, SECS_PER_GAME_HOUR);
        tick(&mut world, SECS_PER_GAME_HOUR);
        let expected = LATE_H + 1.0 - HOURS_PER_DAY;
        assert!((world.clock.hour - expected).abs() < TOLERANCE);
    }

    /// UT-067 An animal whose species is missing stays as it was, and the others still
    /// live on.
    #[test]
    fn an_animal_of_a_missing_species_stays_as_it_was() {
        let mut world = world(NOON_H, SECS_PER_GAME_HOUR);
        let mut lost = Agent::calm();
        lost.species = "missing".to_string();
        world.agents.push(lost.clone());
        tick(&mut world, SECS_PER_GAME_HOUR);
        assert_eq!(world.agents.get(1), Some(&lost));
        assert!(
            world
                .agents
                .first()
                .is_some_and(|agent| agent.needs[Need::Thirst] > 0.0)
        );
    }

    /// UT-068 A game hour of no length stops game time: the clock stays and the needs
    /// do not rise.
    #[test]
    fn a_game_hour_of_no_length_stops_game_time() {
        let mut world = world(NOON_H, 0.0);
        tick(&mut world, SECS_PER_GAME_HOUR);
        assert!((world.clock.hour - NOON_H).abs() < TOLERANCE);
        assert_eq!(world.agents.first(), Some(&Agent::calm()));
    }
}
