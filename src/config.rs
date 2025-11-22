#[derive(Debug, Default, Clone, Copy)]
pub struct AppConfig {
    pub dvd_logo_enabled: bool,
}

impl AppConfig {
    pub fn new(dvd_logo_enabled: bool) -> Self {
        Self { dvd_logo_enabled }
    }
}
