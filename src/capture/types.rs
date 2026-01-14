use gtk::gdk_pixbuf::Pixbuf;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Region,
    Window,
    Fullscreen,
}

#[derive(Debug)]
pub struct Screenshot {
    pub pixbuf: Pixbuf,
    pub source_path: Option<PathBuf>,
    pub capture_mode: CaptureMode,
}

impl Screenshot {
    pub fn width(&self) -> i32 {
        self.pixbuf.width()
    }

    pub fn height(&self) -> i32 {
        self.pixbuf.height()
    }
}
