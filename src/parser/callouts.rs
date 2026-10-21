//! Helpers for recognizing and classifying Obsidian-style callout headers.

use crate::nodes::CalloutFold;

/// A recognized callout header line.
///
/// All offsets are relative to the input string, zero-based, and measured in
/// UTF-8 bytes.
pub(super) struct CalloutHeader<'a> {
    /// The raw type, as it appears in the input (before normalization).
    pub(super) raw_type: &'a str,

    /// The raw metadata after the first `|`, if any.
    pub(super) metadata: Option<&'a str>,

    /// The fold marker.
    pub(super) fold: CalloutFold,

    /// Start of the title, immediately after the consumed separating
    /// whitespace character.
    pub(super) title_start: usize,
}

/// Attempt to parse `input` as a callout header.
///
/// `input` is the first line at the current quote depth: the `>` of that
/// depth (plus up to one following ASCII space) has already been removed,
/// as have the physical line ending CR/LF. All other whitespace, backslashes,
/// entities and markdown markup are preserved.
///
/// On success, returns the parsed header; on failure, returns `None`.
pub(super) fn parse_callout_header<'a>(input: &'a str) -> Option<CalloutHeader<'a>> {
    let rest = input.strip_prefix("[!")?;
    let close = rest.find(']')?;
    let payload = &rest[..close];
    if payload.is_empty() {
        return None;
    }

    let mut after_close = &rest[close + 1..];
    let fold = match after_close.chars().next() {
        Some('+') => {
            after_close = &after_close[1..];
            CalloutFold::Expanded
        }
        Some('-') => {
            after_close = &after_close[1..];
            CalloutFold::Collapsed
        }
        _ => CalloutFold::None,
    };

    let title_start = match after_close.chars().next() {
        None => input.len(),
        Some(c) if c.is_whitespace() => input.len() - after_close[c.len_utf8()..].len(),
        Some(_) => return None,
    };

    let (raw_type, metadata) = match payload.split_once('|') {
        Some((t, m)) => (t, Some(m)),
        None => (payload, None),
    };

    Some(CalloutHeader {
        raw_type,
        metadata,
        fold,
        title_start,
    })
}

/// Normalize a raw callout type.
///
/// Trims Rust whitespace from both ends, lowercases the whole string, then
/// collapses runs of Rust whitespace into single ASCII hyphens. Existing
/// hyphens, underscores, punctuation and Unicode characters are left alone.
pub(super) fn normalize_callout_type(raw_type: &str) -> String {
    raw_type
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}
