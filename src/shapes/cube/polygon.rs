use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub a_pos: [f32; 3],
    pub a_uv: [f32; 2],
    pub a_normal: [f32; 3],
}

const TOP: [f32; 3] = [0.0, 0.0, 1.0];
const BOTTOM: [f32; 3] = [0.0, 0.0, -1.0];
const RIGHT: [f32; 3] = [1.0, 0.0, 0.0];
const LEFT: [f32; 3] = [-1.0, 0.0, 0.0];
const FRONT: [f32; 3] = [0.0, 1.0, 0.0];
const BACK: [f32; 3] = [0.0, -1.0, 0.0];

pub const VERTICES: &[Vertex] = &[
    // top (0, 0, 1)
    Vertex {
        a_pos: [-1.0, -1.0, 1.0],
        a_uv: [0.0, 0.0],
        a_normal: TOP,
    },
    Vertex {
        a_pos: [1.0, -1.0, 1.0],
        a_uv: [1.0, 0.0],
        a_normal: TOP,
    },
    Vertex {
        a_pos: [1.0, 1.0, 1.0],
        a_uv: [1.0, 1.0],
        a_normal: TOP,
    },
    Vertex {
        a_pos: [-1.0, 1.0, 1.0],
        a_uv: [0.0, 1.0],
        a_normal: TOP,
    },
    // bottom (0.0, 0.0, -1.0)
    Vertex {
        a_pos: [-1.0, 1.0, -1.0],
        a_uv: [1.0, 0.0],
        a_normal: BOTTOM,
    },
    Vertex {
        a_pos: [1.0, 1.0, -1.0],
        a_uv: [0.0, 0.0],
        a_normal: BOTTOM,
    },
    Vertex {
        a_pos: [1.0, -1.0, -1.0],
        a_uv: [0.0, 1.0],
        a_normal: BOTTOM,
    },
    Vertex {
        a_pos: [-1.0, -1.0, -1.0],
        a_uv: [1.0, 1.0],
        a_normal: BOTTOM,
    },
    // right (1.0, 0.0, 0.0)
    Vertex {
        a_pos: [1.0, -1.0, -1.0],
        a_uv: [0.0, 0.0],
        a_normal: RIGHT,
    },
    Vertex {
        a_pos: [1.0, 1.0, -1.0],
        a_uv: [1.0, 0.0],
        a_normal: RIGHT,
    },
    Vertex {
        a_pos: [1.0, 1.0, 1.0],
        a_uv: [1.0, 1.0],
        a_normal: RIGHT,
    },
    Vertex {
        a_pos: [1.0, -1.0, 1.0],
        a_uv: [0.0, 1.0],
        a_normal: RIGHT,
    },
    // left (-1.0, 0.0, 0.0)
    Vertex {
        a_pos: [-1.0, -1.0, 1.0],
        a_uv: [1.0, 0.0],
        a_normal: LEFT,
    },
    Vertex {
        a_pos: [-1.0, 1.0, 1.0],
        a_uv: [0.0, 0.0],
        a_normal: LEFT,
    },
    Vertex {
        a_pos: [-1.0, 1.0, -1.0],
        a_uv: [0.0, 1.0],
        a_normal: LEFT,
    },
    Vertex {
        a_pos: [-1.0, -1.0, -1.0],
        a_uv: [1.0, 1.0],
        a_normal: LEFT,
    },
    // front (0.0, 1.0, 0.0)
    Vertex {
        a_pos: [1.0, 1.0, -1.0],
        a_uv: [1.0, 0.0],
        a_normal: FRONT,
    },
    Vertex {
        a_pos: [-1.0, 1.0, -1.0],
        a_uv: [0.0, 0.0],
        a_normal: FRONT,
    },
    Vertex {
        a_pos: [-1.0, 1.0, 1.0],
        a_uv: [0.0, 1.0],
        a_normal: FRONT,
    },
    Vertex {
        a_pos: [1.0, 1.0, 1.0],
        a_uv: [1.0, 1.0],
        a_normal: FRONT,
    },
    // back (0.0, -1.0, 0.0)
    Vertex {
        a_pos: [1.0, -1.0, 1.0],
        a_uv: [0.0, 0.0],
        a_normal: BACK,
    },
    Vertex {
        a_pos: [-1.0, -1.0, 1.0],
        a_uv: [1.0, 0.0],
        a_normal: BACK,
    },
    Vertex {
        a_pos: [-1.0, -1.0, -1.0],
        a_uv: [1.0, 1.0],
        a_normal: BACK,
    },
    Vertex {
        a_pos: [1.0, -1.0, -1.0],
        a_uv: [0.0, 1.0],
        a_normal: BACK,
    },
];
pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];
