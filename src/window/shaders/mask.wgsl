struct Uniforms {
    scale: vec2<f32>,
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
    output.position = vec4<f32>(position.xy * uniforms.scale, position.zw);
    output.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return output;
}

@group(0) @binding(1)
var mask: texture_2d<f32>;
@group(0) @binding(2)
var mask_sampler: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(mask, mask_sampler, input.uv);
}
