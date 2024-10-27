//! Serialization methods for sending off Room data to be rendered

use std::collections::{HashMap, HashSet, VecDeque};

use serde::Serialize;
use wasm_bindgen::JsValue;

use super::{Map, RoomId, RootRoomInfo};

/// A serializable snapshot of room data for sending over to the frontend
#[derive(Serialize)]
pub struct CameraNode {
    /// The room's ID
    pub id: RoomId,
    /// Room name
    pub name: String,
    /// X position on canvas
    pub x: f32,
    /// Y position on canvas
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Rooms connected to this room
    pub connected_to: Vec<RoomId>,
}

impl CameraNode {
    /// Generates a new camera node
    pub fn new<NAME: Into<String>>(id: RoomId, name: NAME, x: f32, y: f32, width: f32, height: f32, connected_to: Vec<RoomId>) -> Self {
        Self { id, name: name.into(), x, y, width, height, connected_to }
    }
}

impl Map {
    /// Serializes the entire room layout as a JSON array of rooms with their positions
    pub fn serialize_room_layout(&self, root: &RootRoomInfo) -> JsValue {
        let mut coords = HashMap::new();
        let mut seen = HashSet::new();
        let mut queue = VecDeque::new();
        let mut room_nodes = vec![];

        let mut y = 0.0;

        queue.push_back(root.root);
        coords.insert(root.root, (0.0, 0.0));

        while let Some(room) = queue.pop_front() {
            // Find self's coord, create CameraNode from that and push it to vec
            let (node_x, node_y) = coords[&room];
            let node = CameraNode::new(room, self.0[room].get_name(), node_x, node_y, 12.0, 12.0, self.0[room].conencts_to.clone());
            room_nodes.push(node);

            y += 16.0;
            let mut local_x = -16.0;

            for connection in &self.0[room].conencts_to {
                if !coords.contains_key(connection) {
                    coords.insert(*connection, (local_x, y));
                    local_x += 16.0;
                }
                if !seen.contains(connection) {
                    queue.push_back(*connection)
                }
            }
            
            seen.insert(room);
        }

        serde_wasm_bindgen::to_value(&room_nodes).expect("Failed to serialize")
    }
}
