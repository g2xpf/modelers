use crate::Context;
use std::f32::consts::PI;
use std::io::Cursor;
use std::mem;
use std::{borrow::Cow, mem::size_of_val};

use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix, Matrix4, Rad, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

mod polygon;
use polygon::{Vertex, INDICES, VERTICES};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, BufferUsages, Extent3d, Face, Features, PipelineLayoutDescriptor, RenderBundle,
    RenderBundleDescriptor, RenderBundleEncoderDescriptor, RenderPipeline, ShaderSource,
    ShaderStages, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureViewDescriptor,
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CubeUniforms {
    model_matrix: [f32; 16],
    model_matrix_inverted_transposed: [f32; 16],
}

pub struct Cube {
    pub model_matrix: Matrix4<f32>,

    pub texture: Texture,
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
    pub pipeline_cube: RenderPipeline,
    pub pipeline_wire: Option<RenderPipeline>,
    pub num_indicies: usize,
    pub render_bundle: RenderBundle,
}

impl Cube {
    pub fn new(ctx: &Context) -> Self {
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

        // create texture
        let raw_image = include_bytes!("cube/Wood_Floor_011_basecolor.png");
        let decoder = png::Decoder::new(Cursor::new(raw_image));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let image_buffer = &buf[..info.buffer_size()];

        let texture_format = match info {
            png::OutputInfo {
                bit_depth,
                color_type: png::ColorType::Rgba,
                ..
            } => match bit_depth {
                png::BitDepth::Eight => Some(TextureFormat::Rgba8Uint),
                png::BitDepth::Sixteen => Some(TextureFormat::Rgba16Uint),
                _ => None,
            },
            _ => None,
        };
        let texture_format = texture_format.unwrap_or_else(|| panic!("info: {:?}", info));

        // let texels = create_texels(size as usize);
        let texture_extent = Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: 1,
        };
        let texture = ctx.device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        ctx.queue.write_texture(
            texture.as_image_copy(),
            image_buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(info.width * 4 /* Rgba */).unwrap()),
                rows_per_image: None,
            },
            texture_extent,
        );

        let model_matrix = Matrix4::identity();

        let uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: mem::size_of::<[f32; 32]>() as u64,
            mapped_at_creation: false,
        });

        Self::update_inner(&ctx.queue, &uniform_buffer, model_matrix);

        let bind_group_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<[f32; 32]>() as u64
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&ctx.global.bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
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
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of_val(&VERTICES[0].a_pos) as u64,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: size_of_val(&VERTICES[0].a_pos) as u64
                        + size_of_val(&VERTICES[0].a_uv) as u64,
                    shader_location: 2,
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
                    targets: &[ctx.surface_config.format.into()],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
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
                                format: ctx.surface_config.format,
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
                            cull_mode: Some(Face::Back),
                            polygon_mode: wgpu::PolygonMode::Line,
                            ..Default::default()
                        },
                        depth_stencil: Some(wgpu::DepthStencilState {
                            format: TextureFormat::Depth32Float,
                            depth_write_enabled: false,
                            depth_compare: wgpu::CompareFunction::Always,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        }),
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
            model_matrix,
            texture,
            index_buffer,
            vertex_buffer,
            pipeline_cube,
            pipeline_wire,
            bind_group,
            render_bundle,
            num_indicies,
            uniform_buffer,
        }
    }

    fn update_inner(queue: &wgpu::Queue, uniform_buffer: &Buffer, model_matrix: Matrix4<f32>) {
        let raw_model_matrix: &[f32; 16] = model_matrix.as_ref();
        queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(raw_model_matrix));
        let model_matrix_inverted_transposed = model_matrix
            .invert()
            .expect("failed to calculate inverse matrix of Cube rotation")
            .transpose();
        let raw_model_matrix_inverted_transposed: &[f32; 16] =
            model_matrix_inverted_transposed.as_ref();
        queue.write_buffer(
            uniform_buffer,
            mem::size_of::<[f32; 16]>() as u64,
            bytemuck::cast_slice(raw_model_matrix_inverted_transposed),
        );
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let delta_rotation =
            Matrix4::from_axis_angle(Vector3::new(1.0, 1.0, 1.0).normalize(), Rad(PI / 180.0));
        self.model_matrix = delta_rotation * self.model_matrix;
        Self::update_inner(queue, &self.uniform_buffer, self.model_matrix);
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
                    depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                        format: TextureFormat::Depth32Float,
                        depth_read_only: false,
                        stencil_read_only: true,
                    }),
                    sample_count: 1,
                });

        render_bundle_encoder.set_pipeline(pipeline_cube);
        render_bundle_encoder.set_bind_group(0, &ctx.global.bind_group, &[]);
        render_bundle_encoder.set_bind_group(1, bind_group, &[]);
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
