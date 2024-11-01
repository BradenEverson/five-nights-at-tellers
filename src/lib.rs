//! The Game State Machine Definition, Creates a Game Session with a number of enemies, a target
//! time to aim for, an amount of ticks to reach that time, power information, and door states

use std::collections::HashMap;

use enemies::{
    impls::{double::DoubleBehavior, generic::StraightPathBehavior, random::RandomBehavior},
    EnemyId, Freak,
};
use map::{Map, RoomId, RootRoomInfo};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use slotmap::SlotMap;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub mod enemies;
pub mod map;

/// How much power a door being closed draws
pub const POWER_DRAW_DOOR: i32 = 75;
/// How much power is idly drawn
pub const DEFAULT_POWER_DRAW: i32 = 5;
/// How much power being on the cameras draws
pub const CAMERA_ON_DRAW: i32 = 15;
/// How much power you start with
pub const INITIAL_POWER: i32 = 500_000;
/// How many game ticks we need to win
pub const TICKS_PER_HOUR: u64 = 1800;
/// How many hours do we need to survive
pub const HOURS_TO_WIN: u64 = 6;

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
        let mut rng = thread_rng();
        let move_rng = thread_rng();

        let mut enemies: SlotMap<EnemyId, Freak> = SlotMap::default();

        // Register all enemies we want in the game
        let enemy_registry: Vec<Freak> = vec![
            Freak::new("teller", 800..1200, StraightPathBehavior::default()),
            Freak::new(
                "remington",
                800..2500,
                DoubleBehavior::new(StraightPathBehavior::default()),
            ),
            Freak::new("frank", 300..800, RandomBehavior::new(move_rng)),
        ];

        for enemy in enemy_registry {
            enemies.insert(enemy);
        }

        let state =
            GameState::default().with_enemies(&enemies.keys().collect::<Vec<_>>(), &mut rng);

        Self {
            enemies,
            state,
            rng,
        }
    }
}

#[wasm_bindgen]
impl Game {
    /// Ticks the game forward
    pub fn tick(&mut self) -> bool {
        self.state.tick(&mut self.enemies, &mut self.rng)
    }

    /// Gets the map context as a JsValue
    pub fn get_map(&self) -> JsValue {
        self.state.map.serialize_room_layout(&self.state.office)
    }

    /// Create a new game (trait impls aren't accessible to wasm_bindgen
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current time as an hour
    pub fn get_time(&self) -> u8 {
        let hours = (self.state.ticks / TICKS_PER_HOUR) as u8;
        match hours {
            0 => 12,
            _ => hours,
        }
    }

    /// Gets a snapshot of what the current camera room's name is and what enemies are in it
    pub fn get_room(&self, room: u64) -> Option<Vec<String>> {
        let room = slotmap::KeyData::from_ffi(room);
        let room = &self.state.map.0[room.into()];
        let (_, enemies) = room.get_cams()?;

        let mut res = vec![];

        for enemy in enemies {
            let name = self.enemies[*enemy].get_name();

            res.push(name.to_string());
        }

        Some(res)
    }

    /// Toggles the camera state
    pub fn toggle_cameras(&mut self) {
        self.state.toggle_cameras();
    }

    /// Close the left door
    pub fn toggle_left(&mut self) {
        self.state.toggle_door(Door::Left)
    }

    /// Close the right door
    pub fn toggle_right(&mut self) {
        self.state.toggle_door(Door::Right)
    }

    /// Is left door closed?
    pub fn is_left_closed(&self) -> bool {
        self.state.left_door
    }

    /// Is right door closed?
    pub fn is_right_closed(&self) -> bool {
        self.state.right_door
    }

    /// Check the current power draw
    pub fn power_percent(&self) -> f64 {
        ((self.state.power as f64 / INITIAL_POWER as f64) * 100.0).max(0.0)
    }

    /// Check if we're dead
    pub fn is_dead(&self) -> Option<String> {
        if self.state.dead {
            self.state
                .get_enemy_in_room()
                .map(|id| self.enemies[id].get_name().to_string())
        } else {
            None
        }
    }

