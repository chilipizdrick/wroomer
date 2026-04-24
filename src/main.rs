// Prevent console window from showing up on windows
#![windows_subsystem = "windows"]

#[cfg(all(feature = "default", feature = "wayland"))]
compile_error!("Complile with either default feature, or wayland.");

mod app;
mod cli_args;
mod config;
mod screenshot;

use std::io::{Cursor, Read, stdin};

use anyhow::anyhow;
use clap::Parser;
use image::{DynamicImage, ImageReader};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};
use winit::{error::EventLoopError, event_loop::EventLoop};

use crate::{
    app::App, cli_args::Args, config::AppConfig, screenshot::get_screenshot_of_all_screens,
};

fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let image = load_image(&args)?;
    let config = AppConfig::from(args);

    let window_event_loop = create_window_event_loop()?;
    let mut app = App::new(config, image);
    window_event_loop.run_app(&mut app)?;

    Ok(())
}

fn load_image(args: &Args) -> anyhow::Result<DynamicImage> {
    match &args.image_path {
        Some(path) => match path.as_str() {
            "-" => {
                let mut bytes = Vec::<u8>::new();
                stdin().lock().read_to_end(&mut bytes)?;
                let cursor = Cursor::new(bytes);
                let reader = ImageReader::new(cursor).with_guessed_format()?;
                reader.decode().map_err(Into::into)
            }
            path => image::open(path).map_err(Into::into),
        },
        None if args.capture_screenshot => get_screenshot_of_all_screens(),
        _ => Err(anyhow!(
            "Provide image path or use --capture-screenshot flag."
        )),
    }
}

fn create_window_event_loop() -> Result<EventLoop<()>, EventLoopError> {
    if cfg!(feature = "wayland") {
        use winit::platform::wayland::EventLoopBuilderExtWayland;
        EventLoop::builder().with_wayland().build()
    } else {
        EventLoop::new()
    }
}

fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
