// Written for Ante: draws one atlas cell of a card sprite, tinted, under the tilt vertex
// stage of `tilt.wgsl`.

struct Face {
    rect: vec4<f32>,
    tint: vec4<f32>,
};

@group(1) @binding(0) var<uniform> face: Face;
@group(1) @binding(1) var image: texture_2d<f32>;
@group(1) @binding(2) var image_sampler: sampler;

@fragment
fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let coords = face.rect.xy + uv * face.rect.zw;
    return textureSample(image, image_sampler, coords) * face.tint;
}
