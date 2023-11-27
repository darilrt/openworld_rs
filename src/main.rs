use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use camera::*;
use voxel::*;

mod camera;
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
            FPSCameraPlugins::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(FPSCamera {
        camera: Camera3dBundle {
            transform: Transform::from_xyz(-20.0, 34.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        ..default()
    });

    commands.spawn((
        RigidBody::Dynamic,
        Collider::capsule(-Vec3::Y, Vec3::Y, 1.0),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 1.0,
                depth: 2.0,
                ..default()
            })),
            material: materials.add(Color::rgb_u8(124, 144, 255).into()),
            transform: Transform::from_xyz(5.0, 30.0, 5.0),
            ..default()
        },
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-0.7 * std::f32::consts::PI)),
        directional_light: DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
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

    let delta_seconds = time.delta_seconds_f64();
    if delta_seconds == 0.0 {
        return;
    }
}
