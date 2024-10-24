//! The Game State Machine Definition, Creates a Game Session with a number of enemies, a target

use std::collections::HashMap;

use enemies::{EnemyId, Freak};
use map::{Map, RoomId, RootRoomInfo};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use slotmap::SlotMap;
use wasm_bindgen::prelude::wasm_bindgen;

pub mod enemies;
pub mod map;

/// How much power a door being closed draws
pub const POWER_DRAW_DOOR: u32 = 500;
/// How much power is idly drawn
pub const DEFAULT_POWER_DRAW: u32 = 10;
/// How much power you start with
pub const INITIAL_POWER: u32 = 500_000;

/// The full driver for a game responsible for holding both the enemies and the game state
#[wasm_bindgen]
pub struct Game {
    /// All enemies that exist in the game
    enemies: SlotMap<EnemyId, Freak>,
    /// The actual game's state
    state: GameState,
    /// The random number generation
    rng: ThreadRng,
}

impl Default for Game {
    fn default() -> Self {
        let rng = thread_rng();
        let state = GameState::default();
        let enemies: SlotMap<EnemyId, Freak> = SlotMap::default();

        Self {
            enemies,
            state,
            rng,
        }
    }
}

impl Game {
    /// Ticks the game forward
    pub fn tick(&mut self) {
        self.state.tick(&mut self.enemies, &mut self.rng)
    }
}

/// The game's internal state, responsible for keeping track of what enemies we have, where they
/// are, if our doors are closed, what time it is, etc!
pub struct GameState {
    /// The currently registered cooldown times for each enemy
    cooldowns: HashMap<EnemyId, u64>,
    /// The current time
    ticks: u64,
    /// The map as graph-like structure
    pub map: Map,
    /// The office room
    pub office: RootRoomInfo,
    /// Where enemies can be spawned
    pub spawn_points: Vec<RoomId>,
    /// If the left door is closed
    left_door: bool,
    /// If the right door is closed
    right_door: bool,
    /// How much power is left
    power: u32,
    /// The current power draw (per tick)
    draw: u32,
    /// Are we dead?
    dead: bool,
}

impl Default for GameState {
    fn default() -> Self {
        let mut map = Map::default();
        let (office, spawn_points) = map.generate();

        GameState {
            cooldowns: HashMap::default(),
            ticks: 0,
            map,
            office,
            spawn_points,
            power: INITIAL_POWER,
            left_door: false,
            right_door: false,
            draw: DEFAULT_POWER_DRAW,
            dead: false,
        }
    }
}

/// A door's direction
pub enum Door {
    /// Left door
    Left,
    /// Right door
    Right,
}

impl GameState {
    /// Registers a collection of enemies into the map
    pub fn with_enemies<RNG: Rng>(mut self, enemies: &[EnemyId], rng: &mut RNG) -> Self {
        for enemy in enemies {
            let room = self.spawn_points.choose(rng);
            if let Some(room) = room {
                self.map.register_enemy(*enemy, *room);
            }
        }
        self
    }
    /// Ticks through all enemy behaviors if it's time
    pub fn tick<RNG: Rng>(&mut self, enemies: &mut SlotMap<EnemyId, Freak>, rng: &mut RNG) {
        self.ticks += 1;

        self.power -= self.draw;

        for (id, enemy) in enemies {
            if let Some(time) = self.cooldowns.get(&id) {
                if self.ticks % time == 0 {
                    // It's action time
                    enemy.tick(id, self);
                }
            } else {
                let new_cooldown = enemy.gen_cooldown(rng);
                self.cooldowns.insert(id, new_cooldown);
            }
        }

        if self.map.room_has_enemies(self.office.root) {
            self.dead = true
        }
    }

    /// Toggles if a door is open or closed, affecting power draw respectively
    pub fn toggle_door(&mut self, direction: Door) {
        let now_closed = match direction {
            Door::Left => {
                self.left_door = !self.left_door;
                !self.left_door
            }
            Door::Right => {
                self.right_door = !self.right_door;
                !self.right_door
            }
        };

        if now_closed {
            self.draw += POWER_DRAW_DOOR;
        } else {
            self.draw -= POWER_DRAW_DOOR;
        }
    }

    /// Moves an enemy from their current room to the desired room if possible
    pub(crate) fn move_enemy(&mut self, freak: EnemyId, to: RoomId) {
        let room = self.map.get_enemy_room(freak);
        if let Some(room) = room {
            self.map.move_enemy_out_of(room, freak)
        }
        self.map.move_enemy_to(to, freak);
    }

    /// Attacks with a given enemy if possible
    pub(crate) fn attack(&mut self, attacker: EnemyId) {
        if self.attack_possible(attacker) {
            self.move_enemy(attacker, self.office.root);
        }
    }

    /// Checks if an attack into the main office is possible currently for the given attacker
    fn attack_possible(&self, attacker: EnemyId) -> bool {
        if let Some(room) = self.map.get_enemy_room(attacker) {
            if room == self.office.left {
                !self.left_door
            } else if room == self.office.right {
                !self.right_door
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use slotmap::SlotMap;

    use crate::{enemies::Freak, GameState};

    #[test]
    fn default_enemy_behavior_comes_closer_to_office() {
        let mut rng = thread_rng();
        let mut enemy_map = SlotMap::default();

        let enemy_1 = enemy_map.insert(Freak::default_test_enemy());
        let enemy_2 = enemy_map.insert(Freak::default_test_enemy());
        let enemy_3 = enemy_map.insert(Freak::default_test_enemy());

        let mut game = GameState::default().with_enemies(&[enemy_1, enemy_2, enemy_3], &mut rng);

        for _ in 0..4 {
            game.map.display();
            game.tick(&mut enemy_map, &mut rng);
        }

        game.map.display();
    }
}
