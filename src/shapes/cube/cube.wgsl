struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

fn create_vertex_out(position: vec3<f32>, uv: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.uv = uv;
    out.position = r_locals.transform * vec4<f32>(position, 1.0);
    return out;
}

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] a_position: vec3<f32>,
    [[location(1)]] a_uv: vec2<f32>,
) -> VertexOutput {
    return create_vertex_out(a_position, a_uv);
}

[[group(1), binding(0)]]
var r_color: texture_2d<u32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let tex = textureLoad(r_color, vec2<i32>(in.uv * 256.0), 0);
    let v = f32(tex.x) / 255.0;
    return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);
}

[[stage(fragment)]]
fn fs_wire() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}

