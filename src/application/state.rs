use std::sync::Arc;

use image::DynamicImage;
use wgpu::{BindGroupLayout, util::DeviceExt};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, InnerSizeWriter, KeyEvent, Modifiers, MouseButton, MouseScrollDelta},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::Window,
};

use crate::{
    application::{texture::DiffuseImageTexture, uniforms::Uniforms, vec2d::Vec2f32},
    config::AppConfig,
};

#[derive(Debug)]
pub struct State<'a> {
    config: AppConfig,
    device: wgpu::Device,
    dvd_logo_speed: Vec2f32,
    dvd_texture_bind_group: Option<wgpu::BindGroup>,
    image_texture_bind_group: wgpu::BindGroup,
    initial_draging_position: Option<Vec2f32>,
    old_image_offset: Vec2f32,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    scroll_behaviour: ScrollBehaviour,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    time: u32,
    uniforms: Uniforms,
    uniforms_bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
    window: Arc<Window>,
}

impl State<'_> {
    pub fn new(window: Window, image: &DynamicImage, config: AppConfig) -> Self {
        let rendering_backends = wgpu::Backends::PRIMARY;

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

        let alpha_mode = select_alpha_mode_prefer_transparency(&surface_capabilities);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            height: window_size.height,
            width: window_size.width,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };

        let mut uniforms = Uniforms::with_canvas_and_image_size(
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
            DiffuseImageTexture::from_image(&device, &queue, image, Some("Image Texture"));

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

        let image_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let dvd_texture_bind_group = if config.dvd_logo_enabled {
            let dvd_logo_image =
                image::load_from_memory(include_bytes!("../../assets/dvd_logo.png")).unwrap();

            let dvd_logo_image = dvd_logo_image.resize(
                (uniforms.image_size.x / 5.0) as u32,
                (uniforms.image_size.y / 5.0) as u32,
                image::imageops::FilterType::Nearest,
            );

            uniforms.dvd_logo_size = (
                dvd_logo_image.width() as f32,
                dvd_logo_image.height() as f32,
            )
                .into();

            let dvd_logo_texture = DiffuseImageTexture::from_image(
                &device,
                &queue,
                &dvd_logo_image,
                Some("DVD Logo Texture"),
            );

            let dvd_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&dvd_logo_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&dvd_logo_texture.sampler),
                    },
                ],
                label: Some("Diffuse Bind Group"),
            });

            Some(dvd_texture_bind_group)
        } else {
            None
        };

        let shader_module_desc = match config.dvd_logo_enabled {
            false => wgpu::include_wgsl!("../../assets/shaders/shader.wgsl"),
            true => wgpu::include_wgsl!("../../assets/shaders/shader_dvd.wgsl"),
        };

        let shader = device.create_shader_module(shader_module_desc);

        let bind_group_layouts: &[&BindGroupLayout] = match config.dvd_logo_enabled {
            false => &[&uniforms_bind_group_layout, &texture_bind_group_layout],
            true => &[
                &uniforms_bind_group_layout,
                &texture_bind_group_layout,
                &texture_bind_group_layout,
            ],
        };

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
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
            config,
            device,
            dvd_logo_speed: Vec2f32::new(1.0, 1.0),
            dvd_texture_bind_group,
            image_texture_bind_group,
            initial_draging_position: None,
            old_image_offset: uniforms.image_offset,
            queue,
            render_pipeline,
            scroll_behaviour: ScrollBehaviour::Zoom,
            surface,
            surface_config,
            time: 0,
            uniforms,
            uniforms_bind_group,
            uniforms_buffer,
            window,
        }
    }

    pub fn request_window_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                        load: wgpu::LoadOp::Clear(color_from_rgba(&self.uniforms.spotlight_color)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            };

            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
            render_pass.set_bind_group(1, &self.image_texture_bind_group, &[]);

            if self.config.dvd_logo_enabled {
                render_pass.set_bind_group(2, &self.dvd_texture_bind_group, &[]);
            }

            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }

    pub fn handle_resize_event(&mut self, size: PhysicalSize<u32>) {
        self.resize(size.width, size.height);
    }

    // TODO: Test this implementation
    pub fn handle_scale_factor_changed(
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

        self.reset_zoom_factor_and_image_offset();
        self.write_uniforms();
    }

    pub fn handle_keyboard_input(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
        match (event.logical_key, event.state) {
            (Key::Named(NamedKey::Escape), ElementState::Pressed) => event_loop.exit(),
            (Key::Character(char), ElementState::Pressed) => match char.as_str() {
                "q" => event_loop.exit(),
                "d" if self.config.dvd_logo_enabled => {
                    self.toggle_dvd_logo_visibility();
                }
                ">" if self.config.dvd_logo_enabled => {
                    self.dvd_logo_speed *= 1.2;
                }
                "<" if self.config.dvd_logo_enabled => {
                    self.dvd_logo_speed *= 0.8;
                }
                "=" if self.config.dvd_logo_enabled => {
                    self.dvd_logo_speed = Vec2f32::new(1.0, 1.0);
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn handle_modifiers_changed(&mut self, modifiers: Modifiers) {
        const TRANSPARENT: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        const BLACK_TINT: [f32; 4] = [0.0, 0.0, 0.0, 0.9];

        let state = modifiers.state();

        if state.control_key() {
            self.uniforms.spotlight_color = BLACK_TINT;
        } else {
            self.uniforms.spotlight_color = TRANSPARENT;
        }

        if state.control_key() && state.shift_key() {
            self.scroll_behaviour = ScrollBehaviour::ChangeSpotlightRadius;
        } else {
            self.scroll_behaviour = ScrollBehaviour::Zoom;
        }

        self.write_uniforms();
    }

    pub fn handle_mouse_input(
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
                self.reset_zoom_factor_and_image_offset();
            }
            _ => {}
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match self.scroll_behaviour {
            ScrollBehaviour::Zoom => self.handle_zoom_changed(delta),
            ScrollBehaviour::ChangeSpotlightRadius => self.handle_spotlight_radius_chaged(delta),
        }
    }

    pub fn handle_redraw_requested(&mut self) {
        if self.uniforms.dvd_logo_visible != 0 {
            self.update_dvd_logo_state();
            self.time = self.time.wrapping_add(1);
        }

        match self.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
            }
            Err(err) => log::error!("Render error: {err}"),
        }
    }

    pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.uniforms.cursor_position = position.into();

        if let Some(initial_draging_position) = self.initial_draging_position {
            self.uniforms.image_offset =
                self.old_image_offset + self.uniforms.cursor_position - initial_draging_position;
        }

        self.write_uniforms();
    }

    fn handle_zoom_changed(&mut self, delta: MouseScrollDelta) {
        let zoom_multiplier = self.zoom_multiplier(delta);

        let old_zoom_factor = self.uniforms.zoom_factor;
        let new_zoom_factor = (old_zoom_factor * zoom_multiplier).clamp(0.1, 10.0);

        let relative_position = self.uniforms.cursor_position - self.uniforms.image_offset;
        let image_coord = relative_position / old_zoom_factor;

        self.uniforms.image_offset = self.uniforms.cursor_position - image_coord * new_zoom_factor;
        self.old_image_offset = self.uniforms.image_offset;
        self.uniforms.zoom_factor = new_zoom_factor;

        self.write_uniforms();
    }

    fn handle_spotlight_radius_chaged(&mut self, delta: MouseScrollDelta) {
        let delta = self.spotlight_radius_delta(delta);
        self.uniforms.spotlight_radius_multiplier =
            (self.uniforms.spotlight_radius_multiplier + delta).clamp(0.2, 5.0);
        self.write_uniforms();
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

    fn reset_zoom_factor_and_image_offset(&mut self) {
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

    fn spotlight_radius_delta(&self, delta: MouseScrollDelta) -> f32 {
        const PIXEL_DELTA_SCROLL_COEFFICIENT: f32 = 10.0;

        match delta {
            MouseScrollDelta::LineDelta(_, y) => y.clamp(-0.1, 0.1),
            MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                (y as f32) / self.uniforms.canvas_size.y * PIXEL_DELTA_SCROLL_COEFFICIENT
            }
        }
    }

    fn zoom_multiplier(&self, delta: MouseScrollDelta) -> f32 {
        match delta {
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
        }
    }

    fn update_dvd_logo_state(&mut self) {
        self.update_dvd_logo_position();
        self.update_dvd_logo_color();
        self.write_uniforms();
    }

    fn update_dvd_logo_position(&mut self) {
        let image_size = self.uniforms.image_size;
        let dvd_logo_size = self.uniforms.dvd_logo_size;

        let dvd_logo_position = &mut self.uniforms.dvd_logo_position;
        *dvd_logo_position += self.dvd_logo_speed;

        if dvd_logo_position.x < 0.0 || dvd_logo_position.x + dvd_logo_size.x > image_size.x {
            let x_pos = &mut dvd_logo_position.x;
            *x_pos = x_pos.clamp(1.0, image_size.x - dvd_logo_size.x - 1.0);
            self.dvd_logo_speed.x = -self.dvd_logo_speed.x;
        }

        if dvd_logo_position.y < 0.0 || dvd_logo_position.y + dvd_logo_size.y > image_size.y {
            let y_pos = &mut dvd_logo_position.y;
            *y_pos = y_pos.clamp(1.0, image_size.y - dvd_logo_size.y - 1.0);
            self.dvd_logo_speed.y = -self.dvd_logo_speed.y;
        }
    }

    fn update_dvd_logo_color(&mut self) {
        let wheel_pos = self.time % 256;
        self.uniforms.dvd_logo_color = color_wheel(wheel_pos);
    }

    fn toggle_dvd_logo_visibility(&mut self) {
        if self.uniforms.dvd_logo_visible == 0 {
            self.uniforms.dvd_logo_visible = 1;
        } else {
            self.uniforms.dvd_logo_visible = 0;
        };

        self.write_uniforms();
    }
}

