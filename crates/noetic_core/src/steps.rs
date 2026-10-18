//! How an animal does an activity, one step at a time: each step lasts a drawn time and
//! may lower a need while it runs (API-COR-011).
use crate::agent::{Agent, AgentState};
use crate::model::{Activity, Need, Species, Step, StepKind};

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

/// Puts the animal in step `index` of activity `act` with its time drawn from the
/// step's `secs` (the start of API-COR-011); past the last step the activity is over
/// and the animal goes back to choosing.
pub fn enter_step(agent: &mut Agent, act: &str, activity: &Activity, index: usize) {
    match activity.steps.get(index) {
        Some(step) => {
            let t_left = agent.rng.range(step.secs.0, step.secs.1);
            agent.state = AgentState::InActivity {
                act: act.to_string(),
                step: index,
                t_left,
            };
        }
        None => {
            agent.state = AgentState::Choosing;
        }
    }
}

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

/// What an animal in an activity does for `dt` seconds (PRC-11 of API-COR-021): runs
/// its current step, moves to the next one when it is done, and goes back to choosing
/// when the step fails or the activity is gone from the species.
pub fn do_activity(agent: &mut Agent, species: &Species, dt: f32) {
    let AgentState::InActivity { act, step, .. } = &agent.state else {
        return;
    };
    // own copies: run_step needs the whole agent, so nothing may still point into it
    let (act, index) = (act.clone(), *step);
    let Some(activity) = species.activities.get(&act) else {
        agent.state = AgentState::Choosing;
        return;
    };
    let Some(step) = activity.steps.get(index) else {
        agent.state = AgentState::Choosing;
        return;
    };
    match run_step(agent, step, dt) {
        StepStatus::Running => {}
        StepStatus::Done => enter_step(agent, &act, activity, index + 1),
        StepStatus::Failed => agent.state = AgentState::Choosing,
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

/// Whether the animal is resting now: in an activity whose current step is `Rest`
/// (ALG-02 keeps fatigue from rising then).
pub fn is_resting(agent: &Agent, species: &Species) -> bool {
    let AgentState::InActivity { act, step, .. } = &agent.state else {
        return false;
    };
    let Some(activity) = species.activities.get(act) else {
        return false;
    };
    let Some(current) = activity.steps.get(*step) else {
        return false;
    };
    current.do_ == StepKind::Rest
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Curve, GoToTarget};

    const TOLERANCE: f32 = 1e-4;
    /// How long the test step lasts.
    const STEP_S: f32 = 3.0;
    const HUNGER: f32 = 50.0;
    /// How long the first step of the test activity lasts.
    const EAT_SECS: (f32, f32) = (1.0, 2.0);
    /// A time longer than any step of the test activity, so each run finishes one.
    const LONG_S: f32 = 10.0;

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

    /// A step of `kind` that lasts exactly `STEP_S`, with no animation.
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

    // ---- enter_step and do_activity -----------------------------------------

    /// A `graze` activity whose first step is `first`, for `EAT_SECS`, and whose second
    /// is watching, and a species that offers it.
    fn grazing(first: StepKind) -> (Species, Activity) {
        let activity = Activity {
            need: Need::Hunger,
            curve: Curve::Linear,
            weight: 1.0,
            steps: vec![
                Step {
                    do_: first,
                    secs: EAT_SECS,
                    anim: None,
                },
                step(StepKind::Watch),
            ],
        };
        let mut species = Species::diurnal(0.0);
        species
            .activities
            .insert("graze".to_string(), activity.clone());
        (species, activity)
    }

    /// UT-062 Entering a step draws its time within the step's seconds; entering past
    /// the last step ends the activity.
    #[test]
    fn entering_a_step_draws_its_time_and_past_the_end_ends_the_activity() {
        let (_, activity) = grazing(StepKind::Eat);
        let mut agent = Agent::calm();
        enter_step(&mut agent, "graze", &activity, 0);
        let AgentState::InActivity { act, step, t_left } = &agent.state else {
            panic!("the animal is in the activity");
        };
        assert_eq!((act.as_str(), *step), ("graze", 0));
        assert!((EAT_SECS.0..EAT_SECS.1).contains(t_left), "t_left {t_left}");

        enter_step(&mut agent, "graze", &activity, activity.steps.len());
        assert_eq!(agent.state, AgentState::Choosing);
    }

    /// UT-063 An activity runs its steps in order and then goes back to choosing.
    #[test]
    fn an_activity_runs_its_steps_in_order_then_ends() {
        let (species, activity) = grazing(StepKind::Eat);
        let mut agent = Agent::calm();
        enter_step(&mut agent, "graze", &activity, 0);
        do_activity(&mut agent, &species, LONG_S);
        let on_step = |agent: &Agent| match &agent.state {
            AgentState::InActivity { step, .. } => Some(*step),
            AgentState::Choosing => None,
        };
        assert_eq!(on_step(&agent), Some(1));
        do_activity(&mut agent, &species, LONG_S);
        assert_eq!(agent.state, AgentState::Choosing);
    }

    /// UT-064 A step that fails drops the activity, and the animal chooses again.
    #[test]
    fn a_failed_step_drops_the_activity() {
        let (species, activity) = grazing(StepKind::GoTo(GoToTarget::Food));
        let mut agent = Agent::calm();
        enter_step(&mut agent, "graze", &activity, 0);
        do_activity(&mut agent, &species, 1.0);
        assert_eq!(agent.state, AgentState::Choosing);
    }
}
