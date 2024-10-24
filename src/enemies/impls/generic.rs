//! A generic enemy implementation that simply goes straight to the player as fast as possible

use crate::{
    enemies::{
        action::{Action, EnemyBehavior},
        EnemyId,
    },
    map::RoomId,
};

/// The most generic enemy behavior possible, attempt to advance towards the player and if they're
/// only 1 room away attempt to attack
#[derive(Clone, Default)]
pub struct StraightPathBehavior {
    /// The ideal
    ideal_path: Option<Vec<RoomId>>,
}

impl EnemyBehavior for StraightPathBehavior {
    fn tick(&mut self, curr_state: &crate::GameState, id: EnemyId) -> Action {
        if self.ideal_path.is_none() {
            let enemy_room = curr_state.map.get_enemy_room(id);
            if enemy_room.is_none() {
                return Action::Nothing;
            }
            let ideal_path = curr_state
                .map
                .generate_path(enemy_room.unwrap(), curr_state.office.root);
            if let Some(path) = ideal_path {
                self.ideal_path = Some(path);
            } else {
                return Action::Nothing;
            }
        };

        // Safety: Here we are 100% sure ideal path is some, we generate a path for it if it
        // doesn't already exist right above here
        let path = self.ideal_path.clone().unwrap();
        let current_location = curr_state.map.get_enemy_room(id).unwrap();

        let idx = path.iter().position(|room| *room == current_location);
        if let Some(idx) = idx {
            if idx >= path.len() - 2 {
                return Action::Attack
            } else {
                return Action::Move(path[idx + 1])
            }
        }

        return Action::Nothing
    }
}
