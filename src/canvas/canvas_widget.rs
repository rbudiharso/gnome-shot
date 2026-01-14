use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};
use std::cell::{Cell, RefCell};

use super::annotations::{Annotation, ArrowAnnotation, BlurAnnotation, EllipseAnnotation, HighlightAnnotation, LineAnnotation, Point, RectAnnotation};
use super::history::History;
use super::tools::Tool;
use crate::capture::Screenshot;

mod imp {
    use super::*;

    pub struct CanvasWidget {
        pub screenshot: RefCell<Option<Screenshot>>,
        pub annotations: RefCell<Vec<Annotation>>,
        pub history: RefCell<History>,
        pub current_tool: Cell<Tool>,
        pub primary_color: RefCell<gdk::RGBA>,
        pub stroke_width: Cell<f64>,
        pub scale: Cell<f64>,
        pub offset_x: Cell<f64>,
        pub offset_y: Cell<f64>,
        pub drag_start: Cell<Option<(f64, f64)>>,
        pub drag_current: Cell<Option<(f64, f64)>>,
        pub is_drawing: Cell<bool>,
    }

    impl Default for CanvasWidget {
        fn default() -> Self {
            Self {
                screenshot: RefCell::new(None),
                annotations: RefCell::new(Vec::new()),
                history: RefCell::new(History::new()),
                current_tool: Cell::new(Tool::Arrow),
                primary_color: RefCell::new(gdk::RGBA::new(1.0, 0.0, 0.0, 1.0)),
                stroke_width: Cell::new(3.0),
                scale: Cell::new(1.0),
                offset_x: Cell::new(0.0),
                offset_y: Cell::new(0.0),
                drag_start: Cell::new(None),
                drag_current: Cell::new(None),
                is_drawing: Cell::new(false),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasWidget {
        const NAME: &'static str = "GnomeShotCanvas";
        type Type = super::CanvasWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("canvas");
        }
    }

    impl ObjectImpl for CanvasWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            // Set up event controllers
            obj.setup_event_controllers();
        }
    }

    impl WidgetImpl for CanvasWidget {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f64;
            let height = widget.height() as f64;

            if width <= 0.0 || height <= 0.0 {
                return;
            }

            // Create Cairo context
            let bounds = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
            let cr = snapshot.append_cairo(&bounds);

            // Draw background
            cr.set_source_rgb(0.15, 0.15, 0.15);
            let _ = cr.paint();

            // Draw screenshot if available
            if let Some(ref screenshot) = *self.screenshot.borrow() {
                let scale = self.scale.get();
                let offset_x = self.offset_x.get();
                let offset_y = self.offset_y.get();

                cr.save().unwrap();
                cr.translate(offset_x, offset_y);
                cr.scale(scale, scale);

                // Draw the pixbuf using gdk_cairo_set_source_pixbuf equivalent
                let pixbuf = &screenshot.pixbuf;
                gtk::gdk::prelude::GdkCairoContextExt::set_source_pixbuf(&cr, pixbuf, 0.0, 0.0);
                let _ = cr.paint();

                // Draw existing annotations
                for annotation in self.annotations.borrow().iter() {
                    annotation.draw(&cr, 1.0);
                }

                // Draw current annotation being created
                if self.is_drawing.get() {
                    if let (Some(start), Some(current)) = (self.drag_start.get(), self.drag_current.get()) {
                        let start_pt = Point::new(
                            (start.0 - offset_x) / scale,
                            (start.1 - offset_y) / scale,
                        );
                        let end_pt = Point::new(
                            (current.0 - offset_x) / scale,
                            (current.1 - offset_y) / scale,
                        );

                        let color = self.primary_color.borrow().clone();
                        let stroke_width = self.stroke_width.get();

                        match self.current_tool.get() {
                            Tool::Arrow => {
                                let arrow = ArrowAnnotation::new(start_pt, end_pt, color, stroke_width);
                                arrow.draw(&cr, 1.0);
                            }
                            Tool::Rectangle => {
                                let rect = RectAnnotation::new(start_pt, end_pt, color, stroke_width, false);
                                rect.draw(&cr, 1.0);
                            }
                            Tool::Line => {
                                let line = LineAnnotation::new(start_pt, end_pt, color, stroke_width);
                                line.draw(&cr, 1.0);
                            }
                            Tool::Ellipse => {
                                let ellipse = EllipseAnnotation::new(start_pt, end_pt, color, stroke_width, false);
                                ellipse.draw(&cr, 1.0);
                            }
                            Tool::Highlight => {
                                let highlight = HighlightAnnotation::new(start_pt, end_pt, color);
                                highlight.draw(&cr, 1.0);
                            }
                            Tool::Blur => {
                                let blur = BlurAnnotation::new(start_pt, end_pt);
                                blur.draw(&cr, 1.0);
                            }
                        }
                    }
                }

                cr.restore().unwrap();
            }
        }

        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                gtk::Orientation::Horizontal => (400, 800, -1, -1),
                gtk::Orientation::Vertical => (300, 600, -1, -1),
                _ => (100, 100, -1, -1),
            }
        }
    }
}

