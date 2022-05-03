struct VertexOutput {
    [[location(0)]] color: vec3<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] a_position: vec3<f32>,
    [[location(1)]] a_color: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = a_color;
    let extended_pos = vec4<f32>(a_position * 1000.0, 1.0);
    out.position = r_locals.transform * extended_pos;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
