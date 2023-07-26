// Vertex shader

@group(1) @binding(0)
var<uniform> scale_factor: vec3<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = 0.5 * vec2(model.position.x, model.position.y) + 0.5;
    out.clip_position = vec4<f32>(scale_factor * model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_3d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(0)
var<uniform> slice_position: f32;
@group(2) @binding(1)
var<uniform> axis: i32; // 0 = x, 1 = y, 2 = z

fn get_value(position: vec2<f32>) -> vec3<f32> {
    var value: vec3<f32>;
    if (axis == 0) {
        value = vec3<f32>(position.x, position.y, slice_position);
    }
    else if (axis == 1) {
        value = vec3<f32>(position.x, slice_position, position.y);
    }
    else if (axis == 2) {
        value = vec3<f32>(slice_position, position.x, position.y);
    }
    return value;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let position = in.tex_coords;
    let value = textureSample(t_diffuse, s_diffuse, get_value(position))[0];
    return vec4<f32>(value, value, value, 1.0);
}
