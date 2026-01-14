#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Arrow,
    Rectangle,
    Line,
    Ellipse,
    Highlight,
    Blur,
}

impl Tool {
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Arrow => "Arrow",
            Tool::Rectangle => "Rectangle",
            Tool::Line => "Line",
            Tool::Ellipse => "Ellipse",
            Tool::Highlight => "Highlight",
            Tool::Blur => "Blur",
        }
    }
}
