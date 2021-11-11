[[block]]
struct GlobalUniforms {
    vp_matrix: mat4x4<f32>;
    camera_position: vec3<f32>;
    point_light_position: vec3<f32>;
    inf_light_direction: vec3<f32>;
};

[[block]]
struct LocalUniforms {
    model_matrix: mat4x4<f32>;
    model_matrix_inverted: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> u_global: GlobalUniforms;
[[group(1), binding(1)]]
var<uniform> u_local: LocalUniforms;


struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] pos: vec3<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] a_position: vec3<f32>,
    [[location(1)]] a_uv: vec2<f32>,
    [[location(2)]] a_normal: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = a_uv;
    out.normal = (u_local.model_matrix_inverted * vec4<f32>(a_normal, 1.0)).xyz;
    out.position =  u_global.vp_matrix * u_local.model_matrix * vec4<f32>(a_position, 1.0);
    out.pos = a_position;
    return out;
}

[[group(1), binding(0)]]
var r_color: texture_2d<u32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let tex = textureLoad(r_color, vec2<i32>(in.uv * 256.0), 0);
    let v = f32(tex.x) / 255.0;
    let color = vec3<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0));

    let normal = normalize(in.normal);
    let incidence = normalize(in.pos - u_global.point_light_position);
    let reflected = reflect(incidence, normal);
    let half_vector = normalize(incidence + in.pos - u_global.camera_position);

    let ambient = color * vec3<f32>(0.4);
    let diffuse = color * max(dot(-incidence, normal), 0.0);
    let specular = pow(max(dot(-half_vector, normal), 0.0), 50.0);

    return vec4<f32>(ambient + diffuse + specular, 1.0);
    // return vec4<f32>(specular, 1.0);
    // return vec4<f32>(color, 1.0);
}

[[stage(fragment)]]
fn fs_wire() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}

