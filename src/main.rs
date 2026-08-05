use std::{borrow::Cow, sync::Arc};

use wgpu::CurrentSurfaceTexture;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::Window,
};

fn spawn(f: impl Future<Output = ()> + 'static) {
    pollster::block_on(f);
}

struct WgpuState {
    instance: wgpu::Instance,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
}

enum TriangleAction {
    Initialized(WgpuState),
}

#[expect(clippy::large_enum_variant)]
enum AppState {
    Uninitialized,
    Loading,
    Running(WgpuState),
}

struct App {
    proxy: EventLoopProxy<TriangleAction>,
    window: Option<Arc<Window>>,
    state: AppState,
}

impl App {
    fn new(event_loop: &EventLoop<TriangleAction>) -> Self {
        Self {
            proxy: event_loop.create_proxy(),
            window: None,
            state: AppState::Uninitialized,
        }
    }

    fn resize_surface(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let AppState::Running(wgpu_state) = &mut self.state else {
            return;
        };

        wgpu_state.config.width = new_size.width.max(1);
        wgpu_state.config.height = new_size.height.max(1);

        wgpu_state
            .surface
            .configure(&wgpu_state.device, &wgpu_state.config);

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler<TriangleAction> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Uninitialized) {
            return;
        }
        self.state = AppState::Loading;

        let mut attributes = Window::default_attributes();

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("Failed to create window"),
        );

        self.window = Some(window.clone());

        let display_handle = event_loop.owned_display_handle();
        let proxy = self.proxy.clone();

        spawn(async move {
            let mut size = window.inner_size();
            size.width = size.width.max(1);
            size.height = size.height.max(1);

            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                    Box::new(display_handle),
                ));

            let surface = instance.create_surface(window.clone()).unwrap();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    ..Default::default()
                })
                .await
                .expect("Failed to find an appropriate adapter");

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::defaults(),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off,
                })
                .await
                .expect("Failed to create device");

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                immediate_size: 0,
            });

            let swapchain_capabilities = surface.get_capabilities(&adapter);
            let swapchain_format = swapchain_capabilities.formats[0];

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(swapchain_format.into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            let config = surface
                .get_default_config(&adapter, size.width, size.height)
                .unwrap();

            let _ = proxy.send_event(TriangleAction::Initialized(WgpuState {
                instance,
                window,
                device,
                queue,
                surface,
                config,
                render_pipeline,
            }));
        });
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: TriangleAction) {
        match event {
            TriangleAction::Initialized(wgpu_state) => {
                self.state = AppState::Running(wgpu_state);

                if let Some(window) = &self.window {
                    self.resize_surface(window.inner_size());
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let AppState::Running(wgpu_state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::Resized(new_size) => {
                self.resize_surface(new_size);
            }

            WindowEvent::RedrawRequested => {
                let frame = match wgpu_state.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(frame) => frame,
                    CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => {
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }

                        return;
                    }
                    CurrentSurfaceTexture::Suboptimal(texture) => {
                        drop(texture);
                        wgpu_state
                            .surface
                            .configure(&wgpu_state.device, &wgpu_state.config);

                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }

                        return;
                    }
                    CurrentSurfaceTexture::Outdated => {
                        wgpu_state
                            .surface
                            .configure(&wgpu_state.device, &wgpu_state.config);

                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }

                        return;
                    }
                    CurrentSurfaceTexture::Validation => {
                        unreachable!("No error scope registered, so validation errors will panic")
                    }
                    CurrentSurfaceTexture::Lost => {
                        wgpu_state.surface = wgpu_state
                            .instance
                            .create_surface(wgpu_state.window.clone())
                            .unwrap();
                        wgpu_state
                            .surface
                            .configure(&wgpu_state.device, &wgpu_state.config);

                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }

                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = wgpu_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    rpass.set_pipeline(&wgpu_state.render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                wgpu_state.queue.submit(Some(encoder.finish()));

                if let Some(window) = &self.window {
                    window.pre_present_notify();
                }

                wgpu_state.queue.present(frame);
            }

            WindowEvent::Occluded(is_occluded) => {
                if !is_occluded {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app).unwrap();
}
