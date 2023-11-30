use std::{
    collections::LinkedList,
    thread::{self},
};

use super::chunk::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use noise::NoiseFn;

#[derive(Clone)]
pub struct ChunkState {
    entity: Option<Entity>,
    mesh: Handle<Mesh>,
    collider: Collider,
    chunk: Chunk,
    is_showing: bool,
}

#[derive(Resource)]
pub struct World {
    pub chunks: HashMap<IVec3, ChunkState>,
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
            chunks: HashMap::new(),
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

    pub fn unload_chunk(&mut self, position: IVec3) {
        if !self.chunks.contains_key(&position) {
            return;
        }

        self.chunks_to_unload.push_back(position);
    }
}

pub fn startup(mut world: ResMut<World>) {
    world.load_chunk(IVec3::new(0, 0, 0));
}

pub fn update(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<World>,
) {
    let block_texture: Handle<Image> = asset_server.load("textures/block.png");

    if world.chunks_threads.len() > 0 && world.chunks_threads.front().unwrap().is_finished() {
        let ele = world.chunks_threads.pop_front().unwrap();

        let (chunk, mesh, collider) = ele.join().unwrap();
        let mut chunk_state = ChunkState {
            entity: None,
            mesh: meshes.add(mesh),
            collider,
            chunk,
            is_showing: true,
        };

        let state = chunk_state.clone();
        let transform = Transform::from_translation(Vec3::new(
            state.chunk.position.x as f32 * CHUNK_SIZE as f32,
            state.chunk.position.y as f32 * CHUNK_SIZE as f32,
            state.chunk.position.z as f32 * CHUNK_SIZE as f32,
        ));

        chunk_state.entity = Some(
            commands
                .spawn((
                    state.chunk,
                    PbrBundle {
                        mesh: state.mesh,
                        material: materials
                            .add(StandardMaterial {
                                base_color: Color::rgb(0.9, 0.9, 0.9),
                                base_color_texture: Some(block_texture.clone()),
                                ..default()
                            })
                            .into(),
                        transform: transform,
                        ..default()
                    },
                    state.collider,
                ))
                .id(),
        );
        world.chunks.insert(chunk_state.chunk.position, chunk_state);
    }

    if world.chunks_to_load.front().is_none() {
        return;
    }
}

pub fn load_chunks(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<World>,
) {
    let block_texture: Handle<Image> = asset_server.load("textures/block.png");

    let world: &mut World = &mut world;

    for pos in world.chunks_to_load.iter() {
        if world.chunks.contains_key(pos) {
            if let Some(state) = world.chunks.get_mut(pos) {
                if !state.is_showing {
                    let transform = Transform::from_translation(Vec3::new(
                        state.chunk.position.x as f32 * CHUNK_SIZE as f32,
                        state.chunk.position.y as f32 * CHUNK_SIZE as f32,
                        state.chunk.position.z as f32 * CHUNK_SIZE as f32,
                    ));

                    let chunk_state = state.clone();

                    state.entity = Some(
                        commands
                            .spawn((
                                chunk_state.chunk,
                                PbrBundle {
                                    mesh: chunk_state.mesh,
                                    material: materials
                                        .add(StandardMaterial {
                                            base_color: Color::rgb(0.9, 0.9, 0.9),
                                            base_color_texture: Some(block_texture.clone()),
                                            ..default()
                                        })
                                        .into(),
                                    transform: transform,
                                    ..default()
                                },
                                chunk_state.collider,
                            ))
                            .id(),
                    );

                    state.is_showing = true;
                }
            }
            continue;
        }

        let pos = *pos;

        let handle = thread::spawn(move || {
            let mut chunk = generate_chunk(pos.x, pos.z);
            let (mesh, collider) = build_chunk_mesh(&chunk);
            chunk.updated = true;
            (chunk, mesh, collider)
        });

        world.chunks_threads.push_back(handle);
    }

    world.chunks_to_load.clear();
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

fn generate_chunk(x: i32, z: i32) -> Chunk {
    let noise = noise::Perlin::new(21744033);

    let chunk = Chunk::new(IVec3::new(x, 0, z));
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

                let npos: Vec2 = Vec2::new(pos.x, pos.z) / 100.0;

                let noise = noise.get([npos.x as f64, npos.y as f64]);
                chunk.blocks.as_ref().write()[cx][cy][cz] = if pos.y - 16.0 < noise as f32 * 10.0 {
                    if pos.y < 10.0 {
                        1
                    } else {
                        2
                    }
                } else {
                    0
                };
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
