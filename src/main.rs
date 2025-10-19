mod texture;
mod uniforms;
mod vector;

use std::sync::Arc;

use image::DynamicImage;
use libwayshot::WayshotConnection;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, InnerSizeWriter, KeyEvent, Modifiers, MouseButton, MouseScrollDelta,
        WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    platform::wayland::EventLoopBuilderExtWayland,
    window::{Fullscreen, Window, WindowId},
};

use crate::{texture::Texture, uniforms::Uniforms, vector::Vec2f32};

fn main() {
    tracing_subscriber::fmt().init();

    if let Err(err) = run_app() {
        log::error!("{err}");
        std::process::exit(1);
    }
}

fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::builder().with_wayland().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let wayshot_connection = WayshotConnection::new()?;
    let screenshot = wayshot_connection.screenshot_all(false)?;
    log::info!("Captured screenshot of all screens");

    let mut app = App::new(screenshot);

    event_loop.run_app(&mut app).map_err(Into::into)
}

#[derive(Debug, Default)]
struct App {
    state: Option<State<'static>>,
    image: DynamicImage,
}

impl App {
    fn new(image: DynamicImage) -> Self {
        Self { state: None, image }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Wroomer")
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .with_transparent(true);

            let window = event_loop.create_window(window_attributes).unwrap();

            self.state = Some(State::new(window, &self.image));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            let should_request_redraw = !matches!(
                event,
                WindowEvent::RedrawRequested | WindowEvent::CloseRequested | WindowEvent::Destroyed
            );

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Destroyed => event_loop.exit(),
                WindowEvent::Resized(size) => state.handle_resize_event(size),
                WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    inner_size_writer,
                } => state.handle_scale_factor_changed(scale_factor, inner_size_writer),
                WindowEvent::KeyboardInput { event, .. } => {
                    state.handle_keyboard_input(event_loop, event)
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    state.handle_modifiers_changed(modifiers)
                }
                WindowEvent::MouseInput {
                    state: elem_state,
                    button,
                    ..
                } => state.handle_mouse_input(event_loop, elem_state, button),
                WindowEvent::MouseWheel { delta, .. } => state.handle_mouse_wheel(delta),
                WindowEvent::CursorMoved { position, .. } => state.handle_cursor_moved(position),
                WindowEvent::RedrawRequested => state.handle_redraw_requested(),
                _ => {}
            }

            if should_request_redraw {
                state.window.request_redraw();
            }
        }
    }
}

#[derive(Debug)]
struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniforms_buffer: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    window: Arc<Window>,
    old_image_offset: Vec2f32,
    initial_draging_position: Option<Vec2f32>,
    scroll_behaviour: ScrollBehaviour,
}

