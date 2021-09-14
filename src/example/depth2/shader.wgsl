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
    //model: mat4x4<f32>;
};

struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
};

[[group(1), binding(0)]] // 2.
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn main(in : VertexInput,instance: InstanceInput) -> VertexOutput
{
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.clip_position = uniforms.projection * uniforms.view * model_matrix * vec4<f32>(in.position,1.0);
    out.color = in.color;
    out.uv = in.uv;
    return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn main(v:VertexOutput) -> [[location(0)]] vec4<f32>
{
    return textureSample(t_diffuse,s_diffuse,v.uv);
}


