struct Uniforms {
    center_position: vec2<f32>,
    radius: f32,
    darkness: f32,
    aspect_ratio: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    const uvs = array<vec2f, 6>(
        vec2(0.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
    );

    let uv = uvs[index];
    let ndc = vec2(2.0 * uv.x, -2.0 * uv.y) + vec2(-1.0, 1.0);

    var out: VertexOutput;
    out.uv = uv;
    out.pos = vec4(ndc, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius_sq = uniforms.radius * uniforms.radius;

    let aspect_ratio_sq = uniforms.aspect_ratio * uniforms.aspect_ratio;
    let x_diff = in.uv.x - uniforms.center_position.x;
    let y_diff = in.uv.y - uniforms.center_position.y;
    let dist_sq = x_diff * x_diff * aspect_ratio_sq + y_diff * y_diff;

    if dist_sq > radius_sq {
        return vec4(0.0, 0.0, 0.0, uniforms.darkness);
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
}