impl State<'_> {
    fn new(window: Window, image: &DynamicImage) -> Self {
        let rendering_backends = wgpu::Backends::VULKAN;

        let window_size = window.inner_size();

        let window = Arc::new(window);

        let wgpu_instance_descriptor = wgpu::InstanceDescriptor {
            backends: rendering_backends,
            ..Default::default()
        };
        let wgpu_instance = wgpu::Instance::new(&wgpu_instance_descriptor);

        let surface = wgpu_instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = wgpu_instance
            .enumerate_adapters(rendering_backends)
            .into_iter()
            .find(|a| a.is_surface_supported(&surface))
            .unwrap();

        let device_desc = wgpu::DeviceDescriptor::default();
        let (device, queue) = pollster::block_on(adapter.request_device(&device_desc)).unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let texture_format = surface_capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            height: window_size.height,
            width: window_size.width,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
        };

        let uniforms = Uniforms::new(
            (window_size.width as f32, window_size.height as f32).into(),
            (image.width() as f32, image.height() as f32).into(),
        );

        let uniforms_buffer = device.create_buffer_init(&uniforms.buffer_init_descriptor());

        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&Uniforms::bind_group_layout_descriptor());

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms Bind Group"),
            layout: &uniforms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let image_texture =
            Texture::from_image(&device, &queue, image, Some("Image texture")).unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image_texture.sampler),
                },
            ],
            label: Some("Diffuse Bind Group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniforms_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            surface_config,
            render_pipeline,
            uniforms,
            uniforms_buffer,
            uniforms_bind_group,
            texture_bind_group,
            window,
            old_image_offset: (0.0, 0.0).into(),
            initial_draging_position: None,
            scroll_behaviour: ScrollBehaviour::Zoom,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let window_size = self.window.inner_size();
        self.uniforms.canvas_size = (window_size.width as f32, window_size.height as f32).into();

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Image Draw Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.uniforms.spotlight_color[0] as f64,
                            g: self.uniforms.spotlight_color[1] as f64,
                            b: self.uniforms.spotlight_color[2] as f64,
                            a: self.uniforms.spotlight_color[3] as f64,
                        }),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                ..Default::default()
            };

            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }

    fn handle_resize_event(&mut self, size: PhysicalSize<u32>) {
        self.resize(size.width, size.height);
    }

    // TODO: Test this implementation
    fn handle_scale_factor_changed(
        &mut self,
        scale_factor: f64,
        mut inner_size_writer: InnerSizeWriter,
    ) {
        let new_window_size = self.uniforms.canvas_size / scale_factor as f32;
        if let Err(err) = inner_size_writer.request_inner_size(PhysicalSize::new(
            new_window_size.x as u32,
            new_window_size.y as u32,
        )) {
            log::error!("Failed update window size after scale factor change: {err}");
            return;
        }

        self.uniforms.canvas_size = new_window_size;
        self.uniforms.cursor_position /= scale_factor as f32;

        self.write_uniforms();
    }

    fn handle_keyboard_input(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
        use winit::keyboard::{Key, NamedKey};

        match (event.logical_key, event.state) {
            (Key::Named(NamedKey::Escape), ElementState::Pressed) => event_loop.exit(),
            (Key::Character(char), ElementState::Pressed) => {
                if char.as_str() == "q" {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn handle_modifiers_changed(&mut self, modifiers: Modifiers) {
        let state = modifiers.state();

        if state.control_key() {
            self.uniforms.spotlight_color = [0.0, 0.0, 0.0, 0.9];
        } else {
            self.uniforms.spotlight_color = [0.0, 0.0, 0.0, 0.0];
        }

        if state.control_key() && state.shift_key() {
            self.scroll_behaviour = ScrollBehaviour::ChangeSpotlightRadius;
        } else {
            self.scroll_behaviour = ScrollBehaviour::Zoom;
        }

        self.write_uniforms();
    }

    fn handle_mouse_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                self.initial_draging_position = Some(self.uniforms.cursor_position);
            }
            (MouseButton::Left, ElementState::Released) => {
                self.old_image_offset = self.uniforms.image_offset;
                self.initial_draging_position = None;
            }
            (MouseButton::Right, ElementState::Pressed) => {
                event_loop.exit();
            }
            (MouseButton::Back, ElementState::Pressed) => {
                self.reset_zoom_factor_and_center_image();
            }
            _ => {}
        }
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match self.scroll_behaviour {
            ScrollBehaviour::Zoom => self.handle_zoom_with_mouse_delta(delta),
            ScrollBehaviour::ChangeSpotlightRadius => {
                self.handle_spotlight_radius_chage_with_mouse_delta(delta)
            }
        }
    }

    fn handle_zoom_with_mouse_delta(&mut self, delta: MouseScrollDelta) {
        // let pixel_delta = self.scroll_delta_to_pixel_delta(delta);
        // let delta = if pixel_delta > 0.0 { 1.1 } else { 0.9 };
        let zoom_multiplier = match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                if y < 0.0 {
                    0.9
                } else {
                    1.1
                }
            }
            MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                if y < 0.0 {
                    0.98
                } else {
                    1.02
                }
            }
        };

        let old_zoom_factor = self.uniforms.zoom_factor;
        let new_zoom_factor = (old_zoom_factor * zoom_multiplier).clamp(0.1, 10.0);

        let relative_position = self.uniforms.cursor_position - self.uniforms.image_offset;
        let image_coord = relative_position / old_zoom_factor;
        self.uniforms.image_offset = self.uniforms.cursor_position - image_coord * new_zoom_factor;
        self.old_image_offset = self.uniforms.image_offset;
        self.uniforms.zoom_factor = new_zoom_factor;

        self.write_uniforms();
    }

    fn handle_spotlight_radius_chage_with_mouse_delta(&mut self, delta: MouseScrollDelta) {
        let delta = self.scroll_delta_to_pixel_delta(delta);
        self.uniforms.spotlight_radius_multiplier =
            (self.uniforms.spotlight_radius_multiplier + delta).clamp(0.2, 5.0);
        self.write_uniforms();
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.uniforms.cursor_position = position.into();

        if let Some(initial_draging_position) = self.initial_draging_position {
            self.uniforms.image_offset =
                self.old_image_offset + self.uniforms.cursor_position - initial_draging_position;
        }

        self.write_uniforms();
    }

    fn handle_redraw_requested(&mut self) {
        match self.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
            }
            Err(err) => log::error!("Render error: {err}"),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.uniforms.canvas_size = (width as f32, height as f32).into();
            self.write_uniforms();
        }
    }

    fn reset_zoom_factor_and_center_image(&mut self) {
        self.old_image_offset = (0.0, 0.0).into();
        self.uniforms.image_offset = (0.0, 0.0).into();
        self.uniforms.zoom_factor = 1.0;
        self.uniforms.spotlight_radius_multiplier = 1.0;
        self.write_uniforms();
    }

    fn write_uniforms(&mut self) {
        self.queue
            .write_buffer(&self.uniforms_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }

    fn scroll_delta_to_pixel_delta(&self, delta: MouseScrollDelta) -> f32 {
        const PIXEL_DELTA_SCROLL_COEFFICIENT: f32 = 10.0;

        match delta {
            MouseScrollDelta::LineDelta(_, y) => match y {
                ..0.0 => -0.1,
                0.0 => 0.0,
                0.0.. => 0.1,
                _ => 0.0,
            },
            MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                (y as f32) / self.uniforms.canvas_size.y * PIXEL_DELTA_SCROLL_COEFFICIENT
            }
        }
    }
}

#[derive(Debug, Default)]
enum ScrollBehaviour {
    #[default]
    Zoom,
    ChangeSpotlightRadius,
}
