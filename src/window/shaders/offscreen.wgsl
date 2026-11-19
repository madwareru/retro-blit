struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return output;
}

@group(0) @binding(0)
var pixels: texture_2d<f32>;
@group(0) @binding(1)
var nearest_sampler: sampler;
@group(0) @binding(2)
var palette: texture_2d<f32>;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let index = textureSample(pixels, nearest_sampler, input.uv).r;
    let color = textureSample(palette, nearest_sampler, vec2<f32>(index, 0.0)).rgb;
    return vec4<f32>(color, 1.0);
}
