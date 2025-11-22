struct Uniforms {
    spotlight_color: vec4<f32>,
    dvd_logo_color: vec4<f32>,
    canvas_size: vec2<f32>,
    image_size: vec2<f32>,
    image_offset: vec2<f32>,
    cursor_position: vec2<f32>,
    dvd_logo_position: vec2<f32>,
    dvd_logo_size: vec2<f32>,
    zoom_factor: f32,
    spotlight_radius_multiplier: f32,
    dvd_logo_visible: u32,
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

@group(2) @binding(0)
var dvd_logo_texture: texture_2d<f32>;
@group(2) @binding(1)
var dvd_logo_sampler: sampler;

const UNIT_RADIUS: f32 = 0.1;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var texel_color = textureSample(image_texture, image_sampler, in.tex_coord);

    if uniforms.dvd_logo_visible != 0 && coordInDvdRectangle(in.tex_coord * uniforms.image_size) {
        let dvd_logo_position =
            (in.tex_coord - uniforms.dvd_logo_position / uniforms.image_size)
            / uniforms.dvd_logo_size
            * uniforms.image_size;

        let dvd_texel_color = textureSample(dvd_logo_texture, dvd_logo_sampler, dvd_logo_position);

        let dvd_color = uniforms.dvd_logo_color * dvd_texel_color.a;

        texel_color = vec4f(
            dvd_color.rgb + texel_color.rgb * (1.0 - dvd_color.a),
            texel_color.a + dvd_color.a * (1.0 - texel_color.a)
        );
    }

    let image_transform = mat2x2<f32>(
        uniforms.image_size.x / uniforms.image_size.y, 0.0,
        0.0, 1.0
    );

    let local_cursor_position =
        (uniforms.cursor_position - uniforms.image_offset)
        / uniforms.image_size
        / uniforms.zoom_factor;

    let actual_distance_to_cursor = distance(
        image_transform * local_cursor_position,
        image_transform * in.tex_coord
    );

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

fn coordInDvdRectangle(coord: vec2<f32>) -> bool {
    return (coord.x > uniforms.dvd_logo_position.x)
        && (coord.x < uniforms.dvd_logo_position.x + uniforms.dvd_logo_size.x)
        && (coord.y > uniforms.dvd_logo_position.y)
        && (coord.y < uniforms.dvd_logo_position.y + uniforms.dvd_logo_size.y);
}
