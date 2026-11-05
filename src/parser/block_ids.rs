//! Recognition of Obsidian block identifiers (`^abc123`).
//!
//! Identifiers are picked out of a block's raw content before its inlines are
//! parsed, so the tokenizer goes on treating the caret exactly as it would have
//! without this extension.

use crate::Arena;
use crate::nodes::{Ast, Node, NodeBlockId, NodeValue, Sourcepos};
use crate::strings;

/// An identifier ending a block's raw content, with where it sat in that content.
pub(crate) struct TrailingMarker {
    pub name: String,

    /// Which line of the content the marker is on, counting from 0.
    pub line: usize,

    /// The 1-based column of the caret within that line.
    pub column: usize,
}

/// Whether spaces or tabs hang off the end of a block's raw content, not counting
/// the newline that ends its last line.  Obsidian won't recognise a marker with
/// whitespace after it, so `hello ^id  ` is just text.
pub(crate) fn ends_with_whitespace(content: &str) -> bool {
    content
        .trim_end_matches(['\r', '\n'])
        .ends_with([' ', '\t'])
}

pub(crate) fn is_id_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-'
}

/// The identifier ending a block's raw content, if it ends with one, which is
/// also cut off `content` so that it doesn't end up in the block's text.
pub(crate) fn take_trailing_marker(content: &mut String) -> Option<TrailingMarker> {
    let end = content.trim_end_matches(['\r', '\n']).len();
    let start = content[..end].rfind(['\r', '\n']).map_or(0, |i| i + 1);
    let bytes = content.as_bytes();

    let mut name_start = end;
    while name_start > start && is_id_byte(bytes[name_start - 1]) {
        name_start -= 1;
    }

    let caret = name_start.checked_sub(1).filter(|_| name_start < end)?;
    if caret <= start
        || bytes[caret] != b'^'
        || !content[..caret]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_whitespace() || c == '\u{feff}')
    {
        return None;
    }

    let marker = TrailingMarker {
        name: content[name_start..end].to_string(),
        line: strings::count_newlines(&content[..start]).0,
        column: caret - start + 1,
    };
    content.truncate(caret);
    Some(marker)
}

/// A [`NodeValue::BlockId`] node for `name`.
pub(crate) fn marker_node<'a>(arena: &'a Arena<'a>, name: &str, sourcepos: Sourcepos) -> Node<'a> {
    arena.alloc(
        Ast::new_with_sourcepos(
            NodeValue::BlockId(Box::new(NodeBlockId {
                name: name.to_string(),
            })),
            sourcepos,
        )
        .into(),
    )
}

/// The offset of the caret in a literal ending with ` ^name`, if it ends that
/// way.  Used when writing CommonMark back out, where the whitespace that kept
/// the identifier from being recognised won't be there any more.
pub(crate) fn trailing_marker_caret(literal: &str) -> Option<usize> {
    let bytes = literal.as_bytes();
    let mut start = bytes.len();

    while start > 0 && is_id_byte(bytes[start - 1]) {
        start -= 1;
    }

    let caret = start.checked_sub(1)?;
    if start == bytes.len() || bytes[caret] != b'^' {
        return None;
    }

    literal[..caret]
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
        .then_some(caret)
}
