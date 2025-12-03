use cli_args::Args;

use crate::cli_args;

#[derive(Debug, Default, Clone, Copy)]
pub struct AppConfig {
    pub fullscreen: bool,
    pub center_image_on_resize: bool,
}

impl From<Args> for AppConfig {
    fn from(value: Args) -> Self {
        Self {
            fullscreen: value.fullscreen,
            center_image_on_resize: !value.disable_image_centering_on_window_resize,
        }
    }
}
