//! One step of the simulation for each animal: it thinks (its needs rise and, while it
//! is choosing, it picks an activity) and it acts (it runs the step it is in).
use crate::agent::needs::update_needs;
use crate::agent::{Agent, AgentState};
use crate::model::Species;
use crate::steps::{do_activity, enter_step};
use crate::utility::choose_activity;
use crate::world::WorldView;

// ==== Systems ================================================================

/// API-COR-020 One animal thinks for `dt` real seconds: its needs rise by the game
/// hours they make, and while it is choosing it picks an activity and enters its first
/// step. Heal, perceive, memory, fear and the threat branch join in PRC order with
/// their code.
pub fn think_agent(agent: &mut Agent, species: &Species, view: &dyn WorldView, dt: f32) {
    // a game hour of no length means game time stands still, not a division by zero
    let secs = view.secs_per_game_hour();
    let delta_hours = if secs > 0.0 { dt / secs } else { 0.0 };
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
