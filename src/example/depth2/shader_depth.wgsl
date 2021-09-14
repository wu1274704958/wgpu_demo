struct VertexOutput{
    [[builtin(position)]] clip_position : vec4<f32>;
    [[location(0)]] color : vec4<f32>;
    [[location(1)]] uv : vec2<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] uv: vec2<f32>;
};

[[block]] // 1.
struct Uniforms {
    projection: mat4x4<f32>;
    view: mat4x4<f32>;
    model: mat4x4<f32>;
};
[[group(1), binding(0)]] // 2.
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main(in : VertexInput) -> VertexOutput
{
    var out: VertexOutput;
    out.clip_position = uniforms.projection * uniforms.view * uniforms.model * vec4<f32>(in.position,1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}


[[group(0), binding(0)]]
var t_shadow: texture_depth_2d;
[[group(0), binding(1)]]
var s_shadow: sampler;

[[stage(fragment)]]
fn main(v:VertexOutput) -> [[location(0)]] vec4<f32>
{
    let depth = textureSample(t_shadow, s_shadow, v.uv);
    return vec4<f32>(depth);
}


