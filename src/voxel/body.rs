use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Component)]
pub struct Body {
    body_box: BodyBox,
}

impl Default for Body {
    fn default() -> Self {
        Self {
            body_box: BodyBox::new(),
        }
    }
}

#[derive(Default)]
pub struct BodyBox {
    parent: Option<Arc<BodyBox>>,
    children: Vec<Arc<BodyBox>>,

    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    pivot: Vec3,

    uv: Vec<[[f32; 2]; 6]>,
}

impl BodyBox {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),

            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            pivot: Vec3::ZERO,

            uv: Vec::new(),
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BodyMaterial {
    #[uniform(0)]
    color: Color,
    alpha_mode: AlphaMode,
}

impl Material for BodyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

pub fn build(app: &mut App) {
    app.add_plugins(MaterialPlugin::<BodyMaterial>::default());
}
