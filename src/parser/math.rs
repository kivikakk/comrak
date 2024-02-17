// use crate::nodes::{Ast, AstNode, NodeValue};
// use crate::parser::Parser;
// use crate::scanners;
// use crate::strings;

/// An inline math span
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMath {
    /// Whether this is dollar math (`$` or `$$`)
    pub dollar_math: bool,

    /// Whether this is display math (using `$$`)
    pub display_math: bool,

    /// The literal contents of the math span.    
    /// As the contents are not interpreted as Markdown at all,
    /// they are contained within this structure,
    /// rather than inserted into a child inline of any kind.
    pub literal: String,
}

/// The metadata and data of a math block.
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

// TODO: This is an attempt to gather together the "math" related methods into one place.
// Unfortunately ran into compile errors with "container" regarding lifetimes.
// pub fn parse_math_block_prefix(
//     parser: &mut Parser,
//     line: &[u8],
//     container: &'a AstNode<'a>,
//     ast: &mut Ast,
//     should_continue: &mut bool,
// ) -> bool {
//     let (fence_char, fence_length, fence_offset) = match ast.value {
//         NodeValue::MathBlock(ref nmb) => (
//             nmb.fence_char,
//             nmb.fence_length,
//             nmb.fence_offset,
//         ),
//         _ => unreachable!(),
//     };
//
//     let matched = if parser.indent <= 3 && line[parser.first_nonspace] == fence_char {
//         scanners::close_math_fence(&line[parser.first_nonspace..]).unwrap_or(0)
//     } else {
//         0
//     };
//
//     if matched >= fence_length {
//         *should_continue = false;
//         parser.advance_offset(line, matched, false);
//         parser.current = parser.finalize_borrowed(container, ast).unwrap();
//         return false;
//     }
//
//     let mut i = fence_offset;
//     while i > 0 && strings::is_space_or_tab(line[parser.offset]) {
//         parser.advance_offset(line, 1, true);
//         i -= 1;
//     }
//     true
// }
