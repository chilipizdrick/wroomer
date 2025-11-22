pub use platform_specific::get_screenshot_of_all_screens;

#[cfg(not(feature = "wayland"))]
mod platform_specific {
    use image::{DynamicImage, GenericImage};
    use xcap::{Monitor, XCapResult};

    pub type ScreenshotResult = Result<DynamicImage, anyhow::Error>;

    pub fn get_screenshot_of_all_screens() -> ScreenshotResult {
        let monitors = Monitor::all()?;
        let screenshots = monitors
            .iter()
            .map(|m| m.capture_image())
            .collect::<XCapResult<Vec<_>>>()?;

        if screenshots.is_empty() {
            return Err(anyhow::Error::msg("No monitors detected"));
        }

        if screenshots.len() == 1 {
            let screenshot = screenshots.into_iter().next().unwrap();
            return Ok(screenshot.into());
        }

        let composed_height = screenshots.iter().map(|s| s.height()).max().unwrap();
        let composed_width = screenshots.iter().map(|s| s.width()).sum();
        let mut composed_screenshot =
            DynamicImage::new(composed_width, composed_height, image::ColorType::Rgba8);
        let mut current_image_offset = 0;

        for screenshot in screenshots {
            composed_screenshot
                .copy_from(&screenshot, current_image_offset, 0)
                .unwrap();
            current_image_offset += screenshot.width();
        }

        Ok(composed_screenshot)
    }
}

#[cfg(feature = "wayland")]
mod platform_specific {
    use image::DynamicImage;

    pub type ScreenshotResult = Result<DynamicImage, libwayshot::Error>;

    pub fn get_screenshot_of_all_screens() -> ScreenshotResult {
        let connection = libwayshot::WayshotConnection::new()?;
        let screenshot = connection.screenshot_all(false)?;
        Ok(screenshot)
    }
}
