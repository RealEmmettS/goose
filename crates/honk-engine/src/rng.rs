//! Randomness: a pluggable source plus the `Deck` shuffle-bag.
//!
//! The original `Deck.Reshuffle()` is a **biased** shuffle, not a correct
//! Fisher–Yates: it draws the swap partner from `[0, j)` (low-bound 0, exclusive
//! high `j`) instead of `[0, j]`. We port that bias **faithfully** (locked decision
//! C8) and pin it with tests. The RNG *internals* below are clean-room (a SplitMix64,
//! not .NET's `System.Random`) — only the biased *algorithm* is reproduced, so emitted
//! sequences are deterministic per seed but do not match the original byte-for-byte.

/// A source of uniform random numbers in `[0, 1)`.
pub trait RandomSource {
    /// Next value in `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64;

    /// Faithful port of `SamMath.RandomRange`: `min + r * (max - min)`.
    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f64() as f32 * (max - min)
    }
}

/// A small, fast, fully deterministic PRNG (SplitMix64). Seedable for reproducible
/// tests; seed it from the OS in production via [`SplitMix64::from_entropy`].
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Seed the generator. Any `u64` is a valid seed.
    pub fn seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Seed from a non-deterministic OS-derived value (wall-clock nanos). Used by the
    /// running goose; never inside tests.
    pub fn from_entropy() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self::seed(nanos)
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl RandomSource for SplitMix64 {
    fn next_f64(&mut self) -> f64 {
        // Top 53 bits → a double in [0, 1), the standard construction.
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// A shuffle-bag over indices `0..len`: emits each index once before any repeats, then
/// reshuffles. Faithfully reproduces the original's biased shuffle.
#[derive(Debug, Clone)]
pub struct Deck<R: RandomSource> {
    indices: Vec<usize>,
    cursor: usize,
    rng: R,
}

impl<R: RandomSource> Deck<R> {
    /// Build a deck of `len` indices, shuffled with `rng`. `len` must be non-zero.
    pub fn new(len: usize, rng: R) -> Self {
        assert!(len > 0, "Deck length must be non-zero");
        let mut deck = Self {
            indices: vec![0; len],
            cursor: 0,
            rng,
        };
        deck.reshuffle();
        deck
    }

    /// Faithful port of the biased `Reshuffle()`.
    fn reshuffle(&mut self) {
        let len = self.indices.len();
        for j in 0..len {
            self.indices[j] = j;
            // faithful-to-original (biased shuffle, not Fisher–Yates): partner is drawn
            // from [0, j), so j == 0 is a no-op self-swap and low indices are favoured.
            let other = self.range_below(j);
            self.indices.swap(j, other);
        }
    }

    /// `(int)RandomRange(0, hi)` — uniform integer in `[0, hi)`, truncated toward zero.
    /// `hi == 0` yields `0` (the original's degenerate first iteration).
    fn range_below(&mut self, hi: usize) -> usize {
        if hi == 0 {
            return 0;
        }
        let v = self.rng.range(0.0, hi as f32) as usize;
        v.min(hi - 1) // guard the [0,1) endpoint against float round-up
    }

    /// Draw the next index from the bag; reshuffles transparently when exhausted.
    pub fn draw(&mut self) -> usize {
        let result = self.indices[self.cursor];
        self.cursor += 1;
        if self.cursor >= self.indices.len() {
            self.reshuffle();
            self.cursor = 0;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_f64_in_unit_interval() {
        let mut rng = SplitMix64::seed(1);
        for _ in 0..10_000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v), "out of range: {v}");
        }
    }

    #[test]
    fn deck_emits_each_index_once_per_cycle() {
        let mut deck = Deck::new(5, SplitMix64::seed(42));
        let mut seen = [0u32; 5];
        for _ in 0..5 {
            seen[deck.draw()] += 1;
        }
        assert_eq!(seen, [1, 1, 1, 1, 1], "a full cycle must be a permutation");
    }

    #[test]
    fn deck_sequence_is_pinned_for_seed() {
        // Regression wall: the exact biased order for a fixed seed. If the Deck
        // algorithm or RNG construction ever drifts, this changes.
        let mut deck = Deck::new(6, SplitMix64::seed(0xC0FFEE));
        let seq: Vec<usize> = (0..12).map(|_| deck.draw()).collect();
        assert_eq!(
            seq,
            vec![1, 4, 0, 5, 3, 2, 4, 0, 3, 5, 2, 1],
            "Deck sequence drifted from the pinned snapshot"
        );
    }

    #[test]
    fn shuffle_is_biased_not_uniform() {
        // Documents the known bias: over many deterministic reshuffles, the value
        // landing in slot 0 is NOT uniformly distributed. A correct Fisher–Yates would
        // give ~equal counts; the biased shuffle does not.
        const N: usize = 4;
        const TRIALS: usize = 120_000;
        let mut counts = [0usize; N];
        let mut deck = Deck::new(N, SplitMix64::seed(7));
        for _ in 0..TRIALS {
            // Drain a full cycle so each iteration starts from a fresh reshuffle.
            let first = deck.draw();
            counts[first] += 1;
            for _ in 1..N {
                deck.draw();
            }
        }
        let max = *counts.iter().max().unwrap();
        let min = *counts.iter().min().unwrap();
        let uniform = TRIALS / N;
        // The spread dwarfs the noise you'd see from a fair shuffle.
        assert!(
            max - min > uniform / 5,
            "expected a clearly non-uniform slot-0 distribution, got {counts:?}"
        );
    }
}
