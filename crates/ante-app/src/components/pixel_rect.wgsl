// Written for Ante: a box with the stepped corners of the original UI, an emboss lip and an
// optional drop shadow. Sizes are physical pixels; `geom.x` is one 1/20 room unit.

struct PixelParams {
    fill: vec4<f32>,
    emboss: vec4<f32>,
    shadow: vec4<f32>,
    // unit, emboss height (units), shadow offset x and y (units)
    geom: vec4<f32>,
    outline: vec4<f32>,
    // outline width in physical pixels
    line: vec4<f32>,
};

@group(1) @binding(0) var<uniform> params: PixelParams;

struct FragmentIn {
    @location(0) uv: vec2<f32>,
    @location(4) @interpolate(flat) size: vec2<f32>,
};

// Inside a rectangle whose corners step 4, 2 and 1 steps in from each edge.
fn inside(p: vec2<f32>, size: vec2<f32>, step: f32) -> bool {
    if p.x < 0.0 || p.y < 0.0 || p.x >= size.x || p.y >= size.y {
        return false;
    }
    let q = min(p, size - p) / step;
    if q.y < 1.0 && q.x < 4.0 {
        return false;
    }
    if q.y < 2.0 && q.x < 2.0 {
        return false;
    }
    if q.y < 4.0 && q.x < 1.0 {
        return false;
    }
    return true;
}

@fragment
fn fragment(in: FragmentIn) -> @location(0) vec4<f32> {
    let unit = max(params.geom.x, 0.0001);
    let p = in.uv * in.size;
    let emboss = params.geom.y * unit;
    let shadow = params.geom.zw * unit;
    let body = max(in.size - vec2<f32>(abs(shadow.x), shadow.y + emboss), vec2<f32>(0.0));
    let origin = vec2<f32>(max(-shadow.x, 0.0), 0.0);
    // `res` follows the smaller side of the box in room units.
    let side = min(body.x, body.y) / (20.0 * unit);
    var res = 0.15;
    if side > 3.5 {
        res = 0.8;
    } else if side > 0.3 {
        res = 0.6;
    }
    let step = res * unit;
    if inside(p - origin, body, step) {
        let w = params.line.x;
        if w > 0.0 && !inside(p - origin - vec2<f32>(w), body - vec2<f32>(2.0 * w), step) {
            return params.outline;
        }
        return params.fill;
    }
    if emboss > 0.0 && inside(p - origin - vec2<f32>(0.0, emboss), body, step) {
        return params.emboss;
    }
    if params.shadow.a > 0.0 && inside(p - origin - shadow, body + vec2<f32>(0.0, emboss), step) {
        return params.shadow;
    }
    return vec4<f32>(0.0);
}
