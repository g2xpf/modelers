use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub a_pos: [f32; 3],
    pub a_uv: [f32; 2],
}

pub const VERTICES: &[Vertex] = &[
    // top (0, 0, 1)
    Vertex {
        a_pos: [-1.0, -1.0, 1.0],
        a_uv: [0.0, 0.0],
    },
    Vertex {
        a_pos: [1.0, -1.0, 1.0],
        a_uv: [1.0, 0.0],
    },
    Vertex {
        a_pos: [1.0, 1.0, 1.0],
        a_uv: [1.0, 1.0],
    },
    Vertex {
        a_pos: [-1.0, 1.0, 1.0],
        a_uv: [0.0, 1.0],
    },
    // bottom (0.0, 0.0, -1.0)
    Vertex {
        a_pos: [-1.0, 1.0, -1.0],
        a_uv: [1.0, 0.0],
    },
    Vertex {
        a_pos: [1.0, 1.0, -1.0],
        a_uv: [0.0, 0.0],
    },
    Vertex {
        a_pos: [1.0, -1.0, -1.0],
        a_uv: [0.0, 1.0],
    },
    Vertex {
        a_pos: [-1.0, -1.0, -1.0],
        a_uv: [1.0, 1.0],
    },
    // right (1.0, 0.0, 0.0)
    Vertex {
        a_pos: [1.0, -1.0, -1.0],
        a_uv: [0.0, 0.0],
    },
    Vertex {
        a_pos: [1.0, 1.0, -1.0],
        a_uv: [1.0, 0.0],
    },
    Vertex {
        a_pos: [1.0, 1.0, 1.0],
        a_uv: [1.0, 1.0],
    },
    Vertex {
        a_pos: [1.0, -1.0, 1.0],
        a_uv: [0.0, 1.0],
    },
    // left (-1.0, 0.0, 0.0)
    Vertex {
        a_pos: [-1.0, -1.0, 1.0],
        a_uv: [1.0, 0.0],
    },
    Vertex {
        a_pos: [-1.0, 1.0, 1.0],
        a_uv: [0.0, 0.0],
    },
    Vertex {
        a_pos: [-1.0, 1.0, -1.0],
        a_uv: [0.0, 1.0],
    },
    Vertex {
        a_pos: [-1.0, -1.0, -1.0],
        a_uv: [1.0, 1.0],
    },
    // front (0.0, 1.0, 0.0)
    Vertex {
        a_pos: [1.0, 1.0, -1.0],
        a_uv: [1.0, 0.0],
    },
    Vertex {
        a_pos: [-1.0, 1.0, -1.0],
        a_uv: [0.0, 0.0],
    },
    Vertex {
        a_pos: [-1.0, 1.0, 1.0],
        a_uv: [0.0, 1.0],
    },
    Vertex {
        a_pos: [1.0, 1.0, 1.0],
        a_uv: [1.0, 1.0],
    },
    // back (0.0, -1.0, 0.0)
    Vertex {
        a_pos: [1.0, -1.0, 1.0],
        a_uv: [0.0, 0.0],
    },
    Vertex {
        a_pos: [-1.0, -1.0, 1.0],
        a_uv: [1.0, 0.0],
    },
    Vertex {
        a_pos: [-1.0, -1.0, -1.0],
        a_uv: [1.0, 1.0],
    },
    Vertex {
        a_pos: [1.0, -1.0, -1.0],
        a_uv: [0.0, 1.0],
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
