/*! Coordinates on the 8x8 board. */

use super::direction::Direction;

/** Width and height of the board in cells. */
pub const BOARD_SIZE: u8 = 8;

/** Number of cells on the board. */
pub const CELL_COUNT: usize = (BOARD_SIZE as usize) * (BOARD_SIZE as usize);

/**
A cell on the board.

Both coordinates are in `0..BOARD_SIZE`, with the origin at the bottom left.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /** Horizontal coordinate, increasing to the right. */
    pub x: u8,
    /** Vertical coordinate, increasing upwards. */
    pub y: u8,
}

impl Position {
    /**
    Creates a position from its coordinates.

    # Panics

    Panics if either coordinate lies outside the board.
    */
    pub const fn new(x: u8, y: u8) -> Self {
        assert!(x < BOARD_SIZE);
        assert!(y < BOARD_SIZE);
        Self { x, y }
    }

    /**
    Returns the neighbouring position in `direction`, wrapping around the
    board edges.

    Wrapping keeps every move legal, so the snake can only die by biting
    itself. That makes the game forgiving enough to be fun on a board this
    small.
    */
    pub fn step(self, direction: Direction) -> Self {
        let last = BOARD_SIZE - 1;
        match direction {
            Direction::Up => Self::new(self.x, if self.y == last { 0 } else { self.y + 1 }),
            Direction::Down => Self::new(self.x, if self.y == 0 { last } else { self.y - 1 }),
            Direction::Left => Self::new(if self.x == 0 { last } else { self.x - 1 }, self.y),
            Direction::Right => Self::new(if self.x == last { 0 } else { self.x + 1 }, self.y),
        }
    }

    /** Returns the index of this position in a row-major array of cells. */
    pub fn index(self) -> usize {
        (self.y as usize) * (BOARD_SIZE as usize) + (self.x as usize)
    }

    /**
    Returns the position at `index` in a row-major array of cells.

    # Panics

    Panics if `index` is not a valid cell index.
    */
    pub fn from_index(index: usize) -> Self {
        assert!(index < CELL_COUNT);
        Self::new(
            (index % BOARD_SIZE as usize) as u8,
            (index / BOARD_SIZE as usize) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{BOARD_SIZE, CELL_COUNT, Position};
    use crate::game::direction::Direction;

    #[test]
    fn stepping_moves_one_cell() {
        let start = Position::new(3, 3);
        assert_eq!(start.step(Direction::Up), Position::new(3, 4));
        assert_eq!(start.step(Direction::Down), Position::new(3, 2));
        assert_eq!(start.step(Direction::Left), Position::new(2, 3));
        assert_eq!(start.step(Direction::Right), Position::new(4, 3));
    }

    #[test]
    fn stepping_wraps_at_the_edges() {
        let last = BOARD_SIZE - 1;
        assert_eq!(Position::new(0, last).step(Direction::Up), Position::new(0, 0));
        assert_eq!(Position::new(0, 0).step(Direction::Down), Position::new(0, last));
        assert_eq!(Position::new(0, 0).step(Direction::Left), Position::new(last, 0));
        assert_eq!(Position::new(last, 0).step(Direction::Right), Position::new(0, 0));
    }

    #[test]
    fn index_round_trips() {
        for index in 0..CELL_COUNT {
            assert_eq!(Position::from_index(index).index(), index);
        }
    }
}
