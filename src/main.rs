use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use futures::executor;

use modelers::{Camera, Context, LoopClock, RenderConfig};

fn main() {
    env_logger::init();

    let (mut ctx, event_loop) = executor::block_on(Context::create_context());
    let mut camera = Camera::default();
    let config = RenderConfig::new(&ctx, &camera);

    let mut loop_clock = LoopClock::start_clock(60.0);

    event_loop.run(move |event, _, control_flow| match event {
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
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(keycode),
                        state,
                        ..
                    },
                ..
            } => {
                let should_do = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };
                match keycode {
                    VirtualKeyCode::W => camera.move_forward(should_do),
                    VirtualKeyCode::A => camera.move_left(should_do),
                    VirtualKeyCode::S => camera.move_backward(should_do),
                    VirtualKeyCode::D => camera.move_right(should_do),
                    VirtualKeyCode::LShift => camera.move_down(should_do),
                    VirtualKeyCode::Space => camera.move_up(should_do),
                    VirtualKeyCode::H => camera.turn_left(should_do),
                    VirtualKeyCode::L => camera.turn_right(should_do),
                    VirtualKeyCode::J => camera.look_down(should_do),
                    VirtualKeyCode::K => camera.look_up(should_do),
                    _ => {}
                }
            }
            WindowEvent::Resized(size)
            | WindowEvent::ScaleFactorChanged {
                new_inner_size: &mut size,
                ..
            } => {
                ctx.surface_config.width = size.width;
                ctx.surface_config.height = size.height;
                ctx.size = size;

                ctx.recreate_surface();
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            camera.update();
        }
        Event::RedrawEventsCleared => {
            if let Some(wait_instant) = loop_clock.get_wait_duration() {
                *control_flow = ControlFlow::WaitUntil(wait_instant);
            } else {
                ctx.window.request_redraw();
            }
        }
        Event::RedrawRequested(_) => {
            if let Some(average_frametime) = loop_clock.tick() {
                log::info!("average: {}ms", average_frametime);
            }

            let vp_matrix = camera.create_vp_matrix(ctx.get_aspect_ratio());
            let vp_matrix: &[f32; 16] = vp_matrix.as_ref();
            ctx.queue
                .write_buffer(&config.uniform_buffer, 0, bytemuck::cast_slice(vp_matrix));

            let frame = match ctx.surface.get_current_texture() {
                Ok(frame) => frame,
                Err(_) => {
                    ctx.recreate_surface();
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
                rpass.set_pipeline(&config.pipeline_cube);
                rpass.set_bind_group(0, &config.bind_group, &[]);
                rpass.set_index_buffer(config.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, config.vertex_buffer.slice(..));
                rpass.pop_debug_group();
                rpass.insert_debug_marker("Draw!");
                rpass.draw_indexed(0..(config.num_indicies as u32), 0, 0..1);
                if let Some(pipe) = &config.pipeline_wire {
                    rpass.set_pipeline(pipe);
                    rpass.draw_indexed(0..(config.num_indicies as u32), 0, 0..1);
                }
            }

            ctx.queue.submit(Some(encoder.finish()));

            frame.present();
        }
        _ => {}
    });
}
