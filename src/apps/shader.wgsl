// Vertex shader

@group(1) @binding(0)
var<uniform> view_projection_matrix: mat4x4<f32>;

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
    out.tex_coords = model.tex_coords;
    out.clip_position = view_projection_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_3d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(0)
var<uniform> slice_position: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let position = in.tex_coords;
    let value = textureSample(t_diffuse, s_diffuse, vec3<f32>(position.x, position.y, slice_position));
    return vec4<f32>(value[0], value[0], value[0], 1.0);
}
