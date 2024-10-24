//! Enemy behavior trait definition

use crate::{GameState, RoomId};

use super::EnemyId;

/// The behavior of an enemy that happens each tick of the game
pub trait EnemyBehavior {
    /// Given the current game's context, performs an arbitrary amount of actions
    fn tick(&mut self, curr_state: &GameState, id: EnemyId) -> Vec<Action>;
}

/// All different actions an enemy can do in a turn
pub enum Action {
    /// Move to room `RoomId`
    Move(RoomId),
    /// Perform some special action specific to the enemy
    Special(Box<dyn SideEffect>),
    /// Attack the player if close enough
    Attack,
    /// Or do nothing
    Nothing,
}

/// A generic side effect that an enemy may cause to the game state. An example could be disabling
/// cameras, turning back time, spawn a friend, etc..
pub trait SideEffect {
    /// Do something to a mutable game state
    fn do_something(&self, game: &mut GameState);
}
