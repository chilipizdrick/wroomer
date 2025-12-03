use std::{
    f32::{self, consts::PI},
    sync::Arc,
};

use bytemuck::bytes_of;
use glam::{Mat3, Vec2, Vec3};
use image::RgbaImage;
use pollster::block_on;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    application::{
        image_render_pipeline::{ImageRenderPipelineData, ImageUniforms},
        spotlight_render_pipeline::{SpotlightRenderPipelineData, SpotlightUniforms},
    },
    config::AppConfig,
};

#[derive(Debug)]
pub struct State<'a> {
    app_config: AppConfig,
    window: Arc<Window>,

    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,

    device: wgpu::Device,
    queue: wgpu::Queue,

    image_data: ImageRenderPipelineData,
    spotlight_data: SpotlightRenderPipelineData,

    window_size: Vec2,
    image_size: Vec2,
    image_offset: Vec2,
    old_image_offset: Vec2,
    cursor_position: Vec2,
    initial_draging_position: Option<Vec2>,
    image_scale: f32,
    image_rotation_angle: f32,

    spotlight_on: bool,
    spotlight_radius: f32,
    spotlight_darkness: f32,
    scroll_behaviour: ScrollBehaviour,

    #[cfg(feature = "wayland")]
    initial_window_size: Vec2,
    #[cfg(feature = "wayland")]
    initial_image_offset_recalculated: bool,
}

