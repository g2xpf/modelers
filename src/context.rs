use winit::dpi::PhysicalSize;

use winit::event_loop::EventLoop;
use winit::window::Window;

use wgpu::util::{self, DeviceExt};
use wgpu::{
    Adapter, Backends, BindGroup, BufferUsages, Device, DeviceDescriptor, Features, Instance,
    Limits, PresentMode, Queue, Surface, SurfaceConfiguration, TextureUsages,
};

use crate::Camera;

pub struct Context {
    pub window: Window,
    pub adapter: Adapter,
    pub surface_config: SurfaceConfiguration,
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
    pub size: PhysicalSize<u32>,
    pub surface: Surface,

    pub camera: Camera,
    pub global_ubo: wgpu::Buffer,
}

impl Context {
    pub async fn create_context() -> (Context, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).expect("Failed to create window");

        #[cfg(target_os = "macos")]
        let backends = Backends::METAL;
        #[cfg(not(target_os = "macos"))]
        let backends = Backends::VULKAN;

        let instance = Instance::new(backends);
        let size = window.inner_size();
        let surface = unsafe { instance.create_surface(&window) };

        let adapter =
            util::initialize_adapter_from_env_or_default(&instance, backends, Some(&surface))
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

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Mailbox,
        };

        surface.configure(&device, &surface_config);

        let camera = Camera::default();
        let vp_matrix = camera.create_vp_matrix(size.width as f32 / size.height as f32);
        let vp_matrix: &[f32; 16] = vp_matrix.as_ref();
        let global_ubo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vp_matrix),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        (
            Context {
                global_ubo,
                camera,
                surface_config,
                window,
                device,
                queue,
                instance,
                adapter,
                size,
                surface,
            },
            event_loop,
        )
    }

    pub fn recreate_surface(&self) {
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}
