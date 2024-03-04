use std::collections::HashMap;
use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh}};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape, Friction, CoefficientCombineRule};
use noise::{Perlin, NoiseFn};
use rand::{rngs::StdRng, SeedableRng, Rng};

use crate::{CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_VOL, plugins::world::{WorldMap, SeededPerlin}};

use self::structures_generation::{add_tree, add_cactus};

use super::components::BlockType;

mod structures_generation;


const SEA_LEVEL: usize = 62;


pub fn generate_chunk_data(perlin: &Res<SeededPerlin>, chunk_pos: (i32, i32), world_map: &mut ResMut<WorldMap>) {

    let seed = (perlin.seed).wrapping_add(chunk_pos.0 as u32).wrapping_add(chunk_pos.1 as u32);
    let mut random = StdRng::seed_from_u64(seed as u64);

    let mut blocks = [BlockType::Air; CHUNK_VOL];

    generate_terrain_shape(perlin, chunk_pos, &mut blocks);
    generate_ore(&mut random, &mut blocks);
    generate_terrain_cover(perlin, chunk_pos, &mut blocks, world_map);

    world_map.chunks.insert(chunk_pos, blocks);
}


pub fn generate_terrain_shape(perlin: &Res<SeededPerlin>, chunk_pos: (i32, i32), blocks: &mut [BlockType; CHUNK_VOL]) {

    for i in 0 .. CHUNK_VOL {

        let z = i / (CHUNK_WIDTH * CHUNK_HEIGHT);
        let y = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) / CHUNK_WIDTH;
        let x = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) % CHUNK_WIDTH;

        let (height, _) = height_by_coords(perlin.terrain_noise, x, z, chunk_pos);

        let index = x + y * CHUNK_WIDTH + z * CHUNK_WIDTH * CHUNK_HEIGHT;

        if y == 0 {
            blocks[index] = BlockType::BedRock;
        }
        else if y < height {
            blocks[index] = BlockType::Stone;
        }
    }
}


pub fn generate_terrain_cover(perlin: &Res<SeededPerlin>, chunk_pos: (i32, i32), blocks: &mut [BlockType; CHUNK_VOL], world_map: &mut ResMut<WorldMap>) {

    let mut tree_positions = vec![];
    let mut random = StdRng::seed_from_u64(perlin.seed as u64);

    for i in 0 .. CHUNK_VOL {

        let z = i / (CHUNK_WIDTH * CHUNK_HEIGHT);
        let y = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) / CHUNK_WIDTH;
        let x = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) % CHUNK_WIDTH;

        let mut block = BlockType::Air;

        let (height, coverheight) = height_by_coords(perlin.terrain_noise, x, z, chunk_pos);

        if y < height {
            continue;
        }


        if y <= coverheight{

            let temperature = temperature_at(perlin, chunk_pos, x, z);
            let humidity = humidity_at(perlin, chunk_pos, x, z);
            
            if temperature > 0.7 && humidity < 0.4 {
                block = BlockType::Sand;

                if y >= SEA_LEVEL {
                    let tree_value = perlin.tree_noise.get([
                        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.1,
                        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.1]
                    ) as f32;
    
                    let tree_chance = random.gen_range(-1.0 .. tree_value.abs());
                    if tree_value > 0.2 && tree_chance > 0.96 {
                        tree_positions.push((x,y,z));
                    }
                }
            }
            else if y == coverheight {

                if y >= SEA_LEVEL {

                    block = BlockType::Grass;

                    let tree_value = perlin.tree_noise.get([
                        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.03,
                        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.03]
                    ) as f32;
    
                    let tree_chance = random.gen_range(-1.0 .. tree_value.abs());
                    if tree_value > 0.2 && tree_chance > 0.6 {
                        tree_positions.push((x,y,z));
                    }
                }
                else {
                    block = BlockType::Dirt;
                }
            }
            else {
                block = BlockType::Dirt;
            }
        }
        else if y <= SEA_LEVEL {
            block = BlockType::Water;
        }


        let index = x + y * CHUNK_WIDTH + z * CHUNK_WIDTH * CHUNK_HEIGHT;
        blocks[index] = block;
    }

    for pos in tree_positions.iter() {
        let temperature = temperature_at(perlin, chunk_pos, pos.0, pos.2);
        let humidity = humidity_at(perlin, chunk_pos, pos.0, pos.2);

        if temperature > 0.65 && humidity < 0.4 {
            add_cactus(random.gen_range(2..5), pos.0, pos.1, pos.2, blocks);
        }
        else {
            add_tree(random.gen_range(3..6), chunk_pos, pos.0, pos.1, pos.2, world_map, blocks);
        }
    }
}


