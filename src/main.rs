mod application;
mod cli_args;
mod config;
mod screenshot;

use clap::Parser;
use winit::event_loop::{ControlFlow, EventLoop};

#[cfg(feature = "wayland")]
use winit::platform::wayland::EventLoopBuilderExtWayland;

use crate::{
    application::App, cli_args::Args, config::AppConfig, screenshot::get_screenshot_of_all_screens,
};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = AppConfig::new(args.dvd_logo);

    tracing_subscriber::fmt().init();

    #[cfg(feature = "wayland")]
    let window_event_loop = EventLoop::builder().with_wayland().build()?;

    #[cfg(not(feature = "wayland"))]
    let window_event_loop = EventLoop::new()?;

    let controll_flow = if args.dvd_logo {
        ControlFlow::Poll
    } else {
        ControlFlow::Wait
    };

    window_event_loop.set_control_flow(controll_flow);

    let image = match args.image_path {
        Some(path) => image::open(path)?,
        None => get_screenshot_of_all_screens()?,
    };

    let mut app = App::new(image, config);
    window_event_loop.run_app(&mut app).map_err(Into::into)
}
