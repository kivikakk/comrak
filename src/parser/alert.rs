/// The metadata of an Alert node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAlert {
    /// Type of alert
    pub alert_type: AlertType,

    /// Overridden title. If `None`, then use the default title.
    pub title: Option<String>,

    /// Originated from a multiline blockquote.
    pub multiline: bool,

    /// The length of the fence (multiline only).
    pub fence_length: usize,

    /// The indentation level of the fence marker (multiline only)
    pub fence_offset: usize,
}

/// The type of alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertType {
    /// Useful information that users should know, even when skimming content
    #[default]
    Note,

    /// Helpful advice for doing things better or more easily
    Tip,

    /// Key information users need to know to achieve their goal
    Important,

    /// Urgent info that needs immediate user attention to avoid problems
    Warning,

    /// Advises about risks or negative outcomes of certain actions
    Caution,
}

impl AlertType {
    /// Returns the default title for an alert type
    pub fn default_title(&self) -> String {
        match *self {
            AlertType::Note => String::from("Note"),
            AlertType::Tip => String::from("Tip"),
            AlertType::Important => String::from("Important"),
            AlertType::Warning => String::from("Warning"),
            AlertType::Caution => String::from("Caution"),
        }
    }

    /// Returns the CSS class to use for an alert type
    pub fn css_class(&self) -> String {
        match *self {
            AlertType::Note => String::from("markdown-alert-note"),
            AlertType::Tip => String::from("markdown-alert-tip"),
            AlertType::Important => String::from("markdown-alert-important"),
            AlertType::Warning => String::from("markdown-alert-warning"),
            AlertType::Caution => String::from("markdown-alert-caution"),
        }
    }
}
