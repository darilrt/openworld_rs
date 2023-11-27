use bevy::prelude::*;

mod chunk;
mod world;

#[derive(Default)]
pub struct VoxelPlugins;

impl Plugin for VoxelPlugins {
    fn build(&self, app: &mut App) {
        app.init_resource::<world::World>();
        app.add_systems(Startup, world::startup);
        app.add_systems(Update, chunk::update);
    }
}