fn generate_ore(random: &mut StdRng, blocks: &mut [BlockType; CHUNK_VOL]) {
    
    let fillings = random.gen_range(0 .. 50);

    for _ in 0 .. fillings {

        // Random walk may walk back on itself, so expect slightly pessimized ore count.
        let neighbours = random.gen_range(1 .. 7);
        let mut pos = random.gen_range(0 .. CHUNK_VOL) as i32;

        if blocks[pos as usize] != BlockType::Stone {
            continue;
        }

        blocks[pos as usize] = BlockType::OreStoneGold;

        for _ in 0 .. neighbours {

            let sub = random.gen_bool(0.5);
            let xyz: f32 = random.gen();

            if xyz < 0.33 {
                // Walk along the x-axis.
                if sub {
                    pos = pos - 1;
                }
                else {
                    pos = pos + 1;
                }
                if pos < CHUNK_VOL as i32 && pos > 0 {
                    blocks[pos as usize] = BlockType::OreStoneGold;
                }
            }
            else if xyz < 0.66 {
                // Walk along the y-axis.
                if sub {
                    pos = pos + CHUNK_WIDTH as i32;
                }
                else {
                    pos = pos - CHUNK_WIDTH as i32;
                }
                if pos < CHUNK_VOL as i32 && pos > 0 {
                    blocks[pos as usize] = BlockType::OreStoneGold;
                }
            }
            else {
                // Walk along the z-axis.
                if sub {
                    pos = pos - (CHUNK_WIDTH * CHUNK_HEIGHT) as i32;
                }
                else {
                    pos = pos + (CHUNK_WIDTH * CHUNK_HEIGHT) as i32;
                }
                if pos < CHUNK_VOL as i32 && pos > 0 {
                    blocks[pos as usize] = BlockType::OreStoneGold;
                }
            }
        }
    }
}


fn temperature_at(perlin: &Res<SeededPerlin>, chunk_pos: (i32, i32), x: usize, z: usize) -> f32 {
    perlin.temperature_noise.get([
        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.001,
        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.001
    ]) as f32 * 10.0
}


fn humidity_at(perlin: &Res<SeededPerlin>, chunk_pos: (i32, i32), x: usize, z: usize) -> f32 {
    perlin.moisture_noise.get([
        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.001,
        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.001
    ]) as f32 * 10.0
}


fn height_by_coords(perlin: Perlin, x: usize, z: usize, chunk_pos: (i32, i32)) -> (usize, usize) {

    let octave0 = perlin.get([
        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.001,
        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.001]
    ) as f32 * 30.0;

    let octave1 = perlin.get([
        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.01,
        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.01]
    ) as f32 * 10.0;

    let octave2 = perlin.get([
        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.06,
        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.06]
    ) as f32 * 4.0;

    let octave2_cover = perlin.get([
        (x as f64 + chunk_pos.0 as f64 * CHUNK_WIDTH as f64) * 0.05,
        (z as f64 + chunk_pos.1 as f64 * CHUNK_WIDTH as f64) * 0.05]
    ) as f32 * 2.0;

    (
        (octave0 + octave1 + octave2 + 60.0).floor() as usize,
        (octave0 + octave1 + octave2_cover + 64.0).floor() as usize,
    )
}