impl State<'_> {
    pub fn new(app_config: AppConfig, window: Window, image: &RgbaImage) -> Self {
        let rendering_backends =
            wgpu::Backends::VULKAN | wgpu::Backends::DX12 | wgpu::Backends::METAL;

        let physical_window_size = window.inner_size();
        let window = Arc::new(window);

        let wgpu_instance = wgpu_instance_with_backends(rendering_backends);

        let surface = wgpu_instance
            .create_surface(Arc::clone(&window))
            .expect("Could not create surface for a window.");

        let adapter = find_adapter_matching_backends_supporting_surface(
            wgpu_instance,
            rendering_backends,
            &surface,
        )
        .expect("Could not find any graphics adapter supporting Vulkan, DX12 or Metal backend.");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_config = surface_configuration(&surface_capabilities, physical_window_size);

        let (device, queue) = request_device(&adapter);

        let window_size = Vec2::new(
            physical_window_size.width as f32,
            physical_window_size.height as f32,
        );

        let image_size = Vec2::new(image.width() as f32, image.height() as f32);

        let (image_offset, image_scale) =
            centered_fitting_image_offset_scale(image_size, window_size);

        let image_rotation_angle = 0.0;

        let image_transform = build_image_transform(
            image_offset,
            image_scale,
            image_rotation_angle,
            image_size,
            window_size,
        );

        let image_uniforms = ImageUniforms::new(image_transform);

        let image_data =
            ImageRenderPipelineData::new(&device, &queue, &surface_config, image, image_uniforms);

        let spotlight_center = Vec2::ZERO;
        let spotlight_radius = 0.1;
        let spotlight_darkness = 0.9;
        let aspect_ratio = window_size.x / window_size.y;

        let spotlight_uniforms = SpotlightUniforms::new(
            spotlight_center,
            spotlight_radius,
            spotlight_darkness,
            aspect_ratio,
        );

        let spotlight_data =
            SpotlightRenderPipelineData::new(&device, &surface_config, spotlight_uniforms);

        Self {
            app_config,
            window,

            surface,
            surface_config,

            device,
            queue,

            image_data,
            spotlight_data,

            image_size,
            window_size,
            image_offset,
            old_image_offset: image_offset,
            cursor_position: Vec2::ZERO,
            initial_draging_position: None,
            image_scale,
            image_rotation_angle,

            spotlight_on: false,
            spotlight_radius,
            spotlight_darkness,
            scroll_behaviour: ScrollBehaviour::default(),

            #[cfg(feature = "wayland")]
            initial_window_size: window_size,
            #[cfg(feature = "wayland")]
            initial_image_offset_recalculated: false,
        }
    }

    pub fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        let should_request_redraw = should_request_redraw(&event);

        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize(size),
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event_loop, event)
            }
            WindowEvent::ModifiersChanged(modifiers) => self.handle_modifiers_changed(modifiers),
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(event_loop, state, button)
            }
            WindowEvent::MouseWheel { delta, .. } => self.handle_mouse_wheel(delta),
            WindowEvent::CursorMoved { position, .. } => self.handle_cursor_moved(position),
            WindowEvent::RedrawRequested => self.handle_redraw_requested(),
            _ => {}
        }

        if should_request_redraw {
            self.window.request_redraw();
        }
    }

    fn handle_keyboard_input(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
        use ElementState as State;
        use KeyCode as Code;

        if let PhysicalKey::Code(key_code) = event.physical_key {
            match (key_code, event.state) {
                (Code::Escape, State::Pressed) => event_loop.exit(),
                (Code::KeyR, State::Pressed) => self.reset_image_position(),
                (Code::KeyQ, State::Pressed) => self.increment_image_rotation_angle(PI / 2.0),
                (Code::KeyE, State::Pressed) => self.increment_image_rotation_angle(-PI / 2.0),
                _ => {}
            }
        }
    }

    fn handle_modifiers_changed(&mut self, modifiers: Modifiers) {
        let state = modifiers.state();

        self.spotlight_on = state.control_key();

        if state.alt_key() {
            self.scroll_behaviour = ScrollBehaviour::Rotate;
            return;
        }

        if state.control_key() && state.shift_key() {
            self.scroll_behaviour = ScrollBehaviour::ChangeSpotlightRadius;
        } else {
            self.scroll_behaviour = ScrollBehaviour::Zoom;
        }
    }

    fn handle_mouse_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        match (button, state) {
            (MouseButton::Left, ElementState::Pressed) => {
                self.initial_draging_position = Some(self.cursor_position);
            }
            (MouseButton::Left, ElementState::Released) => {
                self.old_image_offset = self.image_offset;
                self.initial_draging_position = None;
            }
            (MouseButton::Right, ElementState::Pressed) => {
                event_loop.exit();
            }
            (MouseButton::Back, ElementState::Pressed) => {
                self.reset_image_position();
            }
            _ => {}
        }
    }

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match self.scroll_behaviour {
            ScrollBehaviour::Zoom => self.handle_image_scale_changed(delta),
            ScrollBehaviour::Rotate => self.handle_image_rotation_angle_changed(delta),
            ScrollBehaviour::ChangeSpotlightRadius => self.handle_spotlight_radius_chaged(delta),
        }
    }

    fn handle_image_rotation_angle_changed(&mut self, delta: MouseScrollDelta) {
        let angle_delta = match delta {
            MouseScrollDelta::LineDelta(_, y) => y * PI / 24.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => {
                (y as f32) / self.window_size.y * 2.0 * PI
            }
        };

        self.increment_image_rotation_angle(angle_delta);
    }

    fn increment_image_rotation_angle(&mut self, angle: f32) {
        self.image_rotation_angle += angle % (2.0 * PI);
    }

    fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_position = Vec2::new(position.x as f32, position.y as f32);

        if let Some(initial_draging_position) = self.initial_draging_position {
            self.image_offset =
                self.old_image_offset + self.cursor_position - initial_draging_position;
        }
    }

    fn handle_image_scale_changed(&mut self, delta: MouseScrollDelta) {
        let scale_multiplier = image_scale_multiplier(delta, self.window_size.y);

        let old_scale = self.image_scale;
        let new_scale = (old_scale * scale_multiplier).clamp(0.1, 10.0);

        let relative_position = self.cursor_position - self.image_offset;
        let image_coord = relative_position / old_scale;

        self.image_offset = self.cursor_position - image_coord * new_scale;
        self.old_image_offset = self.image_offset;
        self.image_scale = new_scale;
    }

    fn handle_spotlight_radius_chaged(&mut self, delta: MouseScrollDelta) {
        let radius_multiplier = spotlight_radius_multiplier(delta, self.window_size.y);
        self.spotlight_radius = (self.spotlight_radius * radius_multiplier).clamp(0.01, 1.0);
    }

    fn handle_redraw_requested(&mut self) {
        self.update_uniforms();
        self.write_uniforms();

        match self.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                let window_size = self.window.inner_size();
                self.resize(window_size);
            }
            Err(err) => log::error!("Render error: {err}"),
        }
    }

    fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let texture_view_desc = wgpu::TextureViewDescriptor::default();
        let view = output.texture.create_view(&texture_view_desc);

        let mut encoder = self.create_command_encoder();

        {
            let mut render_pass = begin_render_pass(&mut encoder, &view);

            self.draw_image(&mut render_pass);

            if self.spotlight_on {
                self.draw_spotlight(&mut render_pass);
            }
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        output.present();

        Ok(())
    }

    fn resize(&mut self, window_size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = window_size;

        assert!(
            !(width == 0 || height == 0),
            "Called resize with width / height equal to zero."
        );

        self.window_size = Vec2::new(width as f32, height as f32);
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        if self.app_config.center_image_on_resize {
            self.reset_image_position();
        }

        #[cfg(feature = "wayland")]
        self.check_and_recalculate_initial_image_position();
    }

    // A workaround on wayland to center image after actual window size has been confirmed
    #[cfg(feature = "wayland")]
    fn check_and_recalculate_initial_image_position(&mut self) {
        if !self.initial_image_offset_recalculated && self.initial_window_size != self.window_size {
            self.reset_image_position();
            self.initial_image_offset_recalculated = true;
        }
    }

    fn create_command_encoder(&self) -> wgpu::CommandEncoder {
        let command_encoder_desc = wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        };

        self.device.create_command_encoder(&command_encoder_desc)
    }

    fn draw_image(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.image_data.render_pipeline);
        render_pass.set_bind_group(0, &self.image_data.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn draw_spotlight(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.spotlight_data.render_pipeline);
        render_pass.set_bind_group(0, &self.spotlight_data.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    fn update_uniforms(&mut self) {
        self.update_image_uniforms();

        if self.spotlight_on {
            self.update_spotlight_uniforms();
        }
    }

    fn write_uniforms(&self) {
        self.write_image_uniforms_buffer();

        if self.spotlight_on {
            self.write_spotlight_uniforms_buffer();
        }
    }

    fn write_image_uniforms_buffer(&self) {
        let buffer = &self.image_data.uniforms_buffer;
        let uniforms = &self.image_data.uniforms;
        self.queue.write_buffer(buffer, 0, bytes_of(uniforms));
    }

    fn write_spotlight_uniforms_buffer(&self) {
        let buffer = &self.spotlight_data.uniforms_buffer;
        let uniforms = &self.spotlight_data.uniforms;
        self.queue.write_buffer(buffer, 0, bytes_of(uniforms));
    }

    fn reset_image_position(&mut self) {
        let (offset, scale) =
            centered_fitting_image_offset_scale(self.image_size, self.window_size);
        self.image_offset = offset;
        self.old_image_offset = offset;
        self.image_scale = scale;
        self.image_rotation_angle = 0.0;
    }

    fn update_image_uniforms(&mut self) {
        let transform = build_image_transform(
            self.image_offset,
            self.image_scale,
            self.image_rotation_angle,
            self.image_size,
            self.window_size,
        );

        self.image_data.uniforms.set_transform(transform);
    }

    fn update_spotlight_uniforms(&mut self) {
        let uniforms = &mut self.spotlight_data.uniforms;

        uniforms.center_position = self.cursor_position / self.window_size;
        uniforms.radius = self.spotlight_radius;
        uniforms.darkness = self.spotlight_darkness;
        uniforms.aspect_ratio = self.window_size.x / self.window_size.y;
    }
}

#[derive(Debug, Default)]
enum ScrollBehaviour {
    #[default]
    Zoom,
    Rotate,
    ChangeSpotlightRadius,
}

fn wgpu_instance_with_backends(backends: wgpu::Backends) -> wgpu::Instance {
    let wgpu_instance_desc = wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    };
    wgpu::Instance::new(&wgpu_instance_desc)
}

