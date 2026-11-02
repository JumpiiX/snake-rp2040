//! Movement directions on the board.

/// A direction the snake can travel in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Towards increasing `y`.
    Up,
    /// Towards decreasing `y`.
    Down,
    /// Towards decreasing `x`.
    Left,
    /// Towards increasing `x`.
    Right,
}

impl Direction {
    /// Returns the direction pointing the opposite way.
    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    /// Returns whether a turn into `other` would reverse the snake onto itself.
    ///
    /// A snake of length two or more cannot turn back into its own neck, so
    /// such a turn is rejected rather than ending the game.
    pub fn is_reversal_of(self, other: Self) -> bool {
        self == other.opposite()
    }
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn opposite_is_symmetric() {
        for direction in [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ] {
            assert_eq!(direction.opposite().opposite(), direction);
        }
    }

    #[test]
    fn opposite_directions_are_reversals() {
        assert!(Direction::Up.is_reversal_of(Direction::Down));
        assert!(Direction::Left.is_reversal_of(Direction::Right));
    }

    #[test]
    fn perpendicular_directions_are_not_reversals() {
        assert!(!Direction::Up.is_reversal_of(Direction::Left));
        assert!(!Direction::Up.is_reversal_of(Direction::Up));
    }
}