pub fn generate_water_chunk_mesh(
    world_map: &mut ResMut<WorldMap>,
    position: (i32, i32),
) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );

    let mut verticies: Vec<[f32; 3]> = vec![];
    let mut indices: Vec<u32> = vec![];
    let mut uvs: Vec<Vec2> = vec![];

    for i in 0..CHUNK_VOL {
        let z = i / (CHUNK_WIDTH*CHUNK_HEIGHT);
        let y = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) / CHUNK_WIDTH;
        let x = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) % CHUNK_WIDTH;

        generate_water_block(&mut verticies, &mut indices, &mut uvs, &world_map.chunks, &(x as i32,y as i32,z as i32), &position);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(mesh::Indices::U32(indices));

    mesh
}

pub fn generate_chunk_mesh(
    world_map: &mut ResMut<WorldMap>,
    position: (i32, i32),
) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );

    let mut verticies: Vec<[f32; 3]> = vec![];
    let mut indices: Vec<u32> = vec![];
    let mut uvs: Vec<Vec2> = vec![];
    let mut colors: Vec<[f32; 4]> = vec![];

    for i in 0..CHUNK_VOL {
        let z = i / (CHUNK_WIDTH*CHUNK_HEIGHT);
        let y = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) / CHUNK_WIDTH;
        let x = (i - (z * CHUNK_WIDTH*CHUNK_HEIGHT)) % CHUNK_WIDTH;

        generate_block(&mut verticies, &mut indices, &mut uvs, &world_map.chunks, &(x as i32,y as i32,z as i32), &position);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verticies);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(mesh::Indices::U32(indices));

    calculate_ao(&mut colors, position, &world_map.chunks);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    mesh
}

fn calculate_ao(
    colors: &mut Vec<[f32; 4]>,
    chunk_position: (i32, i32),
    chunks: &HashMap<(i32,i32), [BlockType; CHUNK_VOL]>,
) {
    for index in 0..CHUNK_VOL {
        if !chunks[&chunk_position][index].is_transparent() {
            let z = (index / (CHUNK_WIDTH*CHUNK_HEIGHT)) as i32;
            let y = ((index - (z as usize * CHUNK_WIDTH*CHUNK_HEIGHT)) / CHUNK_WIDTH) as i32;
            let x = ((index - (z as usize * CHUNK_WIDTH*CHUNK_HEIGHT)) % CHUNK_WIDTH) as i32;

            if block_at_position(chunks, (x + 1, y, z), chunk_position).is_transparent() {
                let neighbors = [
                    block_at_position(chunks, (x + 1, y, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z - 1), chunk_position),
                ];
                let darks = side_ao(neighbors);
                colors.append(&mut vec![[darks[0], darks[0], darks[0], 1.],
                                        [darks[1], darks[1], darks[1], 1.],
                                        [darks[2], darks[2], darks[2], 1.],
                                        [darks[3], darks[3], darks[3], 1.]]);
            }

            if block_at_position(chunks, (x - 1, y, z), chunk_position).is_transparent(){
        
                let neighbors = [
                    block_at_position(chunks, (x - 1, y, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x - 1, y, z - 1), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z - 1), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z + 1), chunk_position),
                ];
                let darks = side_ao(neighbors);
                colors.append(&mut vec![[darks[0], darks[0], darks[0], 1.],
                                        [darks[1], darks[1], darks[1], 1.],
                                        [darks[2], darks[2], darks[2], 1.],
                                        [darks[3], darks[3], darks[3], 1.]]);
            }
            
            if block_at_position(chunks, (x, y, z - 1), chunk_position).is_transparent() {

                let neighbors = [
                    block_at_position(chunks, (x - 1, y, z - 1), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z - 1), chunk_position),
                    block_at_position(chunks, (x, y + 1, z - 1), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z - 1), chunk_position),
                ];
                let darks = side_ao(neighbors);
                colors.append(&mut vec![[darks[0], darks[0], darks[0], 1.],
                                        [darks[1], darks[1], darks[1], 1.],
                                        [darks[2], darks[2], darks[2], 1.],
                                        [darks[3], darks[3], darks[3], 1.]]);
            }

            if block_at_position(chunks, (x, y, z + 1), chunk_position).is_transparent() {
                let neighbors = [
                    block_at_position(chunks, (x + 1, y, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z + 1), chunk_position),
                    block_at_position(chunks, (x, y + 1, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z + 1), chunk_position),
                ];
                let darks = side_ao(neighbors);
                colors.append(&mut vec![[darks[0], darks[0], darks[0], 1.],
                                        [darks[1], darks[1], darks[1], 1.],
                                        [darks[2], darks[2], darks[2], 1.],
                                        [darks[3], darks[3], darks[3], 1.]]);
            }

            if block_at_position(chunks, (x, y - 1, z), chunk_position).is_transparent() {
        
                let neighbors = [
                    block_at_position(chunks, (x - 1, y - 1, z), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z + 1), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z), chunk_position),
                    block_at_position(chunks, (x + 1, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x, y - 1, z - 1), chunk_position),
                    block_at_position(chunks, (x - 1, y - 1, z - 1), chunk_position),
                ];
                let darks = side_ao(neighbors);
                colors.append(&mut vec![[darks[0], darks[0], darks[0], 1.],
                                        [darks[1], darks[1], darks[1], 1.],
                                        [darks[2], darks[2], darks[2], 1.],
                                        [darks[3], darks[3], darks[3], 1.]]); 
            }

            if block_at_position(chunks, (x, y + 1, z), chunk_position).is_transparent() {
        
                let neighbors = [
                    block_at_position(chunks, (x, y + 1, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z + 1), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z), chunk_position),
                    block_at_position(chunks, (x - 1, y + 1, z - 1), chunk_position),
                    block_at_position(chunks, (x, y + 1, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z - 1), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z), chunk_position),
                    block_at_position(chunks, (x + 1, y + 1, z + 1), chunk_position),   
                ];
                let darks = side_ao(neighbors);
                colors.append(&mut vec![
                    [darks[0], darks[0], darks[0], 1.],
                    [darks[1], darks[1], darks[1], 1.],
                    [darks[2], darks[2], darks[2], 1.],
                    [darks[3], darks[3], darks[3], 1.],
                ]);
            }
        }
    }
}

