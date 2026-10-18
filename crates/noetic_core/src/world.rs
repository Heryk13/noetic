//! What the core asks of the world an animal lives in. The editor answers it from its
//! `SimWorld` and a game from its own scene, so the core never depends on either.

// ==== World View =============================================================

/// TYP-14 The read-only questions an animal asks its world; only the hour and the
/// length of a game hour so far, the rest of TYP-14 joins as the code that asks it does.
pub trait WorldView {
    /// The hour of the game day, from 0.0 up to 24.0.
    fn hour(&self) -> f32;
    /// How many real seconds one game hour lasts, to turn a step's seconds into the
    /// game hours the needs rise by.
    fn secs_per_game_hour(&self) -> f32;
}
