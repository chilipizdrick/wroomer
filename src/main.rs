// Prevent console window from showing up on windows
#![windows_subsystem = "windows"]

mod application;
mod cli_args;
mod config;
mod screenshot;

use clap::Parser;
use winit::{error::EventLoopError, event_loop::EventLoop};

use crate::{
    application::App, cli_args::Args, config::AppConfig, screenshot::get_screenshot_of_all_screens,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    let config = AppConfig::new(args.dvd_logo);

    let window_event_loop = create_window_event_loop()?;

    let image = match args.image_path {
        Some(path) => image::open(path)?,
        None => get_screenshot_of_all_screens()?,
    };

    let mut app = App::new(image, config);
    window_event_loop.run_app(&mut app).map_err(Into::into)
}

fn create_window_event_loop() -> Result<EventLoop<()>, EventLoopError> {
    use winit::platform::wayland::EventLoopBuilderExtWayland;

    let event_loop = if cfg!(feature = "wayland") {
        EventLoop::builder().with_wayland().build()?
    } else {
        EventLoop::new()?
    };

    Ok(event_loop)
}
