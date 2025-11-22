use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to the image to be displayed
    #[arg(short, long)]
    pub image_path: Option<String>,
    /// DVD icon easter egg
    #[arg(short, long, action)]
    pub dvd_logo: bool,
}
