// Vertex shader

@group(0) @binding(0)
var<uniform> projection_view_model_matrix: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = projection_view_model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_3d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(0)
var<uniform> view_model_matrix_without_model_scale: mat4x4<f32>;
@group(2) @binding(1)
var<uniform> threshold: f32;
@group(2) @binding(2)
var<uniform> sample_step_length: f32;
@group(2) @binding(3)
var<uniform> focal_length: f32;
@group(2) @binding(4)
var<uniform> aspect_ratio: f32;
@group(2) @binding(5)
var<uniform> viewport_size: vec2<f32>;
@group(2) @binding(6)
var<uniform> viewport_position: vec2<f32>;
@group(2) @binding(7)
var<uniform> ray_origin: vec3<f32>;
@group(2) @binding(8)
var<uniform> top_aabb: vec3<f32>;
@group(2) @binding(9)
var<uniform> bottom_aabb: vec3<f32>;
@group(2) @binding(10)
var<uniform> camera_position: vec3<f32>;

fn intersectAABB(ray_origin: vec3<f32>, rayDir: vec3<f32>, boxMin: vec3<f32>, boxMax: vec3<f32>) -> vec2<f32> {
    let tMin: vec3<f32> = (boxMin - ray_origin) / rayDir;
    let tMax: vec3<f32> = (boxMax - ray_origin) / rayDir;
    let t1: vec3<f32> = min(tMin, tMax);
    let t2: vec3<f32> = max(tMin, tMax);
    let tFar: f32 = min(min(t2.x, t2.y), t2.z);
    var tNear: f32 = max(max(t1.x, t1.y), t1.z);
    // clamp tNear to 0.0f, incase the camera is inside the volume
    tNear = max(0.0f, tNear);
    return vec2(tNear, tFar);
}

// since the texture sample depends on @builtin(position), the WGSL spec throws an error
// the diagnostic turn this off
// error: 'textureSample' must only be called from uniform control flow
// https://github.com/gpuweb/gpuweb/pull/3713
// https://www.w3.org/TR/WGSL/#diagnostic-triggering-rule
@diagnostic(off,derivative_uniformity)
fn getVolumeValue(position: vec3<f32>) -> f32 {
	return textureSample(t_diffuse, s_diffuse, position)[0];
}

fn getGradient(position: vec3<f32>) -> vec3<f32> {
	let d: f32 = sample_step_length;
	let top: vec3<f32> = vec3<f32>(getVolumeValue(position + vec3(d, 0.0f, 0.0f)), getVolumeValue(position + vec3(0.0f, d, 0.0f)), getVolumeValue(position + vec3(0.0f, 0.0f, d)));
	let bottom: vec3<f32> = vec3<f32>(getVolumeValue(position - vec3(d, 0.0f, 0.0f)), getVolumeValue(position - vec3(0.0f, d, 0.0f)), getVolumeValue(position - vec3(0.0f, 0.0f, d)));
	return normalize(top - bottom);
}

fn phongShading(ray: vec3<f32>, position: vec3<f32>, lightPosition: vec3<f32>) -> vec3<f32> {
	// Blinn-Phong shading
	let Ka: vec3<f32> = vec3(0.1, 0.1, 0.1); // ambient
	let Kd: vec3<f32> = vec3(0.6, 0.6, 0.6); // diffuse
	let Ks: vec3<f32> = vec3(0.2, 0.2, 0.2); // specular
	let shininess: f32 = 100.0;

	// light properties
	let lightColor: vec3<f32> = vec3(1.0, 1.0, 1.0);
	let ambientLight: vec3<f32> = vec3(0.3, 0.3, 0.3);

	let L: vec3<f32> = normalize(lightPosition - position);
	let V: vec3<f32> = -normalize(ray);
	let N: vec3<f32> = getGradient(position);
	let H: vec3<f32> = normalize(L + V);

	// Compute ambient term
	let ambient: vec3<f32> = Ka * ambientLight;
	// Compute the diffuse term
	let diffuseLight: f32 = max(dot(L, N), 0.0);
	let diffuse: vec3<f32> = Kd * lightColor * diffuseLight;
	// Compute the specular term
    let specularLight = mix(0.0, pow(max(dot(H, N), 0.0), shininess), f32(diffuseLight > 0.0));
	let specular: vec3<f32> = Ks * lightColor * specularLight;
	return ambient + diffuse + specular;
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    // let position = vec3<f32>(0.5, 0.5, 0.5);
    // let value = textureSample(t_diffuse, s_diffuse, position)[0];
    // return vec4<f32>(value, 0.0, 0.0, 1.0);

	let local_frag_coord: vec2<f32> = frag_coord.xy - viewport_position;

    var ray_direction_xy: vec2<f32> = (2.0 * local_frag_coord.xy / viewport_size - 1.0);

	ray_direction_xy.x *= aspect_ratio;
	var ray_direction: vec3<f32> = vec3<f32>(ray_direction_xy, -focal_length);
	ray_direction = (vec4<f32>(ray_direction, 0.0) * view_model_matrix_without_model_scale).xyz;

	let intersection: vec2<f32> = intersectAABB(ray_origin, ray_direction, bottom_aabb, top_aabb);

	var ray_start: vec3<f32> = (ray_origin + ray_direction * intersection.x + top_aabb) / (top_aabb - bottom_aabb);
	var ray_stop: vec3<f32> = (ray_origin + ray_direction * intersection.y + top_aabb) / (top_aabb - bottom_aabb);

	var ray: vec3<f32> = ray_stop - ray_start;
	var step_vector: vec3<f32> = normalize(ray) * sample_step_length;

	// Random jitter
	// vec3 jitter = step_vector * texture(noiseTex, gl_FragCoord.xy / viewport_size).r;
    let jitter: vec3<f32> =  vec3<f32>(0.0, 0.0, 0.0);
	var position: vec3<f32> = ray_start + jitter;

	let steps: i32 = i32(length(ray - jitter) / sample_step_length);

	// Ray march until reaching the end of the volume
	var intensity: f32 = 0.0;
    var i : i32 = 0;
    var firstHit : vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    loop {
        if (i > steps) {
            break;
        }

        intensity = getVolumeValue(position);

        if (intensity >= threshold) {
            var intensity_color: f32 = 0.0;
            if (intensity > 0.0) {
                intensity_color = 0.5;
            }
            firstHit = vec3<f32>(intensity_color, intensity_color, intensity_color);
            break;
        }

        position += step_vector;
        i += 1;
    }

	// for (int i = 0; i <= steps; i++) {
	// 	intensity = getVolumeValue(position);

	// 	if(intensity >= threshold) {
	// 		firstHit = vec3((intensity > 0.0f) ? 0.5f : 0.0f);
	// 		break;
    // 	}

	// 	position += step_vector;
	// }

	// Phong Shading
    firstHit += 0.5 * phongShading(ray, position, camera_position);

    var value: vec4<f32>;

    if (intensity >= threshold) {
        value = vec4<f32>(firstHit, 1.0);
    }
    else {
        value = vec4<f32>(firstHit, 0.0);
    }

	// value.x = frag_coord.x / viewport_size.x;
	// value.y = frag_coord.y / viewport_size.y;


	// return vec4<f32>(local_frag_coord.x / viewport_size.x, local_frag_coord.y / viewport_size.y, 1.0, 1.0);

	// maybe the display dpi affects the egui box size but not the wgpu frag coordinates

	// return vec4<f32>(local_frag_coord.x / viewport_size.x, local_frag_coord.y / viewport_size.y, 1.0, 1.0);

	return value;

	// gl_FragDepth = length(position - ray_start) / gl_FragCoord.w;

}
