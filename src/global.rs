mod camera;
mod light;

use std::mem;

use bytemuck::{Pod, Zeroable};
pub use camera::{Camera, RawCamera};
pub use light::{Light, RawLight};

pub use wgpu::util::DeviceExt;

use winit::dpi::PhysicalSize;

pub struct Global {
    pub camera: Camera,
    pub light: Light,

    pub ubo: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlobalUniforms {
    vp_matrix: [f32; 16],
    camera_pos: [f32; 3],
    _0: f32,
    point_light_pos: [f32; 3],
    _1: f32,
    inf_light_dir: [f32; 3],
    _2: f32,
}

impl Global {
    const VP_MATRIX_SIZE: usize = mem::size_of::<[f32; 16]>();
    pub fn new(device: &wgpu::Device, size: PhysicalSize<u32>) -> Self {
        let camera = Camera::default();
        let light = Light::default();
        let raw_camera = camera.create_raw_camera(size.width as f32 / size.height as f32);
        let raw_light = light.to_raw_light();

        let global_uniforms = GlobalUniforms {
            vp_matrix: raw_camera.vp_matrix,
            camera_pos: raw_camera.camera_pos,
            _0: 0.0,
            point_light_pos: raw_light.point,
            _1: 0.0,
            inf_light_dir: raw_light.infinite,
            _2: 0.0,
        };

        log::info!("size: {}", bytemuck::bytes_of(&global_uniforms).len());

        let ubo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&global_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<GlobalUniforms>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ubo.as_entire_binding(),
            }],
            layout: &bind_group_layout,
        });

        Global {
            camera,
            light,
            bind_group,
            bind_group_layout,
            ubo,
        }
    }

    pub fn on_resize(&self, queue: &wgpu::Queue, size: PhysicalSize<u32>) {
        let raw_camera = self
            .camera
            .create_raw_camera(size.width as f32 / size.height as f32);
        queue.write_buffer(&self.ubo, 0, bytemuck::bytes_of(&raw_camera.vp_matrix));
        queue.write_buffer(
            &self.ubo,
            Self::VP_MATRIX_SIZE as u64,
            bytemuck::bytes_of(&raw_camera.camera_pos),
        );
    }
}