    /// Render the current map
    pub fn render(&mut self) -> String {
        self.state.map.display()
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
    /// Where enemies are located in the camera view
    pub locations: HashMap<EnemyId, (f32, f32)>,
    /// If the left door is closed
    left_door: bool,
    /// If the right door is closed
    right_door: bool,
    /// Are the cameras on
    cameras_on: bool,
    /// How much power is left
    power: i32,
    /// The current power draw (per tick)
    draw: i32,
    /// Are we dead?
    dead: bool,
    /// What is our target ticks
    ticks_needed_to_win: u64,
}

impl Default for GameState {
    fn default() -> Self {
        let mut map = Map::default();
        let mut rng = thread_rng();
        let (office, spawn_points) = map.generate(&mut rng);

        GameState {
            cooldowns: HashMap::default(),
            ticks: 0,
            map,
            office,
            spawn_points,
            locations: HashMap::new(),
            power: INITIAL_POWER,
            left_door: false,
            right_door: false,
            cameras_on: false,
            draw: DEFAULT_POWER_DRAW,
            dead: false,
            ticks_needed_to_win: HOURS_TO_WIN * TICKS_PER_HOUR,
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

    /// Generates a random location for an enemy and reassigns that in the lookup table
    pub fn generate_coords<RNG: Rng>(&mut self, enemy: EnemyId, rng: &mut RNG) -> (f32, f32) {
        let x = rng.gen_range(20..=230);
        let y = rng.gen_range(10..=90);

        self.locations.insert(enemy, (x as f32, y as f32));
        self.locations[&enemy]
    }

    /// Returns an enemy's current location on the cameras
    pub fn get_coords(&self, enemy: &EnemyId) -> (f32, f32) {
        self.locations[enemy]
    }

    /// Ticks through all enemy behaviors if it's time
    pub fn tick<RNG: Rng>(&mut self, enemies: &mut SlotMap<EnemyId, Freak>, rng: &mut RNG) -> bool {
        self.ticks += 1;

        if self.ticks == self.ticks_needed_to_win {
            return true;
        }

        self.power -= self.draw;
        self.out_of_power();

        for (id, enemy) in enemies {
            if let Some(time) = self.cooldowns.get(&id) {
                if self.ticks % time == 0 {
                    // It's action time
                    enemy.tick(id, self, rng);
                }
            } else {
                let new_cooldown = enemy.gen_cooldown(rng);
                self.cooldowns.insert(id, new_cooldown);
            }
        }

        if self.map.room_has_enemies(self.office.root) {
            self.dead = true
        }

        false
    }

    /// If power is below 0, opens the doors
    pub fn out_of_power(&mut self) -> bool {
        if self.power <= 0 {
            self.left_door = false;
            self.right_door = false;

            true
        } else {
            false
        }
    }

    /// Toggles the cameras and sets the appropriate new power draw
    pub fn toggle_cameras(&mut self) {
        self.cameras_on = !self.cameras_on;

        if self.cameras_on {
            self.draw += CAMERA_ON_DRAW
        } else {
            self.draw -= CAMERA_ON_DRAW
        }
    }

    /// Toggles if a door is open or closed, affecting power draw respectively
    pub fn toggle_door(&mut self, direction: Door) {
        if !self.out_of_power() {
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
                self.draw -= POWER_DRAW_DOOR;
            } else {
                self.draw += POWER_DRAW_DOOR;
            }
        }
    }

    /// Returns the first enemy in a room if it exists
    pub fn get_enemy_in_room(&self) -> Option<EnemyId> {
        let enemies_in_room = self.map.enemies_in_room(self.office.root);
        if enemies_in_room.is_empty() {
            None
        } else {
            Some(enemies_in_room[0])
        }
    }

    /// Moves an enemy from their current room to the desired room if possible
    pub(crate) fn move_enemy<RNG: Rng>(&mut self, freak: EnemyId, to: RoomId, rng: &mut RNG) {
        let room = self.map.get_enemy_room(freak);
        if let Some(room) = room {
            self.map.move_enemy_out_of(room, freak)
        }
        self.generate_coords(freak, rng);
        self.map.move_enemy_to(to, freak);
    }

    /// Attacks with a given enemy if possible
    pub(crate) fn attack<RNG: Rng>(&mut self, attacker: EnemyId, rng: &mut RNG) {
        if self.attack_possible(attacker) {
            self.move_enemy(attacker, self.office.root, rng);
        } else {
            self.move_enemy(attacker, *self.spawn_points.choose(rng).unwrap(), rng)
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

        for _ in 0..100 {
            game.tick(&mut enemy_map, &mut rng);
        }

        println!("{}", game.map);
        assert!(game.dead);
    }
}
