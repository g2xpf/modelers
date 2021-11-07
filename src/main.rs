use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use cgmath::{Matrix4, Point3, Vector3};

use std::borrow::Cow;

use futures::executor;
use wgpu::util;
use wgpu::util::DeviceExt;
use wgpu::{
    Adapter, Backends, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
    BindingType, BufferBindingType, BufferSize, BufferUsages, Device, DeviceDescriptor, Extent3d,
    Features, Instance, Limits, PipelineLayoutDescriptor, PresentMode, Queue, ShaderSource,
    ShaderStages, Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor,
};

use std::mem;

use std::time::{Duration, Instant};

use modelers::{Vertex, INDICES, VERTICES};

struct Ctx {
    event_loop: EventLoop<()>,
    window: Window,
    adapter: Adapter,
    instance: Instance,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    surface: Surface,
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

fn main() {
    env_logger::init();

    let ctx = executor::block_on(create_ctx());

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

    let vp_matrix = create_vp(ctx.size.width as f32 / ctx.size.height as f32);
    let vp_matrix: &[f32; 16] = vp_matrix.as_ref();

    let uniform_buffer = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(vp_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
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
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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

    let pipeline = ctx
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

    let pipeline_wire = if ctx.device.features().contains(Features::POLYGON_MODE_LINE) {
        let pipeline_wire = ctx
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
            });
        Some(pipeline_wire)
    } else {
        None
    };

    let index_count = INDICES.len() as u32;

    let mut last_update_inst = Instant::now();
    let mut last_frame_inst = Instant::now();
    let (mut frame_count, mut accum_time) = (0, 0.0);

    ctx.event_loop.run(move |event, _, control_flow| {
        let _ = (&ctx.instance, &ctx.adapter); // force ownership by the closure
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => {
                    let vp_matrix = create_vp(size.width as f32 / size.height as f32);
                    let vp_matrix: &[f32; 16] = vp_matrix.as_ref();
                    ctx.queue
                        .write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(vp_matrix));
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                accum_time += last_frame_inst.elapsed().as_secs_f32();
                last_frame_inst = Instant::now();
                frame_count += 1;
                if frame_count == 100 {
                    println!(
                        "Avg frame time {}ms",
                        accum_time * 1000.0 / frame_count as f32
                    );
                    accum_time = 0.0;
                    frame_count = 0;
                }

                let frame = match ctx.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        ctx.surface.configure(&ctx.device, &config);
                        ctx.surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.push_debug_group("Prepare data for draw.");
                    rpass.set_pipeline(&pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.pop_debug_group();
                    rpass.insert_debug_marker("Draw!");
                    rpass.draw_indexed(0..index_count, 0, 0..1);
                    if let Some(pipe) = &pipeline_wire {
                        rpass.set_pipeline(pipe);
                        rpass.draw_indexed(0..index_count, 0, 0..1);
                    }
                }

                ctx.queue.submit(Some(encoder.finish()));

                frame.present();
            }
            Event::RedrawEventsCleared => {
                let target_frametime = Duration::from_secs_f64(1.0 / 60.0);
                let time_since_last_frame = last_update_inst.elapsed();
                if time_since_last_frame >= target_frametime {
                    ctx.window.request_redraw();
                    last_update_inst = Instant::now();
                } else {
                    *control_flow = ControlFlow::WaitUntil(
                        Instant::now() + target_frametime - time_since_last_frame,
                    );
                }
            }
            _ => {}
        }
    });
}

async fn create_ctx() -> Ctx {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).expect("Failed to create window");

    #[cfg(target_os = "macos")]
    let backends = Backends::METAL;
    #[cfg(not(target_os = "macos"))]
    let backends = Backends::VULKAN;

    let instance = Instance::new(backends);
    let size = window.inner_size();
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = util::initialize_adapter_from_env_or_default(&instance, backends, Some(&surface))
        .await
        .expect("No suitable GPU adapters found on the system!");

    let optional_features = Features::POLYGON_MODE_LINE;
    let required_features = Features::empty();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );
    let features = (optional_features & adapter_features) | required_features;

    let needed_limits = Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());

    let trace_dir = std::env::var("TRACE_DIR");
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .unwrap_or_else(|error| panic!("Failed to create device: {}", error));

    Ctx {
        window,
        event_loop,
        device,
        queue,
        instance,
        adapter,
        size,
        surface,
    }
}

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

fn create_vp(aspect_ratio: f32) -> Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
    let mx_view = cgmath::Matrix4::look_at_rh(
        // Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0.0, 0.0, 5.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_y(),
    );
    let mx_correction = OPENGL_TO_WGPU_MATRIX;
    mx_correction * mx_projection * mx_view
}
