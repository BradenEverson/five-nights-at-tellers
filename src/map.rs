//! Map and Room Layout information

use std::collections::{HashSet, VecDeque};

use slotmap::{new_key_type, SlotMap};

use crate::enemies::EnemyId;

new_key_type! {
    /// A room's ID
    pub struct RoomId;
}

/// A contextual graph of all rooms
#[derive(Default)]
pub struct Map(SlotMap<RoomId, Room>);

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

    /// Generates a new layout, returning the ID of the office room
    pub fn generate(&mut self) -> RootRoomInfo {
        let office = Room::default();
        let left = Room::default();
        let right = Room::default();

        let office = self.0.insert(office);
        let left = self.0.insert(left);
        let right = self.0.insert(right);

        self.connect_rooms(office, left);
        self.connect_rooms(office, right);

        // TODO: Continue creating graph by branching from left and right, don't just make a tree
        // nodes can connect to previous nodes too

        RootRoomInfo {
            root: office,
            left,
            right,
        }
    }

    /// Creates a path from one room to another room
    pub fn generate_path(&self, from: RoomId, to: RoomId) -> Option<Vec<RoomId>> {
        let mut search_queue = VecDeque::new();
        let mut seen = HashSet::new();

        search_queue.push_front(from);

        while let Some(check_room) = search_queue.pop_back() {
            if !seen.insert(check_room) {
                continue;
            }

            if check_room == to {
                // TODO: Right now we basically just return *if* a path exists, not what it is.
                // Will probably need to get recursive and weird with it
                return Some(vec![])
            }

            for room in &self.0[check_room].conencts_to {
                search_queue.push_front(*room)
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
}

/// A room's information, such as who's in the room, is it disabled on the camera, and what rooms
/// does it connect to
#[derive(Default)]
pub struct Room {
    /// A Room's name
    name: &'static str,
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

    /// Sets a room's name
    pub fn set_name(&mut self, name: &'static str) {
        self.name = name
    }

    /// Returns the room's name
    pub fn get_name(&self) -> &str {
        self.name
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
