/*!
Snake game rules, independent of any hardware.

Nothing in this module touches a peripheral, which keeps the rules
testable on the host and lets the same logic back all three binaries.
*/

pub mod direction;
pub mod position;
pub mod rng;
pub mod snake;

use direction::Direction;
use position::{CELL_COUNT, Position};
use rng::Rng;
use snake::Snake;

/** Where the snake starts. */
const START: Position = Position::new(2, 4);

/** Which way the snake starts out travelling. */
const START_HEADING: Direction = Direction::Right;

/** Whether a tick left the game running or ended it. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /** The snake moved and the game continues. */
    Moved,
    /** The snake ate and the game continues. */
    Ate,
    /** The snake bit itself and the game is over. */
    Died,
    /** The snake filled the board and the game is won. */
    Won,
}

/** A game in progress. */
pub struct Game {
    snake: Snake,
    food: Position,
    rng: Rng,
    score: u16,
    over: bool,
}

impl Game {
    /**
    Starts a game seeded with `seed`.

    The seed decides where food appears; derive it from something that
    varies between runs, such as how long the player took to press start.
    */
    pub fn new(seed: u32) -> Self {
        let snake = Snake::new(START, START_HEADING);
        let mut rng = Rng::new(seed);
        let food = Self::place_food(&snake, &mut rng);
        Self {
            snake,
            food,
            rng,
            score: 0,
            over: false,
        }
    }

    /** Returns the snake. */
    pub fn snake(&self) -> &Snake {
        &self.snake
    }

    /** Returns the position of the food. */
    pub fn food(&self) -> Position {
        self.food
    }

    /** Returns how many pieces of food the snake has eaten. */
    pub fn score(&self) -> u16 {
        self.score
    }

    /** Returns whether the game has ended. */
    pub fn is_over(&self) -> bool {
        self.over
    }

    /**
    Points the snake in `direction` for the next tick.

    Turns that would reverse the snake onto its own neck are ignored, and
    steering a finished game does nothing.
    */
    pub fn steer(&mut self, direction: Direction) {
        if !self.over {
            self.snake.steer(direction);
        }
    }

    /** Advances the game by one step. */
    pub fn tick(&mut self) -> Outcome {
        if self.over {
            return Outcome::Died;
        }

        let eating = self.snake.next_head() == self.food;

        if self.snake.would_collide(eating) {
            self.over = true;
            return Outcome::Died;
        }

        self.snake.advance(eating);

        if !eating {
            return Outcome::Moved;
        }

        self.score += 1;

        if self.snake.len() == CELL_COUNT {
            self.over = true;
            return Outcome::Won;
        }

        self.food = Self::place_food(&self.snake, &mut self.rng);
        Outcome::Ate
    }

    /**
    Picks a free cell for the next piece of food.

    Rejection sampling would stall as the board fills up, so this walks
    forward from a random cell to the next free one instead. That is
    slightly biased towards cells just after a run of body segments, which
    no player will ever notice.

    # Panics

    Panics if the board has no free cell. Callers must check for a won game
    before asking for more food.
    */
    fn place_food(snake: &Snake, rng: &mut Rng) -> Position {
        let start = rng.next_below(CELL_COUNT as u32) as usize;
        for offset in 0..CELL_COUNT {
            let candidate = Position::from_index((start + offset) % CELL_COUNT);
            if !snake.occupies(candidate) {
                return candidate;
            }
        }
        panic!("no free cell for food");
    }
}

#[cfg(test)]
mod tests {
    use super::{Game, Outcome};
    use crate::game::direction::Direction;

    #[test]
    fn a_new_game_is_running() {
        let game = Game::new(1);
        assert!(!game.is_over());
        assert_eq!(game.score(), 0);
    }

    #[test]
    fn food_never_starts_under_the_snake() {
        for seed in 1..200 {
            let game = Game::new(seed);
            assert!(!game.snake().occupies(game.food()));
        }
    }

    #[test]
    fn a_plain_tick_only_moves() {
        let mut game = Game::new(1);
        let before = game.snake().len();
        while game.tick() == Outcome::Moved {
            assert_eq!(game.snake().len(), before);
            break;
        }
    }

    #[test]
    fn eating_scores_and_grows() {
        let mut game = Game::new(1);
        let length = game.snake().len();
        for _ in 0..200 {
            if steer_towards_food(&mut game) == Outcome::Ate {
                assert_eq!(game.score(), 1);
                assert_eq!(game.snake().len(), length + 1);
                return;
            }
        }
        panic!("never reached the food");
    }

    #[test]
    fn eating_moves_the_food() {
        let mut game = Game::new(1);
        let food = game.food();
        for _ in 0..200 {
            if steer_towards_food(&mut game) == Outcome::Ate {
                assert_ne!(game.food(), food);
                return;
            }
        }
        panic!("never reached the food");
    }

    #[test]
    fn a_finished_game_stays_finished() {
        let mut game = Game::new(1);
        while !game.is_over() {
            game.tick();
        }
        assert_eq!(game.tick(), Outcome::Died);
    }

    fn steer_towards_food(game: &mut Game) -> Outcome {
        let head = game.snake().head();
        let food = game.food();
        if head.y != food.y {
            let up = (food.y + 8 - head.y) % 8 <= 4;
            game.steer(if up { Direction::Up } else { Direction::Down });
        } else if head.x != food.x {
            let right = (food.x + 8 - head.x) % 8 <= 4;
            game.steer(if right { Direction::Right } else { Direction::Left });
        }
        game.tick()
    }
}
