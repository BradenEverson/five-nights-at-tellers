//! Map and Room Layout information

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use rand::{seq::SliceRandom, Rng};
use slotmap::{new_key_type, SlotMap};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::enemies::EnemyId;

pub mod export;

new_key_type! {
    /// A room's ID
    #[wasm_bindgen]
    pub struct RoomId;
}

/// A contextual graph of all rooms
#[derive(Default)]
pub struct Map(pub(crate) SlotMap<RoomId, Room>);

/// The root room with distinct left and right 'hallways'
pub struct RootRoomInfo {
    /// The root
    pub root: RoomId,
    /// The room connected to the 'left'
    pub left: RoomId,
    /// The room connected to the 'right'
    pub right: RoomId,
}

impl Map {
    /// Connects two rooms by a shared pathway
    pub fn connect_rooms(&mut self, a: RoomId, b: RoomId) {
        self.0[a].connect_to(b);
        self.0[b].connect_to(a);
    }

    /// Returns true if a room has any enemies
    pub fn room_has_enemies(&self, room: RoomId) -> bool {
        !self.0[room].occupied_by.is_empty()
    }

    /// Returns the room an enemy is in if they are in a room, `None` if otherwise
    pub fn get_enemy_room(&self, enemy: EnemyId) -> Option<RoomId> {
        self.0
            .iter()
            .find(|(_, room)| room.enemy_is_in(enemy))
            .map(|(id, _)| id)
    }

    /// Registers an initial enemy to be in a starting room
    pub fn register_enemy(&mut self, enemy: EnemyId, room: RoomId) {
        self.0[room].move_into(enemy);
    }

    /// Get's all enemies in a room
    pub fn enemies_in_room(&self, room: RoomId) -> &[EnemyId] {
        &self.0[room].occupied_by
    }

    /// Generates a new layout, returning the ID of the office room and a list of good spawnable
    /// positions
    pub fn generate<RNG: Rng>(&mut self, rng: &mut RNG) -> (RootRoomInfo, Vec<RoomId>) {
        let mut office = Room::default();
        let mut left = Room::default();
        let mut right = Room::default();

        office.set_name("office");
        left.set_name("left_entrance");
        right.set_name("right_entrance");

        let mut hallway_left = Room::default();
        let mut hallway_right = Room::default();

        hallway_left.set_name("left_hallway");
        hallway_right.set_name("right_hallway");

        let office = self.0.insert(office);
        let left = self.0.insert(left);
        let right = self.0.insert(right);
        let hallway_left = self.0.insert(hallway_left);
        let hallway_right = self.0.insert(hallway_right);

        self.connect_rooms(office, left);
        self.connect_rooms(office, right);

        self.connect_rooms(left, hallway_left);
        self.connect_rooms(right, hallway_right);

        let additional_rooms: usize = rng.gen_range(10..=15);
        let mut room_ids = vec![hallway_left, hallway_right];

        for _ in 0..additional_rooms {
            let new_room = self.0.insert(Room::default());
            let existing_room = *room_ids.choose(rng).unwrap();

            self.connect_rooms(new_room, existing_room);
            room_ids.push(new_room)
        }

        let extra_connections: usize = rng.gen_range(1..=3);
        for _ in 0..extra_connections {
            let room_a = *room_ids.choose(rng).unwrap();
            let room_b = *room_ids.choose(rng).unwrap();

            if room_a != room_b && !self.0[room_a].conencts_to.contains(&room_b) {
                self.connect_rooms(room_a, room_b);
            }
        }

        // Generate viable rooms to spawn enemies in, cannot be directly connected to the main
        // rooms
        let mut viable_spawn_rooms: Vec<_> = room_ids
            .into_iter()
            .filter(|id| !self.0[*id].connects_to_any(&[&left, &right]))
            .collect();
        let mut spawn_rooms = vec![];
        let spawn_rooms_count: usize = rng.gen_range(1..=4);

        for _ in 0..spawn_rooms_count {
            if let Some(popped) = viable_spawn_rooms.pop() {
                spawn_rooms.push(popped);
            }
        }

        (
            RootRoomInfo {
                root: office,
                left,
                right,
            },
            spawn_rooms,
        )
    }

    /// Creates a path from one room to another room
    pub fn generate_path(&self, from: RoomId, to: RoomId) -> Option<Vec<RoomId>> {
        let mut search_queue = VecDeque::new();
        let mut seen = HashSet::new();
        let mut predecessors = HashMap::new();

        search_queue.push_front(from);
        seen.insert(from);

        while let Some(check_room) = search_queue.pop_back() {
            if check_room == to {
                let mut path = Vec::new();
                let mut current = to;
                while current != from {
                    path.push(current);
                    current = *predecessors.get(&current)?;
                }
                path.push(from);
                path.reverse();
                return Some(path);
            }

            for &next_room in &self.0[check_room].conencts_to {
                if seen.insert(next_room) {
                    search_queue.push_front(next_room);
                    predecessors.insert(next_room, check_room);
                }
            }
        }

        None
    }

    /// Disables a room's camera
    pub fn disable_room_cam(&mut self, room: RoomId) {
        self.0[room].disable_camera()
    }

    /// Enables a room's camera
    pub fn enable_room_cam(&mut self, room: RoomId) {
        self.0[room].enable_camera()
    }

