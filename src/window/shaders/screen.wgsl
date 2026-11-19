struct Uniforms {
    aspect: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) uv: vec2<f32>,
) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position.x * uniforms.aspect, position.yzw);
    output.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return output;
}

@group(0) @binding(1)
var screen: texture_2d<f32>;
@group(0) @binding(2)
var screen_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(screen, screen_sampler, input.uv).rgb, 1.0);
}
