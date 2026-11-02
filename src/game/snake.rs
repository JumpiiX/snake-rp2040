//! The snake body.

use heapless::Deque;

use super::direction::Direction;
use super::position::{CELL_COUNT, Position};

/// Number of segments the snake starts with.
const INITIAL_LENGTH: usize = 3;

/// The snake, stored head-first.
///
/// Capacity is the whole board, so growing can never overflow: a snake that
/// filled every cell would have won the game already.
pub struct Snake {
    segments: Deque<Position, CELL_COUNT>,
    heading: Direction,
}

impl Snake {
    /// Creates a snake of [`INITIAL_LENGTH`] segments with its head at `head`,
    /// extending backwards against `heading`.
    ///
    /// # Panics
    ///
    /// Panics if the body would run off the board.
    pub fn new(head: Position, heading: Direction) -> Self {
        let mut segments = Deque::new();
        let mut cell = head;
        for _ in 0..INITIAL_LENGTH {
            segments.push_back(cell).ok().unwrap();
            cell = cell.step(heading.opposite());
        }
        Self { segments, heading }
    }

    /// Returns the position of the head.
    pub fn head(&self) -> Position {
        *self.segments.front().unwrap()
    }

    /// Returns the direction the snake is currently travelling in.
    pub fn heading(&self) -> Direction {
        self.heading
    }

    /// Returns the number of segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns whether the snake has no segments.
    ///
    /// Always false in practice; present to satisfy the companion to
    /// [`len`](Self::len).
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns an iterator over the segments, head first.
    pub fn segments(&self) -> impl Iterator<Item = Position> + '_ {
        self.segments.iter().copied()
    }

    /// Returns whether any segment covers `position`.
    pub fn occupies(&self, position: Position) -> bool {
        self.segments().any(|segment| segment == position)
    }

    /// Turns the snake, unless the turn would reverse it onto its own neck.
    pub fn steer(&mut self, direction: Direction) {
        if !direction.is_reversal_of(self.heading) {
            self.heading = direction;
        }
    }

    /// Returns the cell the head would move into on the next advance.
    pub fn next_head(&self) -> Position {
        self.head().step(self.heading)
    }

    /// Moves the snake one cell forward.
    ///
    /// When `grow` is set the tail stays put and the snake gets one segment
    /// longer; otherwise the tail follows the head.
    pub fn advance(&mut self, grow: bool) {
        let head = self.next_head();
        self.segments.push_front(head).ok().unwrap();
        if !grow {
            self.segments.pop_back();
        }
    }

    /// Returns whether the head would hit the body on the next advance.
    ///
    /// The tail is excluded because it moves out of the way in the same tick,
    /// unless the snake is about to grow into it.
    pub fn would_collide(&self, grow: bool) -> bool {
        let next = self.next_head();
        let body_len = if grow {
            self.segments.len()
        } else {
            self.segments.len() - 1
        };
        self.segments().take(body_len).any(|segment| segment == next)
    }
}

#[cfg(test)]
mod tests {
    use super::{INITIAL_LENGTH, Snake};
    use crate::game::direction::Direction;
    use crate::game::position::Position;

    fn snake() -> Snake {
        Snake::new(Position::new(4, 4), Direction::Right)
    }

    #[test]
    fn starts_at_the_initial_length() {
        assert_eq!(snake().len(), INITIAL_LENGTH);
    }

    #[test]
    fn body_trails_behind_the_head() {
        let snake = snake();
        let body: heapless::Vec<_, 8> = snake.segments().collect();
        assert_eq!(body[0], Position::new(4, 4));
        assert_eq!(body[1], Position::new(3, 4));
        assert_eq!(body[2], Position::new(2, 4));
    }

    #[test]
    fn advancing_moves_the_head_and_keeps_the_length() {
        let mut snake = snake();
        snake.advance(false);
        assert_eq!(snake.head(), Position::new(5, 4));
        assert_eq!(snake.len(), INITIAL_LENGTH);
    }

    #[test]
    fn growing_keeps_the_tail_in_place() {
        let mut snake = snake();
        snake.advance(true);
        assert_eq!(snake.len(), INITIAL_LENGTH + 1);
        assert!(snake.occupies(Position::new(2, 4)));
    }

    #[test]
    fn steering_sideways_is_accepted() {
        let mut snake = snake();
        snake.steer(Direction::Up);
        assert_eq!(snake.heading(), Direction::Up);
    }

    #[test]
    fn steering_into_the_neck_is_ignored() {
        let mut snake = snake();
        snake.steer(Direction::Left);
        assert_eq!(snake.heading(), Direction::Right);
    }

    #[test]
    fn running_into_the_body_is_a_collision() {
        let mut snake = snake();
        snake.advance(true);
        snake.advance(true);
        snake.steer(Direction::Up);
        snake.advance(true);
        snake.steer(Direction::Left);
        snake.advance(true);
        snake.steer(Direction::Down);
        assert!(snake.would_collide(false));
    }

    #[test]
    fn following_the_tail_is_not_a_collision() {
        let snake = snake();
        assert!(!snake.would_collide(false));
    }
}
