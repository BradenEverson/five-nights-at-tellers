//! A behavior implementation that quickly performs two actions each time

use crate::{
    enemies::{
        action::{Action, EnemyBehavior},
        EnemyId,
    },
    GameState,
};

/// A behavior that performs the same action twice in a row
#[derive(Default)]
pub struct DoubleBehavior<BEHAVIOR: EnemyBehavior> {
    /// The ideal
    inner_behavior: BEHAVIOR,
}

impl<BEHAVIOR: EnemyBehavior> DoubleBehavior<BEHAVIOR> {
    /// Creates a new double behavior entity
    pub fn new(inner_behavior: BEHAVIOR) -> Self {
        Self { inner_behavior }
    }
}

impl<BEHAVIOR: EnemyBehavior> EnemyBehavior for DoubleBehavior<BEHAVIOR> {
    fn tick(&mut self, curr_state: &GameState, id: EnemyId) -> Vec<Action> {
        let mut action_1 = self.inner_behavior.tick(curr_state, id);
        if action_1.contains(&Action::Attack) {
            action_1
        } else {
            let action_2 = self.inner_behavior.tick(curr_state, id);
            if action_2.contains(&Action::Attack) {
                action_1
            } else {
                action_1.extend(action_2);
                action_1
            }
        }
    }
}
