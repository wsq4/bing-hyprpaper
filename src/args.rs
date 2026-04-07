use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Run in daemon mode
    #[arg(short, long, default_value_t = false)]
    pub daemon: bool,

    /// Width of the image to download
    #[arg(short = 'W', long, default_value_t = 1920)]
    pub width: u32,

    /// Height of the image to download
    #[arg(short = 'H', long, default_value_t = 1080)]
    pub height: u32,

    /// Directory to store downloaded images
    #[arg(short, long, default_value = "./images")]
    pub storage_path: String,

    /// Switch Desktop Wallpaper Interval in seconds
    #[arg(short, long, default_value_t = 30)]
    pub interval: u64,

    /// Maximum number of images to keep in storage
    #[arg(short, long, default_value_t = 10)]
    pub max_images: usize,

    /// Refresh image list interval in hours
    #[arg(short, long, default_value_t = 6)]
    pub refresh_interval: u64,
}