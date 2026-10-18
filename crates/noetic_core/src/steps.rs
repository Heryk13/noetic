//! How an animal does an activity, one step at a time: each step lasts a drawn time and
//! may lower a need while it runs (API-COR-011).
use crate::agent::{Agent, AgentState};
use crate::model::{Need, Step, StepKind};

// ==== Config =================================================================

// how fast each step lowers its need per second while it runs, from the prototype
/// Hunger an `Eat` step takes off per second.
const EAT_HUNGER_DROP_PER_S: f32 = 14.0;
/// Thirst a `Drink` step takes off per second.
const DRINK_THIRST_DROP_PER_S: f32 = 20.0;
/// Fatigue a `Rest` step takes off per second.
const REST_FATIGUE_DROP_PER_S: f32 = 9.0;
/// Heat a `Wallow` step takes off per second.
const WALLOW_HEAT_DROP_PER_S: f32 = 14.0;
/// Urge to sing a `Sing` step takes off per second.
const SING_URGE_DROP_PER_S: f32 = 20.0;

// ==== Steps ==================================================================

/// What one run of a step tells the caller: keep at it, move to the next step, or
/// drop the activity and choose again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Running,
    Done,
    Failed,
}

// ==== Systems ================================================================

/// API-COR-011 Runs one step of the activity the animal is in for `dt` seconds: lowers
/// the step's need, counts its time down and says whether it is over. A `GoTo` fails
/// until the world can tell where things are, and so does a step run outside an
/// activity; nothing changes on a failure.
pub fn run_step(agent: &mut Agent, step: &Step, dt: f32) -> StepStatus {
    if let StepKind::GoTo(_) = step.do_ {
        return StepStatus::Failed;
    }
    let AgentState::InActivity { t_left, .. } = &mut agent.state else {
        return StepStatus::Failed;
    };
    if let Some((need, per_s)) = relief(step.do_) {
        agent.needs[need] -= per_s * dt;
    }
    *t_left -= dt;
    if *t_left > 0.0 {
        StepStatus::Running
    } else {
        StepStatus::Done
    }
}

// ==== Helpers ================================================================

/// The need a step lowers while it runs and by how much per second; `None` for a step
/// that lowers none.
pub fn relief(kind: StepKind) -> Option<(Need, f32)> {
    match kind {
        StepKind::Eat => Some((Need::Hunger, EAT_HUNGER_DROP_PER_S)),
        StepKind::Drink => Some((Need::Thirst, DRINK_THIRST_DROP_PER_S)),
        StepKind::Rest => Some((Need::Fatigue, REST_FATIGUE_DROP_PER_S)),
        StepKind::Wallow => Some((Need::Heat, WALLOW_HEAT_DROP_PER_S)),
        StepKind::Sing => Some((Need::Sing, SING_URGE_DROP_PER_S)),
        StepKind::Wander
        | StepKind::GoTo(_)
        | StepKind::Sniff
        | StepKind::Watch
        | StepKind::Groom
        | StepKind::Dig
        | StepKind::Mark
        | StepKind::Rub
        | StepKind::Fight
        | StepKind::Perch => None,
    }
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::GoToTarget;

    const TOLERANCE: f32 = 1e-4;
    /// How long the test step lasts.
    const STEP_S: f32 = 3.0;
    const HUNGER: f32 = 50.0;

    /// A hungry animal in the first step of an activity, with `STEP_S` to go.
    fn eating() -> Agent {
        let mut agent = Agent::calm();
        agent.needs[Need::Hunger] = HUNGER;
        agent.state = AgentState::InActivity {
            act: "graze".to_string(),
            step: 0,
            t_left: STEP_S,
        };
        agent
    }

    fn step(kind: StepKind) -> Step {
        Step {
            do_: kind,
            secs: (STEP_S, STEP_S),
            anim: None,
        }
    }

    // ---- run_step -----------------------------------------------------------

    /// UT-059 While an eating step runs, hunger drops at its rate and the step goes on.
    #[test]
    fn eating_lowers_hunger_while_the_step_runs() {
        let mut agent = eating();
        let dt = 1.0;
        assert_eq!(
            run_step(&mut agent, &step(StepKind::Eat), dt),
            StepStatus::Running
        );
        let expected = HUNGER - EAT_HUNGER_DROP_PER_S * dt;
        assert!((agent.needs[Need::Hunger] - expected).abs() <= TOLERANCE);
    }

    /// UT-060 A step is done once its time is over.
    #[test]
    fn a_step_is_done_when_its_time_is_over() {
        let mut agent = eating();
        // two runs of 2/3 of the step each: the first leaves time, the second passes it
        let dt = STEP_S * 2.0 / 3.0;
        assert_eq!(
            run_step(&mut agent, &step(StepKind::Eat), dt),
            StepStatus::Running
        );
        assert_eq!(
            run_step(&mut agent, &step(StepKind::Eat), dt),
            StepStatus::Done
        );
    }

    /// UT-061 A step that cannot happen fails and changes nothing: a `GoTo` with no map
    /// yet, or any step while the animal is not in an activity.
    #[test]
    fn a_failed_step_changes_nothing() {
        let mut agent = eating();
        let before = agent.clone();
        let go_eat = step(StepKind::GoTo(GoToTarget::Food));
        assert_eq!(run_step(&mut agent, &go_eat, 1.0), StepStatus::Failed);
        assert_eq!(agent, before);

        let mut choosing = Agent::calm();
        choosing.needs[Need::Hunger] = HUNGER;
        let before = choosing.clone();
        assert_eq!(
            run_step(&mut choosing, &step(StepKind::Eat), 1.0),
            StepStatus::Failed
        );
        assert_eq!(choosing, before);
    }
}
