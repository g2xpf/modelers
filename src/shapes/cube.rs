use crate::Context;
use std::borrow::Cow;
use std::mem;

use wgpu::util::DeviceExt;

mod polygon;
use polygon::{Vertex, INDICES, VERTICES};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, BufferBindingType, BufferSize, BufferUsages, Extent3d, Features,
    PipelineLayoutDescriptor, PresentMode, RenderBundle, RenderBundleDescriptor,
    RenderBundleEncoderDescriptor, RenderPipeline, ShaderSource, ShaderStages,
    SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}

pub struct Cube {
    pub texture: Texture,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub bind_group: BindGroup,
    pub pipeline_cube: RenderPipeline,
    pub pipeline_wire: Option<RenderPipeline>,
    pub num_indicies: usize,
    pub render_bundle: RenderBundle,
}

impl Cube {
    pub fn new(ctx: &Context) -> Self {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: ctx.surface.get_preferred_format(&ctx.adapter).unwrap(),
            width: ctx.size.width,
            height: ctx.size.height,
            present_mode: PresentMode::Mailbox,
        };

        ctx.surface.configure(&ctx.device, &config);

        let vertex_size = mem::size_of::<Vertex>();
        let vertex_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX,
            });

        let index_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX,
            });

        let bind_group_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        // create texture
        let size = 256u32;
        let texels = create_texels(size as usize);
        let texture_extent = Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let texture = ctx.device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        ctx.queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(size).unwrap()),
                rows_per_image: None,
            },
            texture_extent,
        );

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ctx.global_ubo.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        let shader = ctx
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("cube/cube.wgsl"))),
            });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline_cube = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[config.format.into()],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
            });

        let pipeline_wire = ctx
            .device
            .features()
            .contains(Features::POLYGON_MODE_LINE)
            .then(|| {
                ctx.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &vertex_buffers,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_wire",
                            targets: &[wgpu::ColorTargetState {
                                format: config.format,
                                blend: Some(wgpu::BlendState {
                                    color: wgpu::BlendComponent {
                                        operation: wgpu::BlendOperation::Add,
                                        src_factor: wgpu::BlendFactor::SrcAlpha,
                                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                    },
                                    alpha: wgpu::BlendComponent::REPLACE,
                                }),
                                write_mask: wgpu::ColorWrites::ALL,
                            }],
                        }),
                        primitive: wgpu::PrimitiveState {
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            polygon_mode: wgpu::PolygonMode::Line,
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                    })
            });

        let num_indicies = INDICES.len();

        let render_bundle = Self::create_render_bundle(
            ctx,
            &pipeline_cube,
            pipeline_wire.as_ref(),
            &bind_group,
            &index_buffer,
            &vertex_buffer,
            num_indicies,
        );

        Cube {
            texture,
            index_buffer,
            vertex_buffer,
            pipeline_cube,
            pipeline_wire,
            bind_group,
            render_bundle,
            num_indicies,
        }
    }

    pub fn create_render_bundle(
        ctx: &Context,
        pipeline_cube: &RenderPipeline,
        pipeline_wire: Option<&RenderPipeline>,
        bind_group: &BindGroup,
        index_buffer: &Buffer,
        vertex_buffer: &Buffer,
        num_indicies: usize,
    ) -> RenderBundle {
        let mut render_bundle_encoder =
            ctx.device
                .create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                    label: None,
                    color_formats: &[ctx.surface_config.format],
                    depth_stencil: None,
                    sample_count: 1,
                });

        render_bundle_encoder.set_pipeline(pipeline_cube);
        render_bundle_encoder.set_bind_group(0, bind_group, &[]);
        render_bundle_encoder.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_bundle_encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_bundle_encoder.draw_indexed(0..(num_indicies as u32), 0, 0..1);

        if let Some(pipe) = pipeline_wire {
            render_bundle_encoder.set_pipeline(pipe);
            render_bundle_encoder.draw_indexed(0..(num_indicies as u32), 0, 0..1);
        }

        render_bundle_encoder.finish(&RenderBundleDescriptor { label: None })
    }
}
