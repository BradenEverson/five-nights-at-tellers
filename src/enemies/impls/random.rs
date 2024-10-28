//! An enemy implementation that randomly decides where to go each round
use rand::{seq::SliceRandom, Rng};

use crate::{
    enemies::{
        action::{Action, EnemyBehavior},
        EnemyId,
    },
    GameState,
};

/// Every turn, the enemy chooses a random path from the current room it's in
pub struct RandomBehavior<RNG: Rng> {
    /// Rng used for choosing a room each time
    rng: RNG,
}

impl<RNG: Rng> RandomBehavior<RNG> {
    /// Creates a new behavior based on existing Rng
    pub fn new(rng: RNG) -> Self {
        Self { rng }
    }
}

impl<RNG: Rng> EnemyBehavior for RandomBehavior<RNG> {
    fn tick(&mut self, curr_state: &GameState, id: EnemyId) -> Vec<Action> {
        if let Some(enemy_room) = curr_state.map.get_enemy_room(id) {
            let rooms = curr_state.map.0[enemy_room].connections();
            let goto = rooms.choose(&mut self.rng).unwrap();

            if goto == &curr_state.office.root {
                vec![Action::Attack]
            } else {
                vec![Action::Move(*goto)]
            }
        } else {
            vec![Action::Nothing]
        }
    }
}
