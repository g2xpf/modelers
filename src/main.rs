use modelers::shapes::BaseLine;
use modelers::LoopClock;
use wgpu::Operations;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use futures::executor;

use modelers::{shapes::Cube, Context};

use std::io::Write;

fn main() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let style = buf.default_level_style(record.level());
            let level = style.value(record.level());
            let args = record.args();
            let time = buf.timestamp_nanos();
            writeln!(
                buf,
                "[{time} {level}] {args}",
                time = time,
                level = level,
                args = args
            )
        })
        .init();

    let (mut ctx, event_loop) = executor::block_on(Context::create_context());
    let base_line = BaseLine::new(&ctx);
    let mut cube = Cube::new(&ctx);

    let mut loop_clock = LoopClock::start_clock(60.0);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
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
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state,
                            ..
                        },
                    ..
                } => {
                    log::info!("KeyboardInput");
                    let should_do = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };
                    match keycode {
                        VirtualKeyCode::W => ctx.global.camera.move_forward(should_do),
                        VirtualKeyCode::A => ctx.global.camera.move_left(should_do),
                        VirtualKeyCode::S => ctx.global.camera.move_backward(should_do),
                        VirtualKeyCode::D => ctx.global.camera.move_right(should_do),
                        VirtualKeyCode::LShift => ctx.global.camera.move_down(should_do),
                        VirtualKeyCode::Space => ctx.global.camera.move_up(should_do),
                        VirtualKeyCode::H => ctx.global.camera.turn_left(should_do),
                        VirtualKeyCode::L => ctx.global.camera.turn_right(should_do),
                        VirtualKeyCode::J => ctx.global.camera.look_down(should_do),
                        VirtualKeyCode::K => ctx.global.camera.look_up(should_do),
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
            // Event::MainEventsCleared => {
            //     ctx.window.request_redraw();
            // }
            Event::RedrawRequested(_) => {
                ctx.global.camera.update();
                cube.update(&ctx.queue);

                ctx.global.on_resize(&ctx.queue, ctx.size);

                let frame = match ctx.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        log::info!("recreating a new surface...");
                        ctx.recreate_surface();
                        ctx.surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                if let Some(average_frametime) = loop_clock.tick() {
                    log::info!("average_frametime: {average_frametime}");
                }

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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &ctx.depth_texture_view,
                            depth_ops: Some(Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });
                    rpass.execute_bundles(
                        [&cube.render_bundle, &base_line.render_bundle].into_iter(),
                    );
                }

                ctx.queue.submit(Some(encoder.finish()));

                frame.present();
            }
            Event::RedrawEventsCleared => {
                if let Some(wait_duration) = loop_clock.get_wait_duration() {
                    *control_flow = ControlFlow::WaitUntil(wait_duration);
                } else {
                    ctx.window.request_redraw();
                }
            }
            _ => {}
        }
    });
}
