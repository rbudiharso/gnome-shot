use gtk::gdk::RGBA;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub enum Annotation {
    Arrow(ArrowAnnotation),
    Rectangle(RectAnnotation),
    Line(LineAnnotation),
    Ellipse(EllipseAnnotation),
    Highlight(HighlightAnnotation),
    Blur(BlurAnnotation),
}

impl Annotation {
    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        match self {
            Annotation::Arrow(a) => a.draw(cr, scale),
            Annotation::Rectangle(r) => r.draw(cr, scale),
            Annotation::Line(l) => l.draw(cr, scale),
            Annotation::Ellipse(e) => e.draw(cr, scale),
            Annotation::Highlight(h) => h.draw(cr, scale),
            Annotation::Blur(b) => b.draw(cr, scale),
        }
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        match self {
            Annotation::Arrow(a) => a.bounds(),
            Annotation::Rectangle(r) => r.bounds(),
            Annotation::Line(l) => l.bounds(),
            Annotation::Ellipse(e) => e.bounds(),
            Annotation::Highlight(h) => h.bounds(),
            Annotation::Blur(b) => b.bounds(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrowAnnotation {
    pub start: Point,
    pub end: Point,
    pub color: RGBA,
    pub stroke_width: f64,
}

impl ArrowAnnotation {
    pub fn new(start: Point, end: Point, color: RGBA, stroke_width: f64) -> Self {
        Self { start, end, color, stroke_width }
    }

    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            self.color.alpha() as f64,
        );
        cr.set_line_width(self.stroke_width * scale);
        cr.set_line_cap(cairo::LineCap::Round);

        let sx = self.start.x * scale;
        let sy = self.start.y * scale;
        let ex = self.end.x * scale;
        let ey = self.end.y * scale;

        // Draw line
        cr.move_to(sx, sy);
        cr.line_to(ex, ey);
        let _ = cr.stroke();

        // Draw arrowhead
        let angle = (ey - sy).atan2(ex - sx);
        let arrow_length = 15.0 * scale;
        let arrow_angle = std::f64::consts::PI / 6.0;

        let x1 = ex - arrow_length * (angle - arrow_angle).cos();
        let y1 = ey - arrow_length * (angle - arrow_angle).sin();
        let x2 = ex - arrow_length * (angle + arrow_angle).cos();
        let y2 = ey - arrow_length * (angle + arrow_angle).sin();

        cr.move_to(ex, ey);
        cr.line_to(x1, y1);
        cr.move_to(ex, ey);
        cr.line_to(x2, y2);
        let _ = cr.stroke();
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let x = self.start.x.min(self.end.x);
        let y = self.start.y.min(self.end.y);
        let w = (self.end.x - self.start.x).abs();
        let h = (self.end.y - self.start.y).abs();
        (x, y, w, h)
    }
}

#[derive(Debug, Clone)]
pub struct RectAnnotation {
    pub start: Point,
    pub end: Point,
    pub color: RGBA,
    pub stroke_width: f64,
    pub filled: bool,
}

impl RectAnnotation {
    pub fn new(start: Point, end: Point, color: RGBA, stroke_width: f64, filled: bool) -> Self {
        Self { start, end, color, stroke_width, filled }
    }

    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            self.color.alpha() as f64,
        );
        cr.set_line_width(self.stroke_width * scale);

        let x = self.start.x.min(self.end.x) * scale;
        let y = self.start.y.min(self.end.y) * scale;
        let w = (self.end.x - self.start.x).abs() * scale;
        let h = (self.end.y - self.start.y).abs() * scale;

        cr.rectangle(x, y, w, h);

        if self.filled {
            let _ = cr.fill();
        } else {
            let _ = cr.stroke();
        }
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let x = self.start.x.min(self.end.x);
        let y = self.start.y.min(self.end.y);
        let w = (self.end.x - self.start.x).abs();
        let h = (self.end.y - self.start.y).abs();
        (x, y, w, h)
    }
}

#[derive(Debug, Clone)]
pub struct LineAnnotation {
    pub start: Point,
    pub end: Point,
    pub color: RGBA,
    pub stroke_width: f64,
}

impl LineAnnotation {
    pub fn new(start: Point, end: Point, color: RGBA, stroke_width: f64) -> Self {
        Self { start, end, color, stroke_width }
    }

    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            self.color.alpha() as f64,
        );
        cr.set_line_width(self.stroke_width * scale);
        cr.set_line_cap(cairo::LineCap::Round);

        cr.move_to(self.start.x * scale, self.start.y * scale);
        cr.line_to(self.end.x * scale, self.end.y * scale);
        let _ = cr.stroke();
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let x = self.start.x.min(self.end.x);
        let y = self.start.y.min(self.end.y);
        let w = (self.end.x - self.start.x).abs();
        let h = (self.end.y - self.start.y).abs();
        (x, y, w, h)
    }
}

