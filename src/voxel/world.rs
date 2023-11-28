use std::{
    collections::LinkedList,
    thread::{self},
};

use super::chunk::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::prelude::*;
use noise::NoiseFn;

#[derive(Resource)]
pub struct World {
    pub chunks: HashMap<IVec3, Entity>,
    pub chunks_to_load: LinkedList<IVec3>,
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
            chunks: HashMap::default(),
            chunks_to_load: LinkedList::default(),
            chunks_threads: LinkedList::default(),
        }
    }

    pub fn set_chunk_ref(&mut self, position: IVec3, entity: Entity) {
        self.chunks.insert(position, entity);
    }

    pub fn load_chunk(&mut self, position: IVec3) {
        if self.chunks_to_load.contains(&position) || self.chunks.contains_key(&position) {
            return;
        }

        self.chunks_to_load.push_back(position);
    }
}

pub fn startup(mut world: ResMut<World>) {
    world.load_chunk(IVec3::new(0, 0, 0));
}

fn generate_chunk(x: i32, z: i32) -> Chunk {
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

                let npos: Vec2 = Vec2::new(pos.x, pos.z) / 100.0;

                let noise = noise.get([npos.x as f64, npos.y as f64]);
                chunk.blocks[cx][cy][cz] = if pos.y - 16.0 < noise as f32 * 10.0 {
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

        let x = chunk.position.x;
        let y = chunk.position.y;
        let z = chunk.position.z;

        let id: Entity = commands
            .spawn((
                chunk,
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials
                        .add(StandardMaterial {
                            base_color: Color::rgb(0.9, 0.9, 0.9),
                            base_color_texture: Some(block_texture.clone()),
                            ..default()
                        })
                        .into(),
                    transform: Transform::from_xyz(
                        x as f32 * CHUNK_SIZE as f32,
                        0.0,
                        z as f32 * CHUNK_SIZE as f32,
                    ),
                    ..default()
                },
                collider,
            ))
            .id();

        world.set_chunk_ref(IVec3 { x, y, z }, id);
    }

    if world.chunks_to_load.front().is_none() {
        return;
    }

    let pos = world.chunks_to_load.pop_front().unwrap();

    let handle = thread::spawn(move || {
        let mut chunk = generate_chunk(pos.x, pos.z);
        let (mesh, collider) = build_chunk_mesh(&chunk);
        chunk.updated = true;
        (chunk, mesh, collider)
    });

    world.chunks_threads.push_back(handle);
}

pub fn debug(mut world: ResMut<World>, mut contexts: EguiContexts) {
    egui::Window::new("World Diagnostics").show(&contexts.ctx_mut(), |ui| {
        ui.label(format!("Chunks: {}", world.chunks.len()));
        ui.label(format!("Chunks to load: {}", world.chunks_to_load.len()));
        ui.label(format!("Chunks threads: {}", world.chunks_threads.len()));
    });
}
