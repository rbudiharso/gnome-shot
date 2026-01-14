use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib, gdk};
use std::cell::RefCell;

use crate::application::GnomeShotApplication;
use crate::canvas::{CanvasWidget, Tool};
use crate::capture::Screenshot;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct GnomeShotWindow {
        pub canvas: RefCell<Option<CanvasWidget>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GnomeShotWindow {
        const NAME: &'static str = "GnomeShotWindow";
        type Type = super::GnomeShotWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for GnomeShotWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_ui();
        }
    }

    impl WidgetImpl for GnomeShotWindow {}
    impl WindowImpl for GnomeShotWindow {}
    impl ApplicationWindowImpl for GnomeShotWindow {}
    impl AdwApplicationWindowImpl for GnomeShotWindow {}
}

glib::wrapper! {
    pub struct GnomeShotWindow(ObjectSubclass<imp::GnomeShotWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl GnomeShotWindow {
    pub fn new(app: &GnomeShotApplication) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    fn setup_ui(&self) {
        self.set_title(Some("GNOME Shot"));
        self.set_default_size(900, 600);

        // Create header bar
        let header = adw::HeaderBar::new();

        let capture_btn = gtk::Button::builder()
            .label("Capture")
            .action_name("app.capture")
            .build();
        capture_btn.add_css_class("suggested-action");
        header.pack_start(&capture_btn);

        // Create status page for empty state
        let status_page = adw::StatusPage::builder()
            .icon_name("camera-photo-symbolic")
            .title("Welcome to GNOME Shot")
            .description("Click Capture or press Ctrl+N to take a screenshot")
            .build();

        // Create toolbar view
        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&status_page));

        self.set_content(Some(&toolbar_view));
    }

    pub fn load_screenshot(&self, screenshot: Screenshot) {
        let imp = self.imp();

        // Create canvas widget
        let canvas = CanvasWidget::new();
        canvas.set_hexpand(true);
        canvas.set_vexpand(true);

        // Create header bar
        let header = adw::HeaderBar::new();

        let capture_btn = gtk::Button::builder()
            .label("Capture")
            .action_name("app.capture")
            .build();
        capture_btn.add_css_class("suggested-action");
        header.pack_start(&capture_btn);

        // Copy button
        let copy_btn = gtk::Button::builder()
            .icon_name("edit-copy-symbolic")
            .tooltip_text("Copy to clipboard (Ctrl+C)")
            .build();

        let canvas_for_copy = canvas.clone();
        copy_btn.connect_clicked(move |_| {
            if let Some(texture) = canvas_for_copy.export_to_texture() {
                if let Some(display) = gdk::Display::default() {
                    let clipboard = display.clipboard();
                    clipboard.set_texture(&texture);
                    eprintln!("Copied to clipboard!");
                }
            }
        });

        // Save button
        let save_btn = gtk::Button::builder()
            .icon_name("document-save-symbolic")
            .tooltip_text("Save screenshot (Ctrl+S)")
            .build();

        let canvas_for_save = canvas.clone();
        let window_for_save = self.clone();
        save_btn.connect_clicked(move |_| {
            let canvas = canvas_for_save.clone();
            let window = window_for_save.clone();

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

            dialog.save(Some(&window), gio::Cancellable::NONE, move |result: Result<gio::File, glib::Error>| {
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
        });

        header.pack_end(&save_btn);
        header.pack_end(&copy_btn);

        // Create annotation toolbar
        let toolbar = self.create_annotation_toolbar(&canvas);

        // Main content box
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_box.append(&toolbar);
        content_box.append(&canvas);

        // Create toolbar view
        let toolbar_view = adw::ToolbarView::new();
        toolbar_view.add_top_bar(&header);
        toolbar_view.set_content(Some(&content_box));

        self.set_content(Some(&toolbar_view));

        // Load screenshot into canvas
        canvas.load_screenshot(screenshot);

        // Store the canvas
        imp.canvas.replace(Some(canvas));
    }

    fn create_annotation_toolbar(&self, canvas: &CanvasWidget) -> gtk::Box {
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        toolbar.set_margin_start(6);
        toolbar.set_margin_end(6);
        toolbar.set_margin_top(6);
        toolbar.set_margin_bottom(6);

        // Tool buttons with labels
        let arrow_btn = gtk::ToggleButton::builder()
            .label("Arrow")
            .tooltip_text("Arrow tool (draw arrows)")
            .active(true)
            .build();

        let rect_btn = gtk::ToggleButton::builder()
            .label("Rect")
            .tooltip_text("Rectangle tool (draw rectangles)")
            .build();

        let line_btn = gtk::ToggleButton::builder()
            .label("Line")
            .tooltip_text("Line tool (draw lines)")
            .build();

        let ellipse_btn = gtk::ToggleButton::builder()
            .label("Ellipse")
            .tooltip_text("Ellipse tool (draw ellipses)")
            .build();

        let highlight_btn = gtk::ToggleButton::builder()
            .label("Highlight")
            .tooltip_text("Highlight tool (semi-transparent)")
            .build();

        let blur_btn = gtk::ToggleButton::builder()
            .label("Blur")
            .tooltip_text("Blur tool (redact sensitive info)")
            .build();

        // Group the toggle buttons
        rect_btn.set_group(Some(&arrow_btn));
        line_btn.set_group(Some(&arrow_btn));
        ellipse_btn.set_group(Some(&arrow_btn));
        highlight_btn.set_group(Some(&arrow_btn));
        blur_btn.set_group(Some(&arrow_btn));

        // Connect tool buttons
        let canvas_for_arrow = canvas.clone();
        arrow_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas_for_arrow.set_tool(Tool::Arrow);
            }
        });

        let canvas_for_rect = canvas.clone();
        rect_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas_for_rect.set_tool(Tool::Rectangle);
            }
        });

        let canvas_for_line = canvas.clone();
        line_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas_for_line.set_tool(Tool::Line);
            }
        });

        let canvas_for_ellipse = canvas.clone();
        ellipse_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas_for_ellipse.set_tool(Tool::Ellipse);
            }
        });

        let canvas_for_highlight = canvas.clone();
        highlight_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas_for_highlight.set_tool(Tool::Highlight);
            }
        });

        let canvas_for_blur = canvas.clone();
        blur_btn.connect_toggled(move |btn| {
            if btn.is_active() {
                canvas_for_blur.set_tool(Tool::Blur);
            }
        });

        // Color button using MenuButton with color indicator
        let color_indicator = gtk::DrawingArea::builder()
            .width_request(20)
            .height_request(20)
            .build();

        // Set initial color (red)
        let initial_color = gdk::RGBA::new(1.0, 0.0, 0.0, 1.0);
        color_indicator.set_draw_func({
            let color = initial_color;
            move |_, cr, width, height| {
                cr.set_source_rgba(
                    color.red() as f64,
                    color.green() as f64,
                    color.blue() as f64,
                    color.alpha() as f64,
                );
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();
                // Draw border
                cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
                cr.set_line_width(1.0);
                cr.rectangle(0.5, 0.5, width as f64 - 1.0, height as f64 - 1.0);
                let _ = cr.stroke();
            }
        });

        let color_btn = gtk::MenuButton::builder()
            .tooltip_text("Annotation color")
            .build();
        color_btn.set_child(Some(&color_indicator));

        // Create color popover with preset colors
        let color_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        color_box.set_margin_start(6);
        color_box.set_margin_end(6);
        color_box.set_margin_top(6);
        color_box.set_margin_bottom(6);

        let colors = [
            ("Red", gdk::RGBA::new(1.0, 0.0, 0.0, 1.0)),
            ("Green", gdk::RGBA::new(0.0, 0.8, 0.0, 1.0)),
            ("Blue", gdk::RGBA::new(0.0, 0.4, 1.0, 1.0)),
            ("Yellow", gdk::RGBA::new(1.0, 0.9, 0.0, 1.0)),
            ("Orange", gdk::RGBA::new(1.0, 0.5, 0.0, 1.0)),
            ("Purple", gdk::RGBA::new(0.6, 0.2, 0.8, 1.0)),
            ("Black", gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)),
            ("White", gdk::RGBA::new(1.0, 1.0, 1.0, 1.0)),
        ];

        let color_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        for (name, color) in colors {
            let btn = gtk::Button::new();
            btn.set_tooltip_text(Some(name));
            btn.set_size_request(24, 24);

            // Set button color via CSS
            let css = format!(
                "button {{ background: rgba({},{},{},{}); min-width: 24px; min-height: 24px; }}",
                (color.red() * 255.0) as u8,
                (color.green() * 255.0) as u8,
                (color.blue() * 255.0) as u8,
                color.alpha()
            );
            let provider = gtk::CssProvider::new();
            provider.load_from_data(&css);
            btn.style_context().add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

            let canvas_clone = canvas.clone();
            let popover_clone = color_btn.clone();
            let indicator_clone = color_indicator.clone();
            btn.connect_clicked(move |_| {
                canvas_clone.set_color(color);
                // Update the color indicator
                indicator_clone.set_draw_func({
                    move |_, cr, width, height| {
                        cr.set_source_rgba(
                            color.red() as f64,
                            color.green() as f64,
                            color.blue() as f64,
                            color.alpha() as f64,
                        );
                        cr.rectangle(0.0, 0.0, width as f64, height as f64);
                        let _ = cr.fill();
                        // Draw border
                        cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
                        cr.set_line_width(1.0);
                        cr.rectangle(0.5, 0.5, width as f64 - 1.0, height as f64 - 1.0);
                        let _ = cr.stroke();
                    }
                });
                indicator_clone.queue_draw();
                popover_clone.popdown();
            });

            color_row.append(&btn);
        }
        color_box.append(&color_row);

        let popover = gtk::Popover::new();
        popover.set_child(Some(&color_box));
        color_btn.set_popover(Some(&popover));

        // Undo button
        let undo_btn = gtk::Button::builder()
            .icon_name("edit-undo-symbolic")
            .tooltip_text("Undo (Ctrl+Z)")
            .build();

        let canvas_for_undo = canvas.clone();
        undo_btn.connect_clicked(move |_| {
            canvas_for_undo.undo();
        });

        // Redo button
        let redo_btn = gtk::Button::builder()
            .icon_name("edit-redo-symbolic")
            .tooltip_text("Redo (Ctrl+Shift+Z)")
            .build();

        let canvas_for_redo = canvas.clone();
        redo_btn.connect_clicked(move |_| {
            canvas_for_redo.redo();
        });

        // Add separator
        let separator = gtk::Separator::new(gtk::Orientation::Vertical);

        // Add widgets to toolbar
        toolbar.append(&arrow_btn);
        toolbar.append(&rect_btn);
        toolbar.append(&line_btn);
        toolbar.append(&ellipse_btn);
        toolbar.append(&highlight_btn);
        toolbar.append(&blur_btn);
        toolbar.append(&separator);
        toolbar.append(&color_btn);

        let separator2 = gtk::Separator::new(gtk::Orientation::Vertical);
        toolbar.append(&separator2);
        toolbar.append(&undo_btn);
        toolbar.append(&redo_btn);

        toolbar
    }

    pub fn canvas(&self) -> Option<CanvasWidget> {
        self.imp().canvas.borrow().clone()
    }
}