fn ao_value(side1: bool, corner: bool, side2: bool) -> u32 {
    match (side1, corner, side2) {
        (true, _, true) => 0,
        (true, true, false) | (false, true, true) => 1,
        (false, false, false) => 3,
        _ => 2,
    }
}

fn side_ao(neighbors: [BlockType; 8]) -> [f32; 4] {
    let ns = [
        neighbors[0].is_transparent(),
        neighbors[1].is_transparent(),
        neighbors[2].is_transparent(),
        neighbors[3].is_transparent(),
        neighbors[4].is_transparent(),
        neighbors[5].is_transparent(),
        neighbors[6].is_transparent(),
        neighbors[7].is_transparent(),
    ];

    [
        1.0 - ao_value(ns[6], ns[7], ns[0]) as f32 * 0.25,
        1.0 - ao_value(ns[4], ns[5], ns[6]) as f32 * 0.25,
        1.0 - ao_value(ns[2], ns[3], ns[4]) as f32 * 0.25,
        1.0 - ao_value(ns[0], ns[1], ns[2]) as f32 * 0.25,

    ]
}

pub fn build_water_chunk(
    commands: &mut Commands,
    world_map: &mut ResMut<WorldMap>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: (i32, i32),
) {
    if world_map.water_chunk_entities.contains_key(&position) { // if there's a spawned chunk, we remove it
        commands.entity(world_map.water_chunk_entities[&position]).despawn();
        world_map.water_chunk_entities.remove(&position);
    }

    // let material_handle = materials.add(Color::rgba(0.5, 0.5, 1.0, 0.75).into());

    let material_handle = materials.add(
        StandardMaterial {
            base_color: Color::rgba(0.25, 0.5, 1.0, 0.75),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }
    );

    let mesh = generate_water_chunk_mesh(world_map, position);

    let water_chunk = commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh.clone()),
        material: material_handle,
        transform: Transform::from_translation(Vec3::new(position.0 as f32 * CHUNK_WIDTH as f32, 0.0, position.1 as f32  * CHUNK_WIDTH as f32)),
        ..default()
    }).id();

    world_map.water_chunk_entities.insert(position, water_chunk);
}

