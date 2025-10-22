use image::DynamicImage;
use libwayshot::WayshotConnection;

type ScreenshotResult = Result<DynamicImage, libwayshot::Error>;

pub fn get_screenshot_all_screens() -> ScreenshotResult {
    let wayshot_connection = WayshotConnection::new()?;
    let cursor_overlay = false;
    wayshot_connection.screenshot_all(cursor_overlay)
}
