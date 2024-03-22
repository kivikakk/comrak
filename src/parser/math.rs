// use crate::nodes::{Ast, AstNode, NodeValue};
// use crate::parser::Parser;
// use crate::scanners;
// use crate::strings;

/// An inline math span
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// A math block using `$$`
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct NodeMathBlock {
    /// The indentation level of the math within the block.
    pub fence_offset: usize,

    /// The literal contents of the math block.
    /// As the contents are not interpreted as Markdown at all,
    /// they are contained within this structure,
    /// rather than inserted into a child block of any kind.
    pub literal: String,
}