glib::wrapper! {
    pub struct CanvasWidget(ObjectSubclass<imp::CanvasWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for CanvasWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup_event_controllers(&self) {
        // Mouse click controller for drawing
        let click = gtk::GestureClick::new();
        click.set_button(1); // Left mouse button

        click.connect_pressed(glib::clone!(
            #[weak(rename_to = canvas)]
            self,
            move |_, _, x, y| {
                canvas.on_press(x, y);
            }
        ));

        click.connect_released(glib::clone!(
            #[weak(rename_to = canvas)]
            self,
            move |_, _, x, y| {
                canvas.on_release(x, y);
            }
        ));

        self.add_controller(click);

        // Mouse motion controller
        let motion = gtk::EventControllerMotion::new();
        motion.connect_motion(glib::clone!(
            #[weak(rename_to = canvas)]
            self,
            move |_, x, y| {
                canvas.on_motion(x, y);
            }
        ));
        self.add_controller(motion);

        // Scroll controller for zoom
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.connect_scroll(glib::clone!(
            #[weak(rename_to = canvas)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, _, dy| {
                canvas.on_scroll(dy);
                glib::Propagation::Stop
            }
        ));
        self.add_controller(scroll);
    }

    fn on_press(&self, x: f64, y: f64) {
        let imp = self.imp();
        if imp.screenshot.borrow().is_some() {
            imp.drag_start.set(Some((x, y)));
            imp.drag_current.set(Some((x, y)));
            imp.is_drawing.set(true);
        }
    }

    fn on_motion(&self, x: f64, y: f64) {
        let imp = self.imp();
        if imp.is_drawing.get() {
            imp.drag_current.set(Some((x, y)));
            self.queue_draw();
        }
    }

    fn on_release(&self, x: f64, y: f64) {
        let imp = self.imp();

        if !imp.is_drawing.get() {
            return;
        }

        if let Some(start) = imp.drag_start.get() {
            let scale = imp.scale.get();
            let offset_x = imp.offset_x.get();
            let offset_y = imp.offset_y.get();

            let start_pt = Point::new(
                (start.0 - offset_x) / scale,
                (start.1 - offset_y) / scale,
            );
            let end_pt = Point::new(
                (x - offset_x) / scale,
                (y - offset_y) / scale,
            );

            // Only create annotation if there's meaningful distance
            let dist = ((end_pt.x - start_pt.x).powi(2) + (end_pt.y - start_pt.y).powi(2)).sqrt();
            if dist > 5.0 {
                let color = imp.primary_color.borrow().clone();
                let stroke_width = imp.stroke_width.get();

                let annotation = match imp.current_tool.get() {
                    Tool::Arrow => Annotation::Arrow(ArrowAnnotation::new(start_pt, end_pt, color, stroke_width)),
                    Tool::Rectangle => Annotation::Rectangle(RectAnnotation::new(start_pt, end_pt, color, stroke_width, false)),
                    Tool::Line => Annotation::Line(LineAnnotation::new(start_pt, end_pt, color, stroke_width)),
                    Tool::Ellipse => Annotation::Ellipse(EllipseAnnotation::new(start_pt, end_pt, color, stroke_width, false)),
                    Tool::Highlight => Annotation::Highlight(HighlightAnnotation::new(start_pt, end_pt, color)),
                    Tool::Blur => Annotation::Blur(BlurAnnotation::new(start_pt, end_pt)),
                };

                imp.annotations.borrow_mut().push(annotation.clone());
                imp.history.borrow_mut().push_add(annotation);
            }
        }

        imp.drag_start.set(None);
        imp.drag_current.set(None);
        imp.is_drawing.set(false);
        self.queue_draw();
    }

    fn on_scroll(&self, dy: f64) {
        let imp = self.imp();
        let scale = imp.scale.get();
        let new_scale = (scale * (1.0 - dy * 0.1)).clamp(0.1, 5.0);
        imp.scale.set(new_scale);
        self.queue_draw();
    }

    pub fn load_screenshot(&self, screenshot: Screenshot) {
        let imp = self.imp();

        // Calculate scale to fit
        let widget_width = self.width() as f64;
        let widget_height = self.height() as f64;
        let img_width = screenshot.width() as f64;
        let img_height = screenshot.height() as f64;

        let scale_x = if widget_width > 0.0 { widget_width / img_width } else { 1.0 };
        let scale_y = if widget_height > 0.0 { widget_height / img_height } else { 1.0 };
        let scale = scale_x.min(scale_y).min(1.0);

        // Center the image
        let offset_x = (widget_width - img_width * scale) / 2.0;
        let offset_y = (widget_height - img_height * scale) / 2.0;

        imp.scale.set(scale);
        imp.offset_x.set(offset_x.max(0.0));
        imp.offset_y.set(offset_y.max(0.0));

        imp.screenshot.replace(Some(screenshot));
        imp.annotations.borrow_mut().clear();
        imp.history.borrow_mut().clear();

        self.queue_draw();
    }

    pub fn set_tool(&self, tool: Tool) {
        self.imp().current_tool.set(tool);
    }

    pub fn current_tool(&self) -> Tool {
        self.imp().current_tool.get()
    }

    pub fn set_color(&self, color: gdk::RGBA) {
        *self.imp().primary_color.borrow_mut() = color;
    }

    pub fn current_color(&self) -> gdk::RGBA {
        self.imp().primary_color.borrow().clone()
    }

    pub fn set_stroke_width(&self, width: f64) {
        self.imp().stroke_width.set(width);
    }

    pub fn undo(&self) {
        let imp = self.imp();
        let mut annotations = imp.annotations.borrow_mut();
        let mut history = imp.history.borrow_mut();
        if history.undo(&mut annotations) {
            drop(annotations);
            drop(history);
            self.queue_draw();
        }
    }

    pub fn redo(&self) {
        let imp = self.imp();
        let mut annotations = imp.annotations.borrow_mut();
        let mut history = imp.history.borrow_mut();
        if history.redo(&mut annotations) {
            drop(annotations);
            drop(history);
            self.queue_draw();
        }
    }

    pub fn can_undo(&self) -> bool {
        self.imp().history.borrow().can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.imp().history.borrow().can_redo()
    }

    pub fn export_to_texture(&self) -> Option<gdk::Texture> {
        let imp = self.imp();
        let screenshot = imp.screenshot.borrow();
        let screenshot = screenshot.as_ref()?;

        let width = screenshot.width();
        let height = screenshot.height();

        // Create a surface to draw on
        let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).ok()?;
        let cr = cairo::Context::new(&surface).ok()?;

        // Draw the original image
        gtk::gdk::prelude::GdkCairoContextExt::set_source_pixbuf(&cr, &screenshot.pixbuf, 0.0, 0.0);
        cr.paint().ok()?;

        // Draw annotations at original scale
        for annotation in imp.annotations.borrow().iter() {
            annotation.draw(&cr, 1.0);
        }

        drop(cr);
        surface.flush();

        // Convert to texture
        let data = surface.data().ok()?;
        let bytes = glib::Bytes::from(&*data);

        Some(gdk::MemoryTexture::new(
            width,
            height,
            gdk::MemoryFormat::B8g8r8a8Premultiplied,
            &bytes,
            (width * 4) as usize,
        ).upcast())
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let imp = self.imp();
        let screenshot = imp.screenshot.borrow();
        let screenshot = screenshot.as_ref().ok_or_else(|| anyhow::anyhow!("No screenshot loaded"))?;

        let width = screenshot.width();
        let height = screenshot.height();

        // Create a surface to draw on
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
            .map_err(|e| anyhow::anyhow!("Failed to create surface: {}", e))?;
        let cr = cairo::Context::new(&surface)
            .map_err(|e| anyhow::anyhow!("Failed to create context: {}", e))?;

        // Draw the original image
        gtk::gdk::prelude::GdkCairoContextExt::set_source_pixbuf(&cr, &screenshot.pixbuf, 0.0, 0.0);
        cr.paint().map_err(|e| anyhow::anyhow!("Failed to paint: {}", e))?;

        // Draw annotations at original scale
        for annotation in imp.annotations.borrow().iter() {
            annotation.draw(&cr, 1.0);
        }

        drop(cr);

        // Write to PNG file
        let mut file = std::fs::File::create(path)?;
        surface.write_to_png(&mut file)
            .map_err(|e| anyhow::anyhow!("Failed to write PNG: {}", e))?;

        Ok(())
    }
}
