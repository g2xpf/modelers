use wgpu::{
    util::DeviceExt, BindGroupEntry, BindGroupLayoutDescriptor, BlendState, BufferSize,
    BufferUsages, ColorTargetState, ColorWrites, PipelineLayoutDescriptor, VertexAttribute,
};

use crate::{Camera, Context};

mod polygon;
use polygon::{Vertex, INDICES, VERTICES};

use std::borrow::Cow;

pub struct BaseLine {
    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub render_bundle: wgpu::RenderBundle,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl BaseLine {
    pub fn new(ctx: &Context, camera: &Camera) -> Self {
        let index_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            });

        let vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

        let aspect_ratio = ctx.get_aspect_ratio();
        let vp_matrix = camera.create_vp_matrix(aspect_ratio);
        let vp_matrix: &[f32; 16] = vp_matrix.as_ref();

        let uniform_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vp_matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(64),
                    },
                    count: None,
                }],
            });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 4 * 3,
                    shader_location: 1,
                },
            ],
        };

        let shader_module = ctx
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "base_line/base_line.wgsl"
                ))),
            });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[vertex_buffer_layout],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::LineList,
                    strip_index_format: None,
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[ctx.surface_config.format.into()],
                }),
            });

        let index_count = INDICES.len() as u32;

        let mut render_bundle_encoder =
            ctx.device
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: None,
                    color_formats: &[ctx.surface_config.format],
                    depth_stencil: None,
                    sample_count: 1,
                });
        render_bundle_encoder.set_pipeline(&render_pipeline);
        render_bundle_encoder.set_bind_group(0, &bind_group, &[]);
        render_bundle_encoder.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_bundle_encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_bundle_encoder.draw_indexed(0..index_count, 0, 0..1);

        let render_bundle =
            render_bundle_encoder.finish(&wgpu::RenderBundleDescriptor { label: None });

        Self {
            index_buffer,
            vertex_buffer,
            uniform_buffer,
            bind_group,
            render_pipeline,
            render_bundle,
        }
    }
}
