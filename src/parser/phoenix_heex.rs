/// Represents the type of Phoenix HEEx node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeexNode {
    /// A directive like `<% %>` or `<%= %>`.
    Directive,
    /// An expression like `{ }`.
    Expression,
    /// A tag or component with a name (e.g., "div", ".form", "Component").
    Tag(String),
}

/// The metadata of a Phoenix HEEx block-level element.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeHeexBlock {
    /// The literal contents of the Phoenix block element, including delimiters.
    pub literal: String,
    /// The type of HEEx node.
    pub node: HeexNode,
}
