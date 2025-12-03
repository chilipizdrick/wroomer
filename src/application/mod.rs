mod image_render_pipeline;
mod spotlight_render_pipeline;
mod state;
mod texture;

use image::{DynamicImage, RgbaImage};
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
    image: RgbaImage,
    config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig, image: DynamicImage) -> Self {
        let image = match image {
            DynamicImage::ImageRgba8(img) => img,
            _ => image.to_rgba8(),
        };

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
            let fullscreen = match self.config.fullscreen {
                true => Some(Fullscreen::Borderless(None)),
                false => None,
            };

            let window_attributes = Window::default_attributes()
                .with_title("Wroomer")
                .with_fullscreen(fullscreen)
                .with_transparent(true)
                .with_blur(true);

            let window = match event_loop.create_window(window_attributes) {
                Ok(w) => w,
                Err(err) => panic!("Could not create window: {err}"),
            };

            self.state = Some(State::new(self.config, window, &self.image));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(state) = &mut self.state {
            state.handle_window_event(event_loop, event);
        }
    }
}
