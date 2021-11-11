use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Point3, Vector3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RawLight {
    pub point: [f32; 3],
    pub infinite: [f32; 3],
}

pub struct Light {
    pub point: Point3<f32>,
    pub infinite: Vector3<f32>,
}

impl Light {
    pub fn to_raw_light(&self) -> RawLight {
        RawLight {
            point: self.point.into(),
            infinite: self.infinite.into(),
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Light {
            point: Point3::new(4.0, 4.0, 4.0),
            infinite: Vector3::new(1.0, -1.0, 1.0).normalize(),
        }
    }
}
