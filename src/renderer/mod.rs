pub struct Renderer {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Surface,
    config: wgpu::SurfaceConfiguration,
}

pub struct Surface {
    pub surface: wgpu::Surface,
    pub config: SurfaceConfig,
}

pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub present_mode: wgpu::PresentMode,
    pub capabilities: wgpu::SurfaceCapabilities,
    pub format: wgpu:: TextureFormat,
}

impl Renderer {
    pub(crate) async fn create(
        instance: &wgpu::Instance,
        surface: &Surface,
    ) -> (wgpu::Device, wgpu::Adapter, wgpu::Queue) {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface.surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        (device, adapter, queue)
    }

    pub fn new(
        instance: wgpu::Instance,
        size: &winit::dpi::PhysicalSize<u32>,
        surface: Surface,
    ) -> Self {
        let (device, adapter, queue) =
            pollster::block_on(self::Renderer::create(&instance, &surface));

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.config.format,
                width: size.width,
                height: size.height,
                present_mode: surface.config.present_mode,
                alpha_mode: surface.config.capabilities.alpha_modes[0],
                view_formats: vec![],
            };

            surface.surface.configure(&device, &config);

        Renderer {
            instance: instance,
            adapter: adapter,
            device: device,
            queue: queue,
            surface: surface,
            config: config,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Reconfigure the surface with the new size
        self.surface.config.width = size.width;
        self.surface.config.height = size.height;
        self.surface.surface.configure(&self.device, &self.config);
    }

    pub fn present(&self) {
        let frame = self.surface
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