    /// Moves an enemy into the room
    pub fn move_enemy_to(&mut self, room: RoomId, enemy: EnemyId) {
        self.0[room].move_into(enemy)
    }

    /// Moves an enemy out of a room
    pub fn move_enemy_out_of(&mut self, room: RoomId, enemy: EnemyId) {
        self.0[room].move_out_of(enemy)
    }
    /// Returns the map layout as a String
    pub fn display(&self) -> String {
        let mut visited = HashSet::new();
        let mut output = String::new();

        for (room_id, _) in &self.0 {
            if visited.contains(&room_id) {
                continue;
            }
            self.display_room(room_id, 0, &mut visited, &mut output);
        }

        output
    }

    /// Recursively builds a string to represent a room and its connected rooms
    fn display_room(
        &self,
        room_id: RoomId,
        depth: usize,
        visited: &mut HashSet<RoomId>,
        output: &mut String,
    ) {
        if !visited.insert(room_id) {
            return;
        }

        let room = &self.0[room_id];
        let indent = "    ".repeat(depth);

        let enemies_in: String = room.occupied_by.iter().map(|_| "😈").collect();
        output.push_str(&format!(
            "{}Room {:?}: {}{}\n",
            indent,
            room_id,
            room.get_name(),
            enemies_in
        ));

        for &connected_room in &room.conencts_to {
            if !visited.contains(&connected_room) {
                output.push_str(&format!("{}|---> Room {:?}\n", indent, connected_room));
                self.display_room(connected_room, depth + 1, visited, output);
            }
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

/// A room's information, such as who's in the room, is it disabled on the camera, and what rooms
/// does it connect to
#[derive(Default)]
pub struct Room {
    /// A Room's name
    name: String,
    /// Whether the room is disabled on the cameras or not
    disabled: bool,
    /// Who's in it
    occupied_by: Vec<EnemyId>,
    /// Pathways to other rooms
    conencts_to: Vec<RoomId>,
}

impl Room {
    /// Connets a room to another room
    pub fn connect_to(&mut self, room: RoomId) {
        self.conencts_to.push(room)
    }

    /// Checks if the room connects to any of the id's provided
    pub fn connects_to_any(&self, connections: &[&RoomId]) -> bool {
        for room in &self.conencts_to {
            if connections.contains(&room) {
                return true;
            }
        }

        false
    }

    /// If the camera isn't disabled, returns the room's name and enemy id's inside
    pub fn get_cams(&self) -> Option<(&str, &[EnemyId])> {
        if !self.disabled {
            Some((self.get_name(), &self.occupied_by))
        } else {
            None
        }
    }

    /// Sets a room's name
    pub fn set_name<NAME: Into<String>>(&mut self, name: NAME) {
        self.name = name.into()
    }

    /// Returns the room's name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Moves an enemy into the room
    pub fn move_into(&mut self, enemy: EnemyId) {
        self.occupied_by.push(enemy)
    }

    /// Moves an enemy out of the room
    pub fn move_out_of(&mut self, enemy: EnemyId) {
        let idx = self.occupied_by.iter_mut().position(|id| *id == enemy);
        if let Some(idx) = idx {
            self.occupied_by.remove(idx);
        }
    }

    /// Checks if the specified enemy is in this room
    pub fn enemy_is_in(&self, enemy: EnemyId) -> bool {
        self.occupied_by.contains(&enemy)
    }

    /// Disables camera viewing for this room
    pub fn disable_camera(&mut self) {
        self.disabled = true
    }

    /// Enables camera viewing for this room
    pub fn enable_camera(&mut self) {
        self.disabled = false
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use slotmap::SlotMap;

    use crate::enemies::Freak;

    use super::{Map, Room};

    #[test]
    fn path_gen_works() {
        let mut map = Map::default();

        let mut room_a = Room::default();
        let mut room_b = Room::default();
        let mut room_c = Room::default();
        let mut room_d = Room::default();
        let mut room_e = Room::default();
        let room_f = Room::default();
        let room_g = Room::default();

        // Create path from a -> g, which will be a -> c -> d -> b -> g

        let room_g = map.0.insert(room_g);
        room_b.connect_to(room_g);
        let room_b = map.0.insert(room_b);
        room_d.connect_to(room_b);

        // Create off shoots
        let room_f = map.0.insert(room_f);
        room_e.connect_to(room_f);
        let room_e = map.0.insert(room_e);
        room_d.connect_to(room_e);

        let room_d = map.0.insert(room_d);
        room_c.connect_to(room_d);
        let room_c = map.0.insert(room_c);
        room_a.connect_to(room_c);
        let room_a = map.0.insert(room_a);

        let path = map.generate_path(room_a, room_g).expect("Generate path");
        assert_eq!(path, [room_a, room_c, room_d, room_b, room_g]);
    }

    #[test]
    fn map_generation_is_good() {
        let mut map = Map::default();
        let mut rng = thread_rng();
        map.generate(&mut rng);

        println!("{map}")
    }

    #[test]
    fn map_generation_with_enemies_is_good() {
        let mut map = Map::default();
        let mut rng = thread_rng();
        let (_, enemy_spawn) = map.generate(&mut rng);
        let default_enemy = Freak::default_test_enemy();
        let mut enemy_slot = SlotMap::default();
        let enemy_id = enemy_slot.insert(default_enemy);

        map.register_enemy(enemy_id, enemy_spawn[0]);

        println!("{map}")
    }
}
