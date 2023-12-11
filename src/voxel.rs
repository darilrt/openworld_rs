use bevy::prelude::*;

pub mod body;
mod chunk;
mod type_map;
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

        body::build(app);

        app.add_systems(Update, world::update);
        app.add_systems(Update, world::unload_chunks);
        app.add_systems(Update, world::load_chunks);
        app.add_systems(Update, world::generate_chunks);
        app.add_systems(Update, world::debug);
    }
}
