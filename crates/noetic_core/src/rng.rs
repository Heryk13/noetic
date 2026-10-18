//! The seeded PRNG that every random choice in the core goes through (NFR-01), and the
//! weighted pick built on it.
use rand::Rng as _;
use rand::RngExt;
use rand::SeedableRng;
use rand::seq::IndexedRandom;
use rand_pcg::Pcg32;

// ==== Rng ====================================================================

/// API-COR-017 A seeded PRNG: the same seed always gives the same sequence, so a
/// cloned agent keeps drawing the same numbers as the original.
#[derive(Debug, Clone, PartialEq)]
pub struct Rng(Pcg32);

impl Rng {
    /// Starts the sequence from `seed`, spreading its bits so that nearby seeds
    /// still give unrelated sequences.
    pub fn seed(seed: u64) -> Self {
        Self(Pcg32::seed_from_u64(seed))
    }

    /// Draws the next number of the sequence, any `u32` equally likely.
    pub fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    /// Draws the next number of the sequence, any `u64` equally likely.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Draws a number in [0, 1), never 1.0 itself.
    pub fn unit(&mut self) -> f32 {
        self.0.random()
    }

    /// Draws a number between `min` and `max`, any value equally likely; `min` when
    /// the range is empty.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        // random_range panics on an empty range, and the core never panics (EP-01)
        if max <= min {
            return min;
        }
        self.0.random_range(min..max)
    }

    /// Draws a whole number from 0 up to `n - 1`, never `n` itself; 0 when `n` is 0.
    pub fn below(&mut self, n: u32) -> u32 {
        // random_range panics on an empty range, and the core never panics (EP-01)
        if n == 0 {
            return 0;
        }
        self.0.random_range(0..n)
    }
}

// ==== Weighted Pick ==========================================================

/// API-COR-007 Picks one item, each as likely as its weight; `None` when the list
/// is empty or every weight is zero.
pub fn weighted_pick<'a, T>(items: &'a [(T, u8)], rng: &mut Rng) -> Option<&'a T> {
    // summed as u32: two u8 weights over 255 would overflow and fail the pick
    items
        .choose_weighted(&mut rng.0, |(_, w)| u32::from(*w))
        .ok()
        .map(|(item, _)| item)
}

// ==== Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- weighted_pick ------------------------------------------------------

    /// UT-001 A 60/40 split comes out within 1 % of 60/40 over 100 000 picks.
    #[test]
    fn picks_follow_their_weights() {
        const PICKS: u32 = 100_000;
        const WEIGHT_A: u8 = 60;
        const TOLERANCE: f32 = 0.01;
        let items = [("a", WEIGHT_A), ("b", 100 - WEIGHT_A)];
        let mut rng = Rng::seed(1);
        let a_count = (0..PICKS)
            .filter(|_| weighted_pick(&items, &mut rng) == Some(&"a"))
            .count();
        let a_share = a_count as f32 / PICKS as f32;
        let expected = f32::from(WEIGHT_A) / 100.0;
        assert!(
            (a_share - expected).abs() <= TOLERANCE,
            "a came out {a_share}, expected {expected} ± {TOLERANCE}"
        );
    }

    /// UT-002 A lone item with weight is the only possible pick.
    #[test]
    fn a_lone_item_is_always_picked() {
        let items = [("a", 100)];
        let mut rng = Rng::seed(1);
        for _ in 0..1000 {
            assert_eq!(weighted_pick(&items, &mut rng), Some(&"a"));
        }
    }

    /// UT-003 Nothing to pick gives None instead of a panic.
    #[test]
    fn an_empty_or_weightless_list_picks_nothing() {
        let mut rng = Rng::seed(1);
        let empty: [(&str, u8); 0] = [];
        assert_eq!(weighted_pick(&empty, &mut rng), None);
        assert_eq!(weighted_pick(&[("a", 0), ("b", 0)], &mut rng), None);
    }
}
