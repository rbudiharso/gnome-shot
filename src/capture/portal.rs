use ashpd::desktop::screenshot::Screenshot as PortalScreenshot;
use gtk::gdk_pixbuf::Pixbuf;
use std::path::PathBuf;

use super::{CaptureMode, Screenshot};

/// Capture a screenshot using the XDG Desktop Portal (returns path only, for thread safety)
pub async fn capture_interactive_path() -> anyhow::Result<PathBuf> {
    let response = PortalScreenshot::request()
        .interactive(true)
        .modal(true)
        .send()
        .await?
        .response()?;

    let uri = response.uri();
    uri_to_path(uri)
}

/// Load a Screenshot from a file path (must be called on GTK thread)
pub fn load_screenshot_from_path(path: PathBuf) -> anyhow::Result<Screenshot> {
    let pixbuf = Pixbuf::from_file(&path)?;
    Ok(Screenshot {
        pixbuf,
        source_path: Some(path),
        capture_mode: CaptureMode::Region,
    })
}

/// Capture a screenshot using the XDG Desktop Portal
/// This shows GNOME's native screenshot dialog for region/window selection
pub async fn capture_interactive() -> anyhow::Result<Screenshot> {
    let path = capture_interactive_path().await?;
    load_screenshot_from_path(path)
}

/// Capture the entire screen without interactive dialog
pub async fn capture_fullscreen() -> anyhow::Result<Screenshot> {
    let response = PortalScreenshot::request()
        .interactive(false)
        .modal(false)
        .send()
        .await?
        .response()?;

    let uri = response.uri();
    let path = uri_to_path(uri)?;

    let pixbuf = Pixbuf::from_file(&path)?;

    Ok(Screenshot {
        pixbuf,
        source_path: Some(path),
        capture_mode: CaptureMode::Fullscreen,
    })
}

fn uri_to_path(uri: &url::Url) -> anyhow::Result<PathBuf> {
    uri.to_file_path()
        .map_err(|_| anyhow::anyhow!("Invalid file URI: {}", uri))
}
