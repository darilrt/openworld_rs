use std::{
    thread::{self, ScopedJoinHandle},
    time::Instant,
};

use super::chunk::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;
use noise::NoiseFn;

#[derive(Resource)]
pub struct World {
    chunks: HashMap<IVec3, Entity>,
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
        }
    }

    pub fn set_chunk(&mut self, position: IVec3, entity: Entity) {
        self.chunks.insert(position, entity);
    }
}

pub fn startup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<World>,
) {
    const CHUNK_COUNT: i32 = 10;
    let total_chunks = CHUNK_COUNT * CHUNK_COUNT;

    let block_texture: Handle<Image> = asset_server.load("textures/block.png");

    let now = Instant::now();

    thread::scope(|s| {
        let mut threads: Vec<ScopedJoinHandle<'_, (Chunk, Mesh, Collider)>> =
            Vec::with_capacity(total_chunks as usize);

        for i in 0..total_chunks {
            let x: i32 = i % CHUNK_COUNT;
            let z: i32 = i / CHUNK_COUNT;

            let spawn: ScopedJoinHandle<'_, (Chunk, Mesh, Collider)> = s.spawn(move || {
                let mut chunk: Chunk = generate_chunk(x, z);
                let (mesh, collider) = build_chunk_mesh(&chunk);
                chunk.updated = true;
                (chunk, mesh, collider)
            });
            threads.push(spawn);
        }

        for ele in threads {
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

            world.set_chunk(IVec3 { x, y, z }, id);
        }
    });

    println!("Generated {} chunks in {:?}", total_chunks, now.elapsed());
}

fn generate_chunk(x: i32, z: i32) -> Chunk {
    let noise = noise::Perlin::new(21744032);

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
