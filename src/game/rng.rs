/*!
A small pseudo-random number generator.

The board needs somewhere to put the next piece of food, which calls for
randomness. `rand` pulls in more than this game needs, so a xorshift
generator does the job in a handful of instructions and no allocation.
*/

/**
A 32-bit xorshift pseudo-random number generator.

Fast and tiny, with a period long enough for placing food. Not suitable
for anything that depends on unpredictability.
*/
pub struct Rng {
    state: u32,
}

impl Rng {
    /**
    Creates a generator from a seed.

    A zero seed would leave the generator stuck at zero, so it is replaced
    by an arbitrary non-zero constant.
    */
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x2545_F491 } else { seed },
        }
    }

    /** Returns the next value in the sequence. */
    pub fn next_u32(&mut self) -> u32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        self.state
    }

    /**
    Returns a value in `0..bound`.

    # Panics

    Panics if `bound` is zero.
    */
    pub fn next_below(&mut self, bound: u32) -> u32 {
        assert!(bound > 0);
        self.next_u32() % bound
    }
}

#[cfg(test)]
mod tests {
    use super::Rng;

    #[test]
    fn zero_seed_still_produces_values() {
        let mut rng = Rng::new(0);
        assert_ne!(rng.next_u32(), 0);
    }

    #[test]
    fn values_stay_below_the_bound() {
        let mut rng = Rng::new(42);
        for _ in 0..1000 {
            assert!(rng.next_below(64) < 64);
        }
    }

    #[test]
    fn sequence_is_not_constant() {
        let mut rng = Rng::new(7);
        let first = rng.next_u32();
        assert!((0..100).any(|_| rng.next_u32() != first));
    }
}