pub fn build_chunk(
    commands: &mut Commands,
    world_map: &mut ResMut<WorldMap>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    position: (i32, i32),
) -> bool {
    if world_map.chunk_entities.contains_key(&position) { // if there's a spawned chunk, we remove it
        commands.entity(world_map.chunk_entities[&position]).despawn();
        world_map.chunk_entities.remove(&position);
    }

    if world_map.water_chunk_entities.contains_key(&position) {
        commands.entity(world_map.water_chunk_entities[&position]).despawn();
        world_map.water_chunk_entities.remove(&position);
    }

    if world_map.reserved_chunk_data.contains_key(&position) {
        let mut blocks = world_map.chunks[&position];
        for index in 0..CHUNK_VOL {
            if world_map.reserved_chunk_data[&position][index] != BlockType::Air {
                blocks[index] = world_map.reserved_chunk_data[&position][index];
            }
        }
        world_map.chunks.insert(position, blocks);
        world_map.reserved_chunk_data.remove(&position);
    }

    let texture_handle = asset_server.load("blocks.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        unlit: true,
        ..default()
    });

    let mesh = generate_chunk_mesh(world_map, position);

    let chunk = commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(mesh.clone()),
        material: material_handle,
        transform: Transform::from_translation(Vec3::new(position.0 as f32 * CHUNK_WIDTH as f32, 0.0, position.1 as f32  * CHUNK_WIDTH as f32)),
        ..default()
    })
    .insert(Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh).unwrap())
    .insert(Friction {
        coefficient: 0.0,
        combine_rule: CoefficientCombineRule::Min,
    }).id();

    world_map.chunk_entities.insert(position, chunk);

    build_water_chunk(commands, world_map, meshes, materials, position);

    true
}

fn generate_water_block(
    verticies: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    uvs: &mut Vec<Vec2>,
    chunks: &HashMap<(i32,i32), [BlockType; CHUNK_VOL]>,
    block_position: &(i32,i32,i32),
    chunk_position: &(i32, i32),
) {
    let block = block_at_position(chunks, *block_position, *chunk_position);

    let (x,y,z) = (block_position.0 as f32, block_position.1 as f32, block_position.2 as f32);
    if block != BlockType::Water {
        return;
    }

    //top side
    if block_at_position(chunks, (x as i32, y as i32 + 1, z as i32), *chunk_position).is_transparent()
    && block_at_position(chunks, (x as i32, y as i32 + 1, z as i32), *chunk_position) != BlockType::Water {

        verticies.extend([
            [x + 1.0, y + 1.0 - 0.125, z + 1.0],
            [x + 1.0, y + 1.0 - 0.125, z + 0.0],
            [x + 0.0, y + 1.0 - 0.125, z + 0.0],
            [x + 0.0, y + 1.0 - 0.125, z + 1.0]
        ]);
        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().top);
    }
}

