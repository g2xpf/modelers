use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub a_position: [f32; 3],
    pub a_color: [f32; 3],
}
const RED: [f32; 3] = [1.0, 0.0, 0.0];
const GREEN: [f32; 3] = [0.0, 1.0, 0.0];
const BLUE: [f32; 3] = [0.0, 0.0, 1.0];

pub const VERTICES: &[Vertex; 6] = &[
    Vertex {
        a_position: [1.0, 0.0, 0.0],
        a_color: RED,
    },
    Vertex {
        a_position: [0.0, 0.0, 0.0],
        a_color: RED,
    },
    Vertex {
        a_position: [0.0, 1.0, 0.0],
        a_color: BLUE,
    },
    Vertex {
        a_position: [0.0, 0.0, 0.0],
        a_color: BLUE,
    },
    Vertex {
        a_position: [0.0, 0.0, 1.0],
        a_color: GREEN,
    },
    Vertex {
        a_position: [0.0, 0.0, 0.0],
        a_color: GREEN,
    },
];

pub const INDICES: &[u16] = &[0, 1, 2, 3, 4, 5];
