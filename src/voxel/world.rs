use std::{
    collections::LinkedList,
    thread::{self},
};

use crate::player::Player;

use super::chunk::*;
use super::type_map::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use noise::NoiseFn;

#[derive(Clone)]
pub struct ChunkState {
    pub entity: Option<Entity>,
    pub mesh: Handle<Mesh>,
    pub collider: Collider,
    pub chunk: Chunk,
    pub is_showing: bool,
}

#[derive(Resource)]
pub struct World {
    pub chunks: Map<IVec3, ChunkState>,
    pub chunks_to_load: LinkedList<IVec3>,
    pub chunks_to_unload: LinkedList<IVec3>,
    pub chunks_threads: LinkedList<thread::JoinHandle<(Chunk, Mesh, Collider)>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: Map::new(),
            chunks_to_load: LinkedList::new(),
            chunks_to_unload: LinkedList::new(),
            chunks_threads: LinkedList::new(),
        }
    }

    pub fn load_chunk(&mut self, position: IVec3) {
        let pos = self.chunks_to_load.iter().position(|x| x.eq(&position));

        if pos.is_some() {
            return;
        }

        self.chunks_to_load.push_back(position);
    }
}

pub fn startup(mut world: ResMut<World>) {
    world.load_chunk(IVec3::new(0, 0, 0));
}

pub fn update(
    mut commands: Commands,
    mut chunks_query: Query<(&mut Chunk, Entity)>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut world: ResMut<World>,
) {
    if player_query.is_empty() {
        return;
    }

    let transform = player_query.single_mut();

    for (chunk, entity) in chunks_query.iter_mut() {
        let pos = chunk.get_world_position();

        if pos.distance(transform.translation) > 32.0 * 15.0 {
            let pos = chunk.position;

            if let Some(state) = world.chunks.get_mut(&pos) {
                state.entity = None;
                state.is_showing = false;
                commands.entity(entity).despawn_recursive();
            } else {
                world.chunks.remove(&pos);
            }
        }
    }
}

pub fn load_chunks(
    mut world: ResMut<World>,
    mut commands: Commands,
    mut asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if world.chunks_to_load.is_empty() {
        return;
    }

    while world.chunks_to_load.len() > 0 {
        let chunk_pos = world.chunks_to_load.pop_front().unwrap();

        if world.chunks.contains_key(&chunk_pos) {
            if world.chunks.contains_key(&chunk_pos) {
                let mut chunk_state = world.chunks.get_mut(&chunk_pos).unwrap().clone();

                if chunk_state.is_showing {
                    continue;
                }

                chunk_state.entity = Some(spawn_chunk(
                    &mut commands,
                    &mut asset_server,
                    &mut materials,
                    chunk_state.clone(),
                ));
                chunk_state.is_showing = true;

                world.chunks.insert(chunk_state.chunk.position, chunk_state);
                return;
            }
            continue;
        }

        let handle = thread::spawn(move || {
            let mut chunk = generate_chunk_data(chunk_pos.x, chunk_pos.z);
            let (mesh, collider) = build_chunk_mesh(&chunk);
            chunk.updated = true;
            (chunk, mesh, collider)
        });

        world.chunks.insert(
            chunk_pos,
            ChunkState {
                entity: None,
                mesh: Handle::default(),
                collider: Collider::default(),
                chunk: Chunk::new(chunk_pos),
                is_showing: false,
            },
        );

        world.chunks_threads.push_back(handle);
    }
}

pub fn unload_chunks(mut commands: Commands, mut world: ResMut<World>) {
    if world.chunks_to_unload.is_empty() {
        return;
    }

    for _ in 0..world.chunks_to_unload.len() {
        let pos = world.chunks_to_unload.pop_front().unwrap();

        if let Some(state) = world.chunks.get_mut(&pos) {
            if let Some(entity) = state.entity {
                commands.entity(entity).despawn_recursive();
                state.is_showing = false;
                state.entity = None;
            }
        }
    }
    world.chunks_to_unload.clear();
}

pub fn generate_chunks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut world: ResMut<World>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    if world.chunks_threads.is_empty() {
        return;
    }

    let mut threads_unfinished: LinkedList<thread::JoinHandle<(Chunk, Mesh, Collider)>> =
        LinkedList::new();

    while world.chunks_threads.len() > 0 {
        let thread = world.chunks_threads.pop_front().unwrap();

        if !thread.is_finished() {
            threads_unfinished.push_back(thread);
            break;
        }

        let (chunk, mesh, collider) = thread.join().unwrap();

        let mut chunk_state = ChunkState {
            entity: None,
            mesh: meshes.add(mesh),
            collider,
            chunk,
            is_showing: true,
        };

        chunk_state.entity = Some(spawn_chunk(
            &mut commands,
            &mut asset_server,
            &mut materials,
            chunk_state.clone(),
        ));

        world.chunks.insert(chunk_state.chunk.position, chunk_state);
    }

    world.chunks_threads.append(&mut threads_unfinished);
}

fn generate_chunk_data(x: i32, z: i32) -> Chunk {
    let noise = noise::Perlin::new(21744033);

    let mut chunk = Chunk::new(IVec3::new(x, 0, z));
    let global_pos: Vec3 = Vec3::new(
        x as f32 * CHUNK_SIZE as f32 + 0.5,
        0.0,
        z as f32 * CHUNK_SIZE as f32 + 0.5,
    );

    for cx in 0..CHUNK_SIZE {
        for cy in 0..CHUNK_SIZE {
            for cz in 0..CHUNK_SIZE {
                let pos: Vec3 = Vec3::new(
                    cx as f32 + global_pos.x,
                    cy as f32,
                    cz as f32 + global_pos.z,
                );

                let mut npos: Vec2 = Vec2::new(pos.x, pos.z) / 100.0;

                let mut h = noise.get([npos.x as f64, npos.y as f64]);

                npos *= 0.50;
                let factor = noise.get([npos.x as f64, npos.y as f64]);

                h = (h * factor) * 32.0;

                chunk.set_block(
                    cx,
                    cy,
                    cz,
                    if pos.y - 16.0 < h as f32 {
                        if pos.y < 10.0 {
                            1
                        } else {
                            2
                        }
                    } else {
                        0
                    },
                );
            }
        }
    }

    chunk
}

pub fn debug(world: ResMut<World>, mut contexts: EguiContexts) {
    egui::Window::new("World Diagnostics").show(&contexts.ctx_mut(), |ui| {
        ui.label(format!("Chunks: {}", world.chunks.len()));
        ui.label(format!("Chunks to load: {}", world.chunks_to_load.len()));
        ui.label(format!("Chunks threads: {}", world.chunks_threads.len()));
        ui.label(format!(
            "Chunks to unload: {}",
            world.chunks_to_unload.len()
        ));
    });
}
