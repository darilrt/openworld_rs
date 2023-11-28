use bevy::prelude::*;

mod chunk;
mod world;

pub use chunk::Chunk;
pub use chunk::CHUNK_SIZE;
pub use world::World;

#[derive(Default)]
pub struct VoxelPlugins;

impl Plugin for VoxelPlugins {
    fn build(&self, app: &mut App) {
        app.init_resource::<world::World>();
        app.add_systems(Startup, world::startup);
        app.add_systems(Update, world::update);
        app.add_systems(Update, world::debug);
        app.add_systems(Update, chunk::update);
    }
}
