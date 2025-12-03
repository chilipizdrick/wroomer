use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the image to be displayed. Leave empty to display a screenshot of all screens
    pub image_path: Option<String>,

    /// Open window in fullscreen
    #[arg(short, long)]
    pub fullscreen: bool,

    /// Capture screenshot of all monitors
    #[arg(short, long)]
    pub capture_screenshot: bool,

    /// Disable image centering on window resize
    #[arg(short, long)]
    pub disable_image_centering_on_window_resize: bool,
}
