struct Uniforms {
    canvas_size: vec2<f32>,
    image_size: vec2<f32>,
    image_offset: vec2<f32>,
    cursor_position: vec2<f32>,
    spotlight_color: vec4<f32>,
    zoom_factor: f32,
    spotlight_radius_multiplier: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 0.0),
    );

    let scaled_size = uniforms.image_size * uniforms.zoom_factor;
    let window_pos = uniforms.image_offset + uvs[vertex_index] * scaled_size;

    let pos_ndc = uvToNdc(window_pos / uniforms.canvas_size);

    var out: VertexOutput;
    out.position = vec4<f32>(pos_ndc, 0.0, 1.0);
    out.tex_coord = uvs[vertex_index];
    return out;
}

@group(1) @binding(0)
var image_texture: texture_2d<f32>;
@group(1) @binding(1)
var image_sampler: sampler;

const UNIT_RADIUS: f32 = 0.1;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel_color = textureSample(image_texture, image_sampler, in.tex_coord);

    let proportionality_transform = mat2x2<f32>(
        uniforms.canvas_size.x / uniforms.canvas_size.y, 0.0,
        0.0, 1.0
    );

    let local_cursor_position = (uniforms.cursor_position - uniforms.image_offset) / uniforms.canvas_size / uniforms.zoom_factor;

    let actual_distance_to_cursor = distance(proportionality_transform * local_cursor_position, proportionality_transform * in.tex_coord);

    let radius = UNIT_RADIUS * uniforms.spotlight_radius_multiplier / uniforms.zoom_factor;

    if actual_distance_to_cursor > radius {
        return mix(texel_color, vec4<f32>(uniforms.spotlight_color.rgb, 1.0), uniforms.spotlight_color.a);
    } else {
        return texel_color;
    }
}

fn ndcToUv(ndc: vec2<f32>) -> vec2<f32> {
    return (vec2<f32>(ndc.x, -ndc.y) + vec2<f32>(1.0, 1.0)) / 2.0;
}

fn uvToNdc(uv: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(uv.x, -uv.y) * 2.0 - vec2<f32>(1.0, -1.0);
}

