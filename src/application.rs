use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{self, gio, glib};
use std::cell::OnceCell;
use std::sync::OnceLock;

use crate::capture;
use crate::window::GnomeShotWindow;

// Global Tokio runtime for async D-Bus operations
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GnomeShotApplication {
        pub window: OnceCell<GnomeShotWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GnomeShotApplication {
        const NAME: &'static str = "GnomeShotApplication";
        type Type = super::GnomeShotApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for GnomeShotApplication {}

    impl ApplicationImpl for GnomeShotApplication {
        fn activate(&self) {
            let app = self.obj();
            app.present_window();
        }

        fn startup(&self) {
            self.parent_startup();
            let app = self.obj();
            app.setup_actions();
        }

        fn command_line(&self, command_line: &gio::ApplicationCommandLine) -> glib::ExitCode {
            let app = self.obj();
            let args = command_line.arguments();

            // Check for --capture flag
            let auto_capture = args.iter().any(|arg| arg.to_str() == Some("--capture"));

            app.present_window();

            if auto_capture {
                // Trigger capture after window is shown
                glib::idle_add_local_once(glib::clone!(
                    #[weak]
                    app,
                    move || {
                        app.capture_screenshot();
                    }
                ));
            }

            glib::ExitCode::SUCCESS
        }
    }

    impl GtkApplicationImpl for GnomeShotApplication {}
    impl AdwApplicationImpl for GnomeShotApplication {}
}

glib::wrapper! {
    pub struct GnomeShotApplication(ObjectSubclass<imp::GnomeShotApplication>)
        @extends adw::Application, gtk::Application, gio::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl GnomeShotApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", "org.gnome.GnomeShot")
            .property("flags", gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build()
    }

    fn present_window(&self) {
        let imp = self.imp();
        let window = imp.window.get_or_init(|| GnomeShotWindow::new(self));
        window.present();
    }

    fn setup_actions(&self) {
        // Capture region action
        let action_capture = gio::ActionEntry::builder("capture")
            .activate(|app: &Self, _, _| {
                app.capture_screenshot();
            })
            .build();

        // Quit action
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(|app: &Self, _, _| {
                app.quit();
            })
            .build();

        // Undo action
        let action_undo = gio::ActionEntry::builder("undo")
            .activate(|app: &Self, _, _| {
                if let Some(window) = app.imp().window.get() {
                    if let Some(canvas) = window.canvas() {
                        canvas.undo();
                    }
                }
            })
            .build();

        // Redo action
        let action_redo = gio::ActionEntry::builder("redo")
            .activate(|app: &Self, _, _| {
                if let Some(window) = app.imp().window.get() {
                    if let Some(canvas) = window.canvas() {
                        canvas.redo();
                    }
                }
            })
            .build();

        // Copy action
        let action_copy = gio::ActionEntry::builder("copy")
            .activate(|app: &Self, _, _| {
                if let Some(window) = app.imp().window.get() {
                    if let Some(canvas) = window.canvas() {
                        if let Some(texture) = canvas.export_to_texture() {
                            if let Some(display) = gtk::gdk::Display::default() {
                                let clipboard = display.clipboard();
                                clipboard.set_texture(&texture);
                                eprintln!("Copied to clipboard!");
                            }
                        }
                    }
                }
            })
            .build();

        // Save action
        let action_save = gio::ActionEntry::builder("save")
            .activate(|app: &Self, _, _| {
                app.show_save_dialog();
            })
            .build();

        // Quick save and exit (Escape key)
        let action_quick_save = gio::ActionEntry::builder("quick-save")
            .activate(|app: &Self, _, _| {
                app.quick_save_and_exit();
            })
            .build();

        self.add_action_entries([action_capture, action_quit, action_undo, action_redo, action_copy, action_save, action_quick_save]);

        // Set keyboard shortcuts
        self.set_accels_for_action("app.capture", &["<Primary>n"]);
        self.set_accels_for_action("app.quit", &["<Primary>q"]);
        self.set_accels_for_action("app.undo", &["<Primary>z"]);
        self.set_accels_for_action("app.redo", &["<Primary><Shift>z"]);
        self.set_accels_for_action("app.copy", &["<Primary>c"]);
        self.set_accels_for_action("app.save", &["<Primary>s"]);
        self.set_accels_for_action("app.quick-save", &["Escape"]);
    }

    pub fn capture_screenshot(&self) {
        eprintln!("Capture button clicked!");
        let app = self.clone();

        // Use std channel for thread communication (only send PathBuf which is Send)
        let (sender, receiver) = std::sync::mpsc::channel::<anyhow::Result<std::path::PathBuf>>();

        // Spawn the async capture on Tokio runtime
        runtime().spawn(async move {
            eprintln!("Starting capture on Tokio runtime...");
            let result = capture::capture_interactive_path().await;
            let _ = sender.send(result);
        });

        // Poll for result on GTK main thread using idle callback
        glib::idle_add_local(move || {
            match receiver.try_recv() {
                Ok(Ok(path)) => {
                    eprintln!("Capture path received: {:?}", path);
                    // Load the pixbuf on the GTK thread
                    match capture::load_screenshot_from_path(path) {
                        Ok(screenshot) => {
                            eprintln!("Screenshot loaded successfully!");
                            app.open_editor(screenshot);
                        }
                        Err(e) => {
                            eprintln!("Failed to load screenshot: {}", e);
                        }
                    }
                    glib::ControlFlow::Break
                }
                Ok(Err(e)) => {
                    eprintln!("Screenshot capture failed: {}", e);
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Keep polling
                    glib::ControlFlow::Continue
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    eprintln!("Channel disconnected");
                    glib::ControlFlow::Break
                }
            }
        });
    }

    fn open_editor(&self, screenshot: capture::Screenshot) {
        let window = self.imp().window.get().expect("Window not initialized");
        window.load_screenshot(screenshot);
        window.present();
    }

    fn quick_save_and_exit(&self) {
        let Some(window) = self.imp().window.get() else { return };
        let Some(canvas) = window.canvas() else {
            // No screenshot loaded, just quit
            self.quit();
            return;
        };

        // Save to default folder first
        let screenshots_dir = Self::get_screenshots_dir();

        // Create screenshots directory if it doesn't exist
        if !screenshots_dir.exists() {
            let _ = std::fs::create_dir_all(&screenshots_dir);
        }

        // Generate filename with timestamp
        let now = glib::DateTime::now_local().unwrap();
        let filename = format!("screenshot-{}.png", now.format("%Y%m%d-%H%M%S").unwrap());
        let path = screenshots_dir.join(filename);

        if let Err(e) = canvas.save_to_file(&path) {
            eprintln!("Failed to save: {}", e);
        } else {
            eprintln!("Saved to: {}", path.display());
        }

        // Copy to clipboard using wl-copy (works better on Wayland)
        // Use the saved file to copy to clipboard
        let path_clone = path.clone();
        std::thread::spawn(move || {
            // Try wl-copy first (Wayland)
            let result = std::process::Command::new("wl-copy")
                .arg("--type")
                .arg("image/png")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    if let Some(stdin) = child.stdin.as_mut() {
                        let data = std::fs::read(&path_clone)?;
                        use std::io::Write;
                        stdin.write_all(&data)?;
                    }
                    child.wait()
                });

            match result {
                Ok(status) if status.success() => eprintln!("Copied to clipboard with wl-copy!"),
                _ => {
                    // Fallback to xclip (X11)
                    let _ = std::process::Command::new("xclip")
                        .arg("-selection")
                        .arg("clipboard")
                        .arg("-t")
                        .arg("image/png")
                        .arg("-i")
                        .arg(&path_clone)
                        .status();
                    eprintln!("Copied to clipboard with xclip!");
                }
            }
        });

        // Small delay to let clipboard copy start
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Exit the app
        self.quit();
    }

    fn get_screenshots_dir() -> std::path::PathBuf {
        // Check for config file first
        let config_dir = glib::user_config_dir().join("gnome-shot");
        let config_file = config_dir.join("config");

        if config_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_file) {
                for line in content.lines() {
                    if let Some(path) = line.strip_prefix("save_dir=") {
                        let path = std::path::PathBuf::from(path.trim());
                        if path.exists() || std::fs::create_dir_all(&path).is_ok() {
                            return path;
                        }
                    }
                }
            }
        }

        // Default: ~/Pictures/Screenshots/
        let pictures_dir = glib::user_special_dir(glib::UserDirectory::Pictures)
            .unwrap_or_else(|| std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()));
        pictures_dir.join("Screenshots")
    }

    pub fn set_screenshots_dir(path: &std::path::Path) -> std::io::Result<()> {
        let config_dir = glib::user_config_dir().join("gnome-shot");
        std::fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("config");
        std::fs::write(&config_file, format!("save_dir={}\n", path.display()))
    }

    fn show_save_dialog(&self) {
        let Some(window) = self.imp().window.get() else { return };
        let Some(canvas) = window.canvas() else { return };

        // Create file dialog
        let dialog = gtk::FileDialog::builder()
            .title("Save Screenshot")
            .modal(true)
            .build();

        // Set default filename with timestamp
        let now = glib::DateTime::now_local().unwrap();
        let filename = format!("screenshot-{}.png", now.format("%Y%m%d-%H%M%S").unwrap());
        dialog.set_initial_name(Some(&filename));

        // Add PNG filter
        let filter = gtk::FileFilter::new();
        filter.add_pattern("*.png");
        filter.set_name(Some("PNG Images"));
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        dialog.save(Some(window), gio::Cancellable::NONE, move |result: Result<gio::File, glib::Error>| {
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    if let Err(e) = canvas.save_to_file(&path) {
                        eprintln!("Failed to save: {}", e);
                    } else {
                        eprintln!("Saved to: {}", path.display());
                    }
                }
            }
        });
    }
}
