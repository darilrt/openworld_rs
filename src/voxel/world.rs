use super::chunk::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;

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
    let block_texture = asset_server.load("textures/block.png");

    const CHUNK_COUNT: i32 = 2;
    for x in 0..CHUNK_COUNT {
        for z in 0..CHUNK_COUNT {
            let id = commands
                .spawn((
                    Chunk::new(IVec3::new(x, 0, z)),
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Plane::from_size(8.0))),
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
                    Collider::compound(vec![(
                        Vec3::new(0.0, 0.0, 0.0),
                        Rot::IDENTITY,
                        Collider::cuboid(1.0, 1.0, 1.0),
                    )]),
                ))
                .id();

            world.set_chunk(IVec3::new(x, 0, z), id);
        }
    }
}
