//! Enemy behavior definition

use std::ops::Range;

use action::{Action, EnemyBehavior};
use rand::Rng;
use slotmap::new_key_type;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::GameState;

pub mod action;
pub mod impls;

new_key_type! {
    /// An enemy's ID for usage in the room HashMap
    #[wasm_bindgen]
    pub struct EnemyId;
}

/// An enemy and all of its related behavioral metadata
pub struct Freak {
    /// The enemy's name
    name: &'static str,
    /// The enemies state, whether they're on the move or not
    state: State,
    /// The (range in) amount of ticks before another behavior is performed (randomly selected each
    /// time)
    cooldown: Range<u64>,
    /// An enemies behavior is specific to them
    behavior: Box<dyn EnemyBehavior>,
}

impl Freak {
    /// Creates a new enemy
    pub fn new<BEHAVIOR: EnemyBehavior + 'static>(
        name: &'static str,
        cooldown: Range<u64>,
        behavior: BEHAVIOR,
    ) -> Self {
        Self {
            name,
            state: State::Dormant,
            cooldown,
            behavior: Box::new(behavior),
        }
    }

    #[cfg(test)]
    /// Creates a testable default enemy type with the generic pathfinding behavior
    pub fn default_test_enemy() -> Self {
        use impls::generic::StraightPathBehavior;

        Self {
            name: "Test Entity",
            state: State::Dormant,
            cooldown: 1..5,
            behavior: Box::new(StraightPathBehavior::default()),
        }
    }

    /// Returns the enemy's name
    pub fn get_name(&self) -> &str {
        self.name
    }

    /// When it's an enemies turn, goes through all of the logic and game mutation given that
    /// generated Action
    pub fn tick(&mut self, id: EnemyId, curr_game: &mut GameState) {
        match self.state {
            State::Dormant => {
                // Just wake up

                self.state = State::Moving
            }
            State::Moving => {
                // Begin performing actions

                let actions = self.behavior.tick(curr_game, id);

                for action in actions {
                    match action {
                        Action::Move(move_to) => curr_game.move_enemy(id, move_to),
                        Action::Attack => curr_game.attack(id),
                        Action::Special(side_effect) => side_effect.do_something(curr_game),
                        Action::Nothing => {}
                    }
                }
            }
        }
    }

    /// Given the enemies range of cooldown times, returns one of them randomly
    pub fn gen_cooldown<RNG: Rng>(&self, rng: &mut RNG) -> u64 {
        rng.gen_range(self.cooldown.clone())
    }
}

/// Whether the enemy is activated or not
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Enemy is currently asleep
    Dormant,
    /// Enemy is on the move
    Moving,
}
