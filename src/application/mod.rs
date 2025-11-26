mod state;
mod texture;
mod uniforms;
mod vec2d;

use image::DynamicImage;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Fullscreen, Window, WindowId},
};

use crate::{application::state::State, config::AppConfig};

#[derive(Debug, Default)]
pub struct App {
    state: Option<State<'static>>,
    image: DynamicImage,
    config: AppConfig,
}

impl App {
    pub fn new(image: DynamicImage, config: AppConfig) -> Self {
        Self {
            state: None,
            image,
            config,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Wroomer")
                .with_fullscreen(Some(Fullscreen::Borderless(None)))
                .with_transparent(true)
                .with_blur(true);

            let window = event_loop.create_window(window_attributes).unwrap();

            self.state = Some(State::new(window, &self.image, self.config));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        use WindowEvent as WE;
        if let Some(state) = &mut self.state {
            let should_request_redraw = self.config.dvd_logo_enabled
                || matches!(
                    event,
                    WE::Resized(_)
                        | WE::ScaleFactorChanged { .. }
                        | WE::KeyboardInput { .. }
                        | WE::ModifiersChanged(_)
                        | WE::MouseInput { .. }
                        | WE::MouseWheel { .. }
                        | WE::CursorMoved { .. }
                );

            match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => event_loop.exit(),
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
                state.request_window_redraw();
            }
        }
    }
}
