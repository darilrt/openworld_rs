#import bevy_pbr::forward_io::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: CustomMaterial;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return material.color;
}