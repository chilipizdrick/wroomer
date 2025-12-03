// Prevent console window from showing up on windows
#![windows_subsystem = "windows"]

#[cfg(all(feature = "default", feature = "wayland"))]
compile_error!("Complile with either default feature, or wayland.");

mod application;
mod cli_args;
mod config;
mod screenshot;

use clap::Parser;
use image::DynamicImage;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};
use winit::{error::EventLoopError, event_loop::EventLoop};

use crate::{
    application::App, cli_args::Args, config::AppConfig, screenshot::get_screenshot_of_all_screens,
};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let image = load_image(&args)?;

    let config = AppConfig::from(args);

    let window_event_loop = create_window_event_loop()?;
    let mut app = App::new(config, image);
    window_event_loop.run_app(&mut app).map_err(Into::into)
}

fn load_image(args: &Args) -> anyhow::Result<DynamicImage> {
    match &args.image_path {
        Some(path) => image::open(path).map_err(Into::into),
        None if args.capture_screenshot => get_screenshot_of_all_screens(),
        _ => Err(anyhow::Error::msg(
            "Provide image path or use --capture-screenshot flag.",
        )),
    }
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
