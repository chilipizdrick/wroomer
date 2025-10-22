mod application;
mod cli_args;
mod screenshot;

use clap::Parser;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    platform::wayland::EventLoopBuilderExtWayland,
};

use crate::{application::App, cli_args::Args, screenshot::get_screenshot_all_screens};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    let event_loop = EventLoop::builder().with_wayland().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let image = match args.image_path {
        Some(path) => image::open(path)?,
        None => get_screenshot_all_screens()?,
    };

    let mut app = App::new(image);
    event_loop.run_app(&mut app).map_err(Into::into)
}
