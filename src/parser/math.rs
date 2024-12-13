/// An inline math span
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct NodeMath {
    /// Whether this is dollar math (`$` or `$$`).
    /// `false` indicates it is code math
    pub dollar_math: bool,

    /// Whether this is display math (using `$$`)
    pub display_math: bool,

    /// The literal contents of the math span.
    /// As the contents are not interpreted as Markdown at all,
    /// they are contained within this structure,
    /// rather than inserted into a child inline of any kind.
    pub literal: String,
}
