struct VertexOutput{
    [[builtin(position)]] clip_position : vec4<f32>;
    [[location(0)]] color : vec4<f32>;
};

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn main(in : VertexInput) -> VertexOutput
{
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position,1.0);
    out.color = in.color;
    return out;
}

[[stage(fragment)]]
fn main(v:VertexOutput) -> [[location(0)]] vec4<f32>
{
    return v.color;
}


