use winit::dpi::PhysicalSize;

use winit::event_loop::EventLoop;
use winit::window::Window;

use wgpu::util::{self, DeviceExt};
use wgpu::{
    Adapter, Backends, BindGroup, BindGroupLayoutEntry, BufferUsages, Device, DeviceDescriptor,
    Features, Instance, Limits, PresentMode, Queue, Sampler, ShaderStages, Surface,
    SurfaceConfiguration, TextureUsages, TextureView, TextureViewDescriptor,
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

    // global uniforms
    pub camera: Camera,
    pub global_ubo: wgpu::Buffer,

    // for depth test
    pub sampler: Sampler,
    pub depth_texture_view: TextureView,
    pub global_bind_group: BindGroup,
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

        let depth_texture_view = Self::create_depth_texture_view(&device, &surface_config);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..wgpu::SamplerDescriptor::default()
        });

        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: true,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_ubo.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&depth_texture_view),
                },
            ],
            layout: &global_bind_group_layout,
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
                sampler,
                depth_texture_view,
                global_bind_group,
            },
            event_loop,
        )
    }

    pub fn create_depth_texture_view(
        device: &Device,
        surface_config: &SurfaceConfiguration,
    ) -> TextureView {
        let depth_texture_size = wgpu::Extent3d {
            width: surface_config.width,
            height: surface_config.height,
            depth_or_array_layers: 1,
        };
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: depth_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        depth_texture.create_view(&TextureViewDescriptor::default())
    }

    pub fn recreate_surface(&mut self) {
        self.surface.configure(&self.device, &self.surface_config);
        self.depth_texture_view =
            Self::create_depth_texture_view(&self.device, &self.surface_config);
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }
}
