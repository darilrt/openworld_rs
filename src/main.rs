use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::pbr::CascadeShadowConfig;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use player::*;
use voxel::*;

mod player;
mod voxel;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Hello, world!".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
            WorldInspectorPlugin::new(),
            // FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
            VoxelPlugins::default(),
            PlayerPlugins::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-0.7 * std::f32::consts::PI)),
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfig {
            bounds: vec![50.0, 100.0, 200.0, 500.0],
            ..default()
        },
        ..default()
    });
}

fn update(time: Res<Time>, mut contexts: EguiContexts, frame_count: Res<bevy::core::FrameCount>) {
    egui::Window::new("Diagnostics").show(&contexts.ctx_mut(), |ui| {
        ui.label(format!("Frame count: {}", frame_count.0));
        ui.label(format!(
            "Frame time: {:.2} ms",
            time.delta_seconds_f64() * 1000.0
        ));
        ui.label(format!("FPS: {:.2}", 1.0 / time.delta_seconds_f64()));
    });
}
