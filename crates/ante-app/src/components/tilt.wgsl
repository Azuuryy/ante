// Written for Ante: the hover tilt of the original card shaders. Their `position()` stage
// adds to each vertex's clip-space w a term that grows with its squared distance to the
// cursor (in room units) and shrinks toward the screen centre, so a hovered card seems to
// lean toward the pointer. Only `hovering > 0` moves anything.

struct ViewPrefix {
    clip_from_world: mat4x4<f32>,
};

struct Tilt {
    cursor: vec2<f32>,
    screen: vec2<f32>,
    hovering: f32,
    screen_scale: f32,
};

@group(0) @binding(0) var<uniform> view: ViewPrefix;
@group(1) @binding(3) var<uniform> tilt: Tilt;

struct VertexOut {
    @location(0) uv: vec2<f32>,
    @location(1) border_widths: vec4<f32>,
    @location(2) border_radius_x: vec4<f32>,
    @location(3) border_radius_y: vec4<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) border_widths: vec4<f32>,
    @location(4) border_radius_x: vec4<f32>,
    @location(5) border_radius_y: vec4<f32>,
) -> VertexOut {
    var out: VertexOut;
    out.uv = vertex_uv;
    out.size = size;
    out.border_widths = border_widths;
    out.border_radius_x = border_radius_x;
    out.border_radius_y = border_radius_y;
    var position = view.clip_from_world * vec4<f32>(vertex_position, 1.0);
    if (tilt.hovering > 0.0) {
        let mid_dist = length(vertex_position.xy - 0.5 * tilt.screen) / length(tilt.screen);
        let offset = (vertex_position.xy - tilt.cursor) / tilt.screen_scale;
        let depth = 0.2 * (-0.03 - 0.3 * max(0.0, 0.3 - mid_dist)) * tilt.hovering
            * dot(offset, offset) / (2.0 - mid_dist);
        position.w = position.w + depth;
    }
    out.position = position;
    return out;
}
