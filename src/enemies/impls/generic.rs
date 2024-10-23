//! A generic enemy implementation

use crate::enemies::action::{Action, EnemyBehavior};

/// The most generic enemy behavior possible, attempt to advance towards the player and if they're
/// only 1 room away attempt to attack
#[derive(Clone, Copy)]
pub struct GenericBehavior;

impl EnemyBehavior for GenericBehavior {
    fn tick(&mut self, _curr_state: &crate::GameState) -> Action {
        todo!("Flesh out behavior")
    }
}