#[derive(Debug, Default)]
enum ScrollBehaviour {
    #[default]
    Zoom,
    ChangeSpotlightRadius,
}

fn color_from_rgba(rgba: &[f32; 4]) -> wgpu::Color {
    wgpu::Color {
        r: rgba[0] as f64,
        g: rgba[1] as f64,
        b: rgba[2] as f64,
        a: rgba[3] as f64,
    }
}

fn select_alpha_mode_prefer_transparency(
    capabilities: &wgpu::SurfaceCapabilities,
) -> wgpu::CompositeAlphaMode {
    use wgpu::CompositeAlphaMode::*;

    let alpha_modes = &capabilities.alpha_modes;
    match () {
        _ if alpha_modes.contains(&PreMultiplied) => PreMultiplied,
        _ if alpha_modes.contains(&PostMultiplied) => PostMultiplied,
        _ => Auto,
    }
}

fn color_wheel(mut pos: u32) -> [f32; 4] {
    if pos < 85 {
        [
            (255 - pos * 3) as f32 / 255.0,
            0.0,
            (pos * 3) as f32 / 255.0,
            0.5,
        ]
    } else if pos < 170 {
        pos -= 85;
        [
            0.0,
            (pos * 3) as f32 / 255.0,
            (255 - pos * 3) as f32 / 255.0,
            0.5,
        ]
    } else {
        pos -= 170;
        [
            (pos * 3) as f32 / 255.0,
            (255 - pos * 3) as f32 / 255.0,
            0.0,
            0.5,
        ]
    }
}