fn generate_block(
    verticies: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    uvs: &mut Vec<Vec2>,
    chunks: &HashMap<(i32,i32), [BlockType; CHUNK_VOL]>,
    block_position: &(i32,i32,i32),
    chunk_position: &(i32, i32),
) {
    let block = block_at_position(chunks, *block_position, *chunk_position);

    let (x,y,z) = (block_position.0 as f32, block_position.1 as f32, block_position.2 as f32);
    if block.is_transparent() {
        return;
    }

    // right side
    if block_at_position(chunks, (x as i32 + 1, y as i32, z as i32), *chunk_position).is_transparent() {

        verticies.extend([
            [x + 1.0, y + 1.0, z + 0.0],
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 0.0, z + 1.0],
            [x + 1.0, y + 0.0, z + 0.0],
        ]);

        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().right);
    }

    //left side
    if block_at_position(chunks, (x as i32 - 1, y as i32, z as i32), *chunk_position).is_transparent() {

        verticies.extend([
            [x + 0.0, y + 1.0, z + 1.0],
            [x + 0.0, y + 1.0, z + 0.0],
            [x + 0.0, y + 0.0, z + 0.0],
            [x + 0.0, y + 0.0, z + 1.0]
        ]);

        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().left);
    }

    //back side
    if block_at_position(chunks, (x as i32, y as i32, z as i32 - 1), *chunk_position).is_transparent() {

        verticies.extend([
            [x + 0.0, y + 1.0, z + 0.0],
            [x + 1.0, y + 1.0, z + 0.0],
            [x + 1.0, y + 0.0, z + 0.0],
            [x + 0.0, y + 0.0, z + 0.0],
        ]);

        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().back);
    }

    //front side
    if block_at_position(chunks, (x as i32, y as i32, z as i32 + 1), *chunk_position).is_transparent(){

        verticies.extend([
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 0.0, y + 1.0, z + 1.0],
            [x + 0.0, y + 0.0, z + 1.0],
            [x + 1.0, y + 0.0, z + 1.0],
        ]);

        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().front);
    }

    //bottom side
    if block_at_position(chunks, (x as i32, y as i32 - 1, z as i32), *chunk_position).is_transparent() {

        verticies.extend([
            [x + 0.0, y + 0.0, z + 1.0],
            [x + 0.0, y + 0.0, z + 0.0],
            [x + 1.0, y + 0.0, z + 0.0],
            [x + 1.0, y + 0.0, z + 1.0]
        ]);

        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().bottom);
    }

    //top side
    if block_at_position(chunks, (x as i32, y as i32 + 1, z as i32), *chunk_position).is_transparent() {

        verticies.extend([
            [x + 1.0, y + 1.0, z + 1.0],
            [x + 1.0, y + 1.0, z + 0.0],
            [x + 0.0, y + 1.0, z + 0.0],
            [x + 0.0, y + 1.0, z + 1.0]
        ]);

        add_indices(indices, (verticies.len() - 4) as u32);
        uvs.extend(block.uvs().top);
    }
}

fn add_indices(
    indices: &mut Vec<u32>,
    base_index: u32,
) {
    indices.extend([base_index, base_index + 1, base_index + 2]);
    indices.extend([base_index, base_index + 2, base_index + 3]);
}

fn block_at_position(
    chunks: &HashMap<(i32,i32), [BlockType; CHUNK_VOL]>,
    block_position: (i32, i32, i32),
    chunk_position: (i32, i32),
) -> BlockType {

    let mut new_position: (i32,i32,i32) = block_position;
    let mut new_chunk_position: (i32,i32) = chunk_position;

    if block_position.1 < 0 || block_position.1 > CHUNK_HEIGHT as i32 {
        return BlockType::Dirt;
    }

    if block_position.0 > CHUNK_WIDTH as i32 - 1 {
        new_position.0 = 0;
        new_chunk_position.0 += 1;
    } else if block_position.0 < 0 {
        new_position.0 = CHUNK_WIDTH as i32 - 1;
        new_chunk_position.0 -= 1;
    }

    if block_position.2 > CHUNK_WIDTH as i32 - 1 {
        new_position.2 = 0;
        new_chunk_position.1 += 1;
    } else if block_position.2 < 0 {
        new_position.2 = CHUNK_WIDTH as i32 - 1;
        new_chunk_position.1 -= 1;
    }

    if chunks.contains_key(&new_chunk_position) {

        let index = new_position.0 + new_position.1 * CHUNK_WIDTH as i32 + new_position.2 * (CHUNK_WIDTH * CHUNK_HEIGHT) as i32;
        return chunks[&new_chunk_position][index as usize].clone();
        //return chunks[&new_chunk_position][new_position.0 as usize][new_position.1 as usize][new_position.2 as usize].clone();
    }

    return BlockType::Dirt;
}