use std::collections::HashMap;
use std::time::SystemTime;
use bevy::prelude::*;
use noise::Perlin;

use crate::{GameState, CHUNK_WIDTH, CHUNK_HEIGHT};

use self::{systems::{generate_chunks_from_player_movement, deque_chunks, unload_far_chunks}, chunk::components::BlockType};

pub(crate) mod chunk;
pub(crate) mod systems;


pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WorldMap { chunks: HashMap::new(), chunk_entities: HashMap::new(), water_chunk_entities: HashMap::new(), reserved_chunk_data: HashMap::new() })
            .insert_resource(ChunkQueue { queue: vec![], is_next_ready: true })
            .add_systems(Startup, generate_world_system)
            .add_systems(Update, (
                generate_chunks_from_player_movement,
                deque_chunks,
                unload_far_chunks
            ).run_if(in_state(GameState::Running)));
    }
}


#[derive(Resource)]
pub struct WorldMap {
    pub chunks: HashMap<(i32, i32), [BlockType; CHUNK_WIDTH*CHUNK_HEIGHT*CHUNK_WIDTH]>,
    pub chunk_entities: HashMap<(i32,i32), Entity>,
    pub water_chunk_entities: HashMap<(i32, i32), Entity>,
    pub reserved_chunk_data: HashMap<(i32, i32), [BlockType; CHUNK_WIDTH*CHUNK_HEIGHT*CHUNK_WIDTH]>,
}

#[derive(Resource)]
pub struct SeededPerlin {
    pub seed: u32,
    pub terrain_noise: Perlin,
    pub tree_noise: Perlin,
    pub temperature_noise: Perlin,
    pub moisture_noise: Perlin,
}

#[derive(Resource)]
pub struct ChunkQueue {
    pub queue: Vec<(i32, i32)>,
    pub is_next_ready: bool,
}


fn generate_world_system(mut commands: Commands) {
    let seed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("[E] SystemTime before UNIX EPOCH!").as_secs() as u32;
    let terrain_perlin = Perlin::new(seed);
    let tree_perlin = Perlin::new(seed*2);
    let temperature_perlin = Perlin::new(seed+20);
    let moisture_perlin = Perlin::new(seed+30);

    commands.insert_resource(SeededPerlin { seed: seed, terrain_noise: terrain_perlin, tree_noise: tree_perlin, temperature_noise: temperature_perlin, moisture_noise: moisture_perlin});
}

// #[derive(Resource)]
// pub struct ChunkQueueConfig {
//     timer: Timer,
// }