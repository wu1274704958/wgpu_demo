struct VertexOutput{
    [[builtin(position)]] clip_position : vec4<f32>;
    [[location(0)]] pos : vec2<f32>;
};

[[stage(vertex)]]
fn main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput
{
    var out: VertexOutput;
        let x = f32(1 - i32(in_vertex_index)) * 0.5;
        let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
        out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
        out.pos = vec2<f32>(x,y);
        return out;
}

[[stage(fragment)]]
fn main(v:VertexOutput) -> [[location(0)]] vec4<f32>
{
    return vec4<f32>(v.pos, 0.1, 1.0);
}