#[derive(Debug, Clone)]
pub struct EllipseAnnotation {
    pub start: Point,
    pub end: Point,
    pub color: RGBA,
    pub stroke_width: f64,
    pub filled: bool,
}

impl EllipseAnnotation {
    pub fn new(start: Point, end: Point, color: RGBA, stroke_width: f64, filled: bool) -> Self {
        Self { start, end, color, stroke_width, filled }
    }

    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            self.color.alpha() as f64,
        );
        cr.set_line_width(self.stroke_width * scale);

        let x = self.start.x.min(self.end.x) * scale;
        let y = self.start.y.min(self.end.y) * scale;
        let w = (self.end.x - self.start.x).abs() * scale;
        let h = (self.end.y - self.start.y).abs() * scale;

        if w > 0.0 && h > 0.0 {
            let cx = x + w / 2.0;
            let cy = y + h / 2.0;
            let rx = w / 2.0;
            let ry = h / 2.0;

            cr.save().unwrap();
            cr.translate(cx, cy);
            cr.scale(rx, ry);
            cr.arc(0.0, 0.0, 1.0, 0.0, 2.0 * std::f64::consts::PI);
            cr.restore().unwrap();

            if self.filled {
                let _ = cr.fill();
            } else {
                let _ = cr.stroke();
            }
        }
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let x = self.start.x.min(self.end.x);
        let y = self.start.y.min(self.end.y);
        let w = (self.end.x - self.start.x).abs();
        let h = (self.end.y - self.start.y).abs();
        (x, y, w, h)
    }
}

#[derive(Debug, Clone)]
pub struct HighlightAnnotation {
    pub start: Point,
    pub end: Point,
    pub color: RGBA,
}

impl HighlightAnnotation {
    pub fn new(start: Point, end: Point, color: RGBA) -> Self {
        Self { start, end, color }
    }

    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        // Semi-transparent highlight
        cr.set_source_rgba(
            self.color.red() as f64,
            self.color.green() as f64,
            self.color.blue() as f64,
            0.35, // Fixed transparency for highlight
        );

        let x = self.start.x.min(self.end.x) * scale;
        let y = self.start.y.min(self.end.y) * scale;
        let w = (self.end.x - self.start.x).abs() * scale;
        let h = (self.end.y - self.start.y).abs() * scale;

        cr.rectangle(x, y, w, h);
        let _ = cr.fill();
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let x = self.start.x.min(self.end.x);
        let y = self.start.y.min(self.end.y);
        let w = (self.end.x - self.start.x).abs();
        let h = (self.end.y - self.start.y).abs();
        (x, y, w, h)
    }
}

#[derive(Debug, Clone)]
pub struct BlurAnnotation {
    pub start: Point,
    pub end: Point,
    pub block_size: f64,
}

impl BlurAnnotation {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end, block_size: 10.0 }
    }

    pub fn draw(&self, cr: &cairo::Context, scale: f64) {
        // Draw a pixelated/mosaic pattern to obscure content
        let x = self.start.x.min(self.end.x) * scale;
        let y = self.start.y.min(self.end.y) * scale;
        let w = (self.end.x - self.start.x).abs() * scale;
        let h = (self.end.y - self.start.y).abs() * scale;

        let block = self.block_size * scale;

        // Draw checkerboard pattern to simulate pixelation
        let cols = (w / block).ceil() as i32;
        let rows = (h / block).ceil() as i32;

        for row in 0..rows {
            for col in 0..cols {
                // Alternate colors for mosaic effect
                let shade = if (row + col) % 2 == 0 { 0.3 } else { 0.5 };
                cr.set_source_rgb(shade, shade, shade);

                let bx = x + col as f64 * block;
                let by = y + row as f64 * block;
                let bw = block.min(x + w - bx);
                let bh = block.min(y + h - by);

                cr.rectangle(bx, by, bw, bh);
                let _ = cr.fill();
            }
        }
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let x = self.start.x.min(self.end.x);
        let y = self.start.y.min(self.end.y);
        let w = (self.end.x - self.start.x).abs();
        let h = (self.end.y - self.start.y).abs();
        (x, y, w, h)
    }
}