fn find_adapter_matching_backends_supporting_surface(
    wgpu_instance: wgpu::Instance,
    backends: wgpu::Backends,
    surface: &wgpu::Surface,
) -> Option<wgpu::Adapter> {
    wgpu_instance
        .enumerate_adapters(backends)
        .into_iter()
        .find(|a| a.is_surface_supported(surface))
}

fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let device_desc = wgpu::DeviceDescriptor::default();
    block_on(adapter.request_device(&device_desc))
        .expect("Could not open connection to a graphics device.")
}

fn surface_configuration(
    capabilities: &wgpu::SurfaceCapabilities,
    window_size: PhysicalSize<u32>,
) -> wgpu::SurfaceConfiguration {
    let texture_format = capabilities
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(capabilities.formats[0]);

    let alpha_mode = select_surface_alpha_mode(capabilities);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: texture_format,
        height: window_size.height,
        width: window_size.width,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 2,
        alpha_mode,
        view_formats: vec![],
    }
}

fn begin_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    let render_pass_desc = wgpu::RenderPassDescriptor {
        label: Some("Image + Spotlight Draw Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    };

    encoder.begin_render_pass(&render_pass_desc)
}

fn should_request_redraw(event: &WindowEvent) -> bool {
    matches!(
        event,
        WindowEvent::Resized(_)
            | WindowEvent::ScaleFactorChanged { .. }
            | WindowEvent::KeyboardInput { .. }
            | WindowEvent::ModifiersChanged(_)
            | WindowEvent::MouseInput { .. }
            | WindowEvent::MouseWheel { .. }
            | WindowEvent::CursorMoved { .. }
    )
}

// Select surface alpha mode, preferring ones allowing for transparency
fn select_surface_alpha_mode(capabilities: &wgpu::SurfaceCapabilities) -> wgpu::CompositeAlphaMode {
    use wgpu::CompositeAlphaMode as CAM;

    let alpha_modes = &capabilities.alpha_modes;
    match () {
        _ if alpha_modes.contains(&CAM::PreMultiplied) => CAM::PreMultiplied,
        _ if alpha_modes.contains(&CAM::PostMultiplied) => CAM::PostMultiplied,
        _ => CAM::Auto,
    }
}

// Compose an affine transform that would translate the image and scale it up or down
// The resulting transform operates on UV coordinates and transforms them to NDC space
//
// uv_to_ndc * ((scale + translation) * rotation)
fn build_image_transform(
    offset: Vec2,
    scale: f32,
    rotation_angle: f32,
    image_size: Vec2,
    window_size: Vec2,
) -> Mat3 {
    let rcos = rotation_angle.cos();
    let rsin = rotation_angle.sin();

    let image_center = scale * image_size / 2.0;

    let translation_x = offset.x + (1.0 - rcos) * image_center.x + rsin * image_center.y;
    let translation_y = offset.y - rsin * image_center.x + (1.0 - rcos) * image_center.y;

    let col_x = Vec3::new(
        2.0 * scale * rcos * image_size.x / window_size.x,
        -2.0 * scale * rsin * image_size.x / window_size.y,
        0.0,
    );
    let col_y = Vec3::new(
        -2.0 * scale * rsin * image_size.y / window_size.x,
        -2.0 * scale * rcos * image_size.y / window_size.y,
        0.0,
    );
    let col_z = Vec3::new(
        2.0 * translation_x / window_size.x - 1.0,
        1.0 - 2.0 * translation_y / window_size.y,
        1.0,
    );

    Mat3::from_cols(col_x, col_y, col_z)
}

// Calculate offset and scale for image, such that it would get fitted (get smaller) if it is bigger
// than the window, but will get padded (won't change size) if it is smaller than the window
fn centered_fitting_image_offset_scale(image_size: Vec2, window_size: Vec2) -> (Vec2, f32) {
    fn calculate_for_bigger_width(image_size: Vec2, window_size: Vec2) -> (Vec2, f32) {
        let scale = window_size.x / image_size.x;
        (
            Vec2::new(0.0, (window_size.y - scale * image_size.y) / 2.0),
            scale,
        )
    }

    fn calculate_for_bigger_height(image_size: Vec2, window_size: Vec2) -> (Vec2, f32) {
        let scale = window_size.y / image_size.y;
        (
            Vec2::new((window_size.x - scale * image_size.x) / 2.0, 0.0),
            scale,
        )
    }

    let size_difference = window_size - image_size;

    match (size_difference.x > 0.0, size_difference.y > 0.0) {
        (false, false) => {
            let image_scales = window_size / image_size;
            if image_scales.x < image_scales.y {
                calculate_for_bigger_width(image_size, window_size)
            } else {
                calculate_for_bigger_height(image_size, window_size)
            }
        }
        (false, true) => calculate_for_bigger_width(image_size, window_size),
        (true, false) => calculate_for_bigger_height(image_size, window_size),
        (true, true) => ((window_size - image_size) / 2.0, 1.0),
    }
}

fn spotlight_radius_multiplier(delta: MouseScrollDelta, window_height: f32) -> f32 {
    multiplier_from_mouse_delta(delta, window_height)
}

fn image_scale_multiplier(delta: MouseScrollDelta, window_height: f32) -> f32 {
    multiplier_from_mouse_delta(delta, window_height)
}

fn multiplier_from_mouse_delta(delta: MouseScrollDelta, window_height: f32) -> f32 {
    const PIXEL_DELTA_SCROLL_SENSITIVITY: f32 = 5.0;

    match delta {
        MouseScrollDelta::LineDelta(_, y) => match y {
            ..0.0 => 0.9,
            0.0.. => 1.1,
            _ => 1.0,
        },
        MouseScrollDelta::PixelDelta(PhysicalPosition { y, .. }) => match y {
            ..0.0 => 1.0 - (y as f32 / window_height * PIXEL_DELTA_SCROLL_SENSITIVITY).abs(),
            0.0.. => 1.0 + (y as f32 / window_height * PIXEL_DELTA_SCROLL_SENSITIVITY).abs(),
            _ => 1.0,
        },
    }
}
