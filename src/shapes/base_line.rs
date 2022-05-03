use wgpu::{util::DeviceExt, BufferUsages, PipelineLayoutDescriptor, VertexAttribute};

use crate::Context;

mod polygon;
use polygon::{Vertex, INDICES, VERTICES};

use std::borrow::Cow;

pub struct BaseLine {
    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub render_bundle: wgpu::RenderBundle,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl BaseLine {
    pub fn new(ctx: &Context) -> Self {
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
                label: Some("base_line/base_line.wgsl"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "base_line/base_line.wgsl"
                ))),
            });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&ctx.global.bind_group_layout],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[ctx.surface_config.format.into()],
                }),
                multiview: None,
            });

        let index_count = INDICES.len() as u32;

        let mut render_bundle_encoder =
            ctx.device
                .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: None,
                    color_formats: &[ctx.surface_config.format],
                    depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                        format: wgpu::TextureFormat::Depth32Float,
                        depth_read_only: false,
                        stencil_read_only: true,
                    }),
                    sample_count: 1,
                    ..wgpu::RenderBundleEncoderDescriptor::default()
                });
        render_bundle_encoder.set_pipeline(&render_pipeline);
        render_bundle_encoder.set_bind_group(0, &ctx.global.bind_group, &[]);
        render_bundle_encoder.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_bundle_encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_bundle_encoder.draw_indexed(0..index_count, 0, 0..1);

        let render_bundle =
            render_bundle_encoder.finish(&wgpu::RenderBundleDescriptor { label: None });

        Self {
            index_buffer,
            vertex_buffer,
            render_pipeline,
            render_bundle,
        }
    }
}
