use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component)]
pub struct FPSCameraTag;

#[derive(Bundle)]
pub struct FPSCamera {
    pub camera: Camera3dBundle,
    pub tag: FPSCameraTag,
}

impl Default for FPSCamera {
    fn default() -> Self {
        Self {
            camera: Camera3dBundle::default(),
            tag: FPSCameraTag,
        }
    }
}

#[derive(Default)]
pub struct FPSCameraPlugins;

impl Plugin for FPSCameraPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, move_camera);
    }
}

pub fn move_camera(
    mut query: Query<&mut Transform, With<FPSCameraTag>>,
    input: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    time: Res<Time>,
    mut yaw: Local<f32>,
    mut pitch: Local<f32>,
) {
    let direction = Vec3 {
        x: input.pressed(KeyCode::D) as i32 as f32 - input.pressed(KeyCode::A) as i32 as f32,
        y: input.pressed(KeyCode::Space) as i32 as f32
            - input.pressed(KeyCode::ShiftLeft) as i32 as f32,
        z: input.pressed(KeyCode::S) as i32 as f32 - input.pressed(KeyCode::W) as i32 as f32,
    }
    .normalize();

    // get mouse relative movement x and y
    let mut mouse_rel: Vec2 = Vec2::ZERO;

    for event in mouse_motion_events.read() {
        mouse_rel += event.delta;
    }

    if !(mouse_rel.length() > 0.0 || direction.length() > 0.0) {
        return;
    }

    let mouse_rel_dt = mouse_rel * time.delta_seconds() * 1.0;

    for mut transform in &mut query {
        if mouse.pressed(MouseButton::Right) && mouse_rel.length() > 0.0 {
            *yaw -= mouse_rel_dt.x;
            *pitch -= mouse_rel_dt.y;

            *pitch = pitch.clamp(-1.5, 1.5);

            transform.rotation = Quat::from_euler(EulerRot::YXZ, *yaw, *pitch, 0.0);
        }

        if direction.length() > 0.0 {
            let direction = transform.rotation * direction;
            transform.translation += direction * time.delta_seconds() * 20.0;
        }
    }
}
