//! The Game State Machine Definition, Creates a Game Session with a number of enemies, a target
//! "time" to survive to, how many ticks create this time, and current door metadata

use enemies::Freak;

pub mod enemies;

/// A room's ID
pub type RoomId = usize;

/// The game's internal state, responsible for keeping track of what enemies we have, where they
/// are, if our doors are closed, what time it is, etc!
pub struct GameState {
    /// All enemies that exist in this game
    enemies: Vec<Freak>
}
