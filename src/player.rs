use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct Player {
    velocity: Vec3,
    can_jump: bool,
    camera_pivote: Vec3,
    camera_rotation: Vec2,
    camera_distance: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            can_jump: false,
            camera_pivote: Vec3::ZERO,
            camera_rotation: Vec2::ZERO,
            camera_distance: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct ControlsSettings {
    pub mouse_sensitivity: f32,
    pub movement_speed: f32,
    pub run_speed: f32,
}

impl Default for ControlsSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.25,
            movement_speed: 10.0,
            run_speed: 2.5,
        }
    }
}

enum Mode {
    Free = 0,
    Orbit = 1,
    Fly = 2,
}

#[derive(Component)]
pub struct PlayerCamera {
    mode: Mode,
}

#[derive(Default)]
pub struct PlayerPlugins;

impl Plugin for PlayerPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (update_player, player_movement_result));
        app.add_systems(PostUpdate, update_camera);
        app.init_resource::<ControlsSettings>();
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-20.0, 34.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        FogSettings {
            color: Color::rgba(0.35, 0.35, 0.35, 1.0),
            directional_light_color: Color::rgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::from_visibility_colors(
                32.0 * 15.0,
                Color::rgb(0.35, 0.35, 0.35),
                Color::rgb(0.6, 0.6, 0.6),
            ),
        },
        PlayerCamera { mode: Mode::Free },
    ));

    commands.spawn((
        RigidBody::KinematicPositionBased,
        Collider::capsule(-Vec3::Y / 2.0, Vec3::Y / 2.0, 0.5),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.5,
                depth: 1.0,
                ..default()
            })),
            material: materials.add(Color::rgb_u8(124, 144, 255).into()),
            transform: Transform::from_xyz(32.0, 62.0, 32.0),
            ..default()
        },
        Player {
            camera_pivote: Vec3::ZERO,
            camera_rotation: Vec2::ZERO,
            camera_distance: 15.0,
            ..default()
        },
        KinematicCharacterController {
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(1.5),
                min_width: CharacterLength::Absolute(1.5),
                ..default()
            }),
            ..default()
        },
    ));
}

fn update_camera(
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), With<PlayerCamera>>,
    mut player_query: Query<(&mut Transform, &mut Player), (With<Player>, Without<PlayerCamera>)>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    settings: Res<ControlsSettings>,
    mut yaw: Local<f32>,
    mut pitch: Local<f32>,
) {
    let mut mouse_rel: Vec2 = Vec2::ZERO;

    for event in mouse_motion_events.read() {
        mouse_rel += event.delta;
    }

    let mouse_rel_dt = mouse_rel * time.delta_seconds() * settings.mouse_sensitivity;

    let (mut camera_transform, mut camera) = camera_query.single_mut();

    if player_query.is_empty() {
        return;
    }

    let (player_transform, mut player) = player_query.single_mut();

    let mut window = primary_window.single_mut();

    match camera.mode {
        Mode::Free => {
            if mouse.pressed(MouseButton::Left) {
                camera.mode = Mode::Orbit;

                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }
        }
        Mode::Orbit => {
            if input.pressed(KeyCode::Escape) {
                camera.mode = Mode::Free;

                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            } else if input.just_pressed(KeyCode::F) {
                camera.mode = Mode::Fly;

                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }

            if mouse_rel.length() > 0.0 {
                player.camera_rotation.y -= mouse_rel_dt.x;
                player.camera_rotation.x -= mouse_rel_dt.y;

                player.camera_rotation.x = player.camera_rotation.x.clamp(-1.5, 1.5);
            }

            player.camera_pivote = player_transform.translation
                + player_transform.up() * 1.5
                + Quat::from_rotation_y(player.camera_rotation.y) * player_transform.right() * 0.5;

            camera_transform.translation = player.camera_pivote
                + Quat::from_rotation_y(player.camera_rotation.y)
                    * Quat::from_rotation_x(player.camera_rotation.x)
                    * Vec3::new(0.0, 0.0, player.camera_distance);

            camera_transform.look_at(player.camera_pivote, Vec3::Y);
            (*yaw, *pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
        }
        Mode::Fly => {
            if input.pressed(KeyCode::Escape) {
                camera.mode = Mode::Free;

                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            } else if input.just_pressed(KeyCode::F) {
                camera.mode = Mode::Orbit;

                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }

            if mouse_rel.length() > 0.0 {
                player.camera_rotation.y -= mouse_rel_dt.x;
                player.camera_rotation.x -= mouse_rel_dt.y;

                player.camera_rotation.x = player.camera_rotation.x.clamp(-1.5, 1.5);
            }

            let direction = Vec3 {
                x: input.pressed(KeyCode::D) as i32 as f32
                    - input.pressed(KeyCode::A) as i32 as f32,
                y: input.pressed(KeyCode::Space) as i32 as f32
                    - input.pressed(KeyCode::ShiftLeft) as i32 as f32,
                z: input.pressed(KeyCode::S) as i32 as f32
                    - input.pressed(KeyCode::W) as i32 as f32,
            }
            .normalize();

            let mouse_rel_dt = mouse_rel * time.delta_seconds() * 1.0;

            if mouse_rel.length() > 0.0 {
                *yaw -= mouse_rel_dt.x;
                *pitch -= mouse_rel_dt.y;

                *pitch = pitch.clamp(-1.5, 1.5);

                camera_transform.rotation = Quat::from_euler(EulerRot::YXZ, *yaw, *pitch, 0.0);
            }

            if direction.length() > 0.0 {
                let direction = camera_transform.rotation * direction;
                camera_transform.translation += direction * time.delta_seconds() * 40.0;
            }
        }
    }
}

fn update_player(
    mut player_query: Query<(&mut KinematicCharacterController, &mut Player), With<Player>>,
    mut camera_query: Query<&mut PlayerCamera, With<PlayerCamera>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    settings: Res<ControlsSettings>,
) {
    let camera = camera_query.single_mut();

    match camera.mode {
        Mode::Fly => return,
        _ => (),
    }

    let (mut controller, mut player) = player_query.single_mut();

    let direction = Vec3 {
        x: input.pressed(KeyCode::D) as i32 as f32 - input.pressed(KeyCode::A) as i32 as f32,
        y: 0.0,
        z: input.pressed(KeyCode::S) as i32 as f32 - input.pressed(KeyCode::W) as i32 as f32,
    }
    .normalize();

    let mut movment = Vec3::ZERO;

    if input.pressed(KeyCode::Space) && player.can_jump {
        player.velocity.y = 10.0;

        player.can_jump = false;
    }

    if direction.length() > 0.0 {
        let player_direction = Quat::from_rotation_y(player.camera_rotation.y) * direction;

        movment = player_direction * settings.movement_speed;

        if input.pressed(KeyCode::ShiftLeft) {
            movment *= settings.run_speed;
        }
    }

    player.velocity += Vec3::Y * -20.0 * time.delta_seconds();

    controller.translation = Some((player.velocity + movment) * time.delta_seconds());
}

fn player_movement_result(
    mut controller: Query<(&mut Player, &KinematicCharacterControllerOutput), With<Player>>,
) {
    for (mut player, output) in controller.iter_mut() {
        if output.grounded {
            player.velocity.y = 0.0;
            player.can_jump = true;
        } else {
            player.can_jump = false;
        }
    }
}
