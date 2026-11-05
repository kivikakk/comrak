use super::model::{children, Finding, Rec, Source};
use comrak::nodes::NodeValue;

fn is_break(rec: &Rec) -> bool {
    matches!(rec.value, NodeValue::SoftBreak | NodeValue::LineBreak)
}

fn is_placeholder_cell(recs: &[Rec], index: usize) -> bool {
    matches!(recs[index].value, NodeValue::TableCell) && children(recs, index).is_empty()
}

fn is_task_item_paragraph(recs: &[Rec], index: usize) -> bool {
    matches!(recs[index].value, NodeValue::Paragraph)
        && recs[index]
            .parent
            .is_some_and(|p| matches!(recs[p].value, NodeValue::TaskItem(_)))
}

pub fn is_inline_footnote_paragraph(recs: &[Rec], index: usize) -> bool {
    matches!(recs[index].value, NodeValue::Paragraph)
        && recs[index].parent.is_some_and(|p| {
            matches!(&recs[p].value, NodeValue::FootnoteDefinition(d) if d.name.starts_with("__inline_"))
        })
}

fn has_verbatim_bounds(recs: &[Rec], index: usize) -> bool {
    is_task_item_paragraph(recs, index) || is_inline_footnote_paragraph(recs, index)
}

fn is_in_wikilink(recs: &[Rec], index: usize) -> bool {
    recs[index]
        .parent
        .is_some_and(|p| matches!(recs[p].value, NodeValue::WikiLink(_)))
}

fn is_footnote_definition(rec: &Rec) -> bool {
    matches!(rec.value, NodeValue::FootnoteDefinition(_))
}

fn sibling_order(recs: &[Rec], parent: usize, out: &mut Vec<Finding>) {
    for pair in children(recs, parent).windows(2) {
        let (a, b) = (&recs[pair[0]], &recs[pair[1]]);
        if is_footnote_definition(a) || is_footnote_definition(b) {
            continue;
        }
        if is_placeholder_cell(recs, pair[1]) {
            continue;
        }
        if a.sp.end >= b.sp.start {
            out.push(Finding::new(
                "structure",
                &b.kind,
                &format!("overlaps-prev:{}", a.kind),
                format!("{} @ {} then {} @ {}", a.kind, a.sp, b.kind, b.sp),
            ));
        }
    }
}

pub fn structure(recs: &[Rec], source: &Source) -> Vec<Finding> {
    let mut out = vec![];
    sibling_order(recs, 0, &mut out);
    for (index, rec) in recs.iter().enumerate().skip(1) {
        sibling_order(recs, index, &mut out);
        if is_placeholder_cell(recs, index) {
            continue;
        }
        let sp = rec.sp;
        let kind = rec.kind.as_str();
        let detail = format!("{kind} @ {sp}");
        let mut flag = |reason: &str| {
            out.push(Finding::new("structure", kind, reason, detail.clone()));
        };
        if sp.start.line == 0 || sp.start.column == 0 || sp.end.line == 0 || sp.end.column == 0 {
            flag("zero");
            continue;
        }
        if sp.start > sp.end {
            flag("inverted");
            continue;
        }
        let (Some(start), Some(end)) = (source.offset(sp.start), source.offset(sp.end)) else {
            flag("out-of-bounds");
            continue;
        };
        if end >= source.bytes.len() {
            flag("past-eof");
            continue;
        }
        if !source.text.is_char_boundary(start) || !source.text.is_char_boundary(end + 1) {
            flag("splits-utf8");
        }
        if !is_break(rec) {
            if start >= source.content_ends[sp.start.line - 1] && !has_verbatim_bounds(recs, index)
            {
                flag("start-at-eol");
            }
            if end >= source.content_ends[sp.end.line - 1]
                && !is_in_wikilink(recs, index)
                && !is_inline_footnote_paragraph(recs, index)
            {
                flag("end-at-eol");
            }
        }
        let parent = &recs[rec.parent.unwrap()];
        if rec.parent != Some(0) && (sp.start < parent.sp.start || sp.end > parent.sp.end) {
            out.push(Finding::new(
                "structure",
                kind,
                &format!("escapes-parent:{}", parent.kind),
                format!("{kind} @ {sp} parent {} @ {}", parent.kind, parent.sp),
            ));
        }
    }
    out
}

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

fn is_cell_padding(b: u8) -> bool {
    is_ws(b) || matches!(b, 0x0b | 0x0c)
}

fn trim(mut s: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = s {
        if !is_ws(*first) {
            break;
        }
        s = rest;
    }
    while let [rest @ .., last] = s {
        if !is_ws(*last) {
            break;
        }
        s = rest;
    }
    s
}

fn first_line(s: &[u8]) -> &[u8] {
    let end = s
        .iter()
        .position(|&b| b == b'\n' || b == b'\r')
        .unwrap_or(s.len());
    &s[..end]
}

fn last_line(s: &[u8]) -> &[u8] {
    let start = s
        .iter()
        .rposition(|&b| b == b'\n' || b == b'\r')
        .map_or(0, |i| i + 1);
    &s[start..]
}

fn strip_container_prefix(s: &[u8]) -> &[u8] {
    let skip = s.iter().take_while(|&&b| is_ws(b) || b == b'>').count();
    &s[skip..]
}

fn same_last_line(slice: &[u8], literal: &[u8]) -> bool {
    trim(strip_container_prefix(last_line(slice)))
        == trim(strip_container_prefix(last_line(literal)))
}

fn delimited_by(s: &[u8], delimiter: &str) -> bool {
    let d = delimiter.as_bytes();
    s.len() >= 2 * d.len() && s.starts_with(d) && s.ends_with(d)
}

fn text_matches(slice: &[u8], literal: &[u8], fuel: &mut usize) -> bool {
    if *fuel == 0 {
        return true;
    }
    *fuel -= 1;
    let Some(&first) = slice.first() else {
        return literal.is_empty();
    };
    if literal.first() == Some(&first) && text_matches(&slice[1..], &literal[1..], fuel) {
        return true;
    }
    if first == b'\\'
        && slice.len() > 1
        && slice[1].is_ascii_punctuation()
        && literal.first() == Some(&slice[1])
        && text_matches(&slice[2..], &literal[1..], fuel)
    {
        return true;
    }
    if first == b'&' {
        if let Some(semi) = slice.iter().take(40).position(|&b| b == b';') {
            for consumed in 1..=literal.len().min(8) {
                if text_matches(&slice[semi + 1..], &literal[consumed..], fuel) {
                    return true;
                }
            }
        }
    }
    for (raw, decoded) in [
        ("\0", "\u{fffd}"),
        ("\"", "\u{201c}"),
        ("\"", "\u{201d}"),
        ("'", "\u{2018}"),
        ("'", "\u{2019}"),
        ("---", "\u{2014}"),
        ("--", "\u{2013}"),
        ("...", "\u{2026}"),
        ("<<", "\u{ab}"),
        (">>", "\u{bb}"),
    ] {
        if slice.starts_with(raw.as_bytes())
            && literal.starts_with(decoded.as_bytes())
            && text_matches(&slice[raw.len()..], &literal[decoded.len()..], fuel)
        {
            return true;
        }
    }
    false
}

fn kind_problem(recs: &[Rec], index: usize, s: &[u8]) -> Option<&'static str> {
    let rec = &recs[index];
    let first = s[0];
    let bad = |condition: bool, reason: &'static str| condition.then_some(reason);
    match &rec.value {
        NodeValue::BlockQuote | NodeValue::Alert(_) | NodeValue::Callout(_) => {
            bad(first != b'>', "no-gt")
        }
        NodeValue::MultilineBlockQuote(_) => bad(!s.starts_with(b">>>"), "no-fence"),
        NodeValue::List(_) | NodeValue::Item(_) => bad(
            !matches!(first, b'-' | b'+' | b'*' | b'0'..=b'9'),
            "no-marker",
        ),
        NodeValue::TaskItem(_) => bad(
            !matches!(first, b'-' | b'+' | b'*' | b'[' | b'0'..=b'9'),
            "no-marker",
        ),
        NodeValue::DescriptionDetails => bad(!matches!(first, b':' | b'~'), "no-marker"),
        NodeValue::CodeBlock(c) if c.fenced => {
            let fence = vec![c.fence_char; c.fence_length];
            let close = trim(strip_container_prefix(last_line(s)));
            bad(!s.starts_with(&fence), "no-open-fence").or(bad(
                c.closed
                    && !(close.len() >= c.fence_length && close.iter().all(|&b| b == c.fence_char)),
                "no-close-fence",
            ))
        }
        NodeValue::CodeBlock(c) => {
            let literal = c.literal.as_bytes();
            bad(
                trim(first_line(s)) != trim(first_line(literal)),
                "indented-first-line",
            )
            .or(bad(!same_last_line(s, trim(literal)), "indented-last-line"))
        }
        NodeValue::HtmlBlock(h) => {
            let literal = h.literal.as_bytes();
            bad(
                trim(first_line(s)) != trim(first_line(literal)),
                "first-line",
            )
            .or(bad(!same_last_line(s, trim(literal)), "last-line"))
        }
        NodeValue::Paragraph => bad(
            is_ws(first) && !has_verbatim_bounds(recs, index),
            "leading-ws",
        ),
        NodeValue::Heading(h) if h.setext => {
            let underline = trim(strip_container_prefix(last_line(s)));
            bad(
                !(underline.iter().all(|&b| b == b'=') || underline.iter().all(|&b| b == b'-')),
                "setext-no-underline",
            )
        }
        NodeValue::Heading(h) => bad(!s.starts_with(&vec![b'#'; h.level as usize]), "atx-no-hash"),
        NodeValue::ThematicBreak => bad(
            is_ws(first)
                || !s
                    .iter()
                    .all(|&b| matches!(b, b'-' | b'_' | b'*') || is_ws(b)),
            "junk",
        ),
        NodeValue::FootnoteDefinition(_) => {
            bad(!(s.starts_with(b"[^") || s.starts_with(b"^[")), "no-open")
        }
        NodeValue::Table(_) | NodeValue::TableRow(_) => bad(is_ws(first), "leading-ws"),
        NodeValue::Text(_) if is_in_wikilink(recs, index) => None,
        NodeValue::Text(t) => bad(rec.sp.start.line != rec.sp.end.line, "multiline")
            .or(bad(!text_matches(s, t.as_bytes(), &mut 20000), "mismatch")),
        NodeValue::SoftBreak => bad(!s.iter().all(|&b| is_ws(b)), "non-ws")
            .or(bad(!matches!(s.last(), Some(b'\n' | b'\r')), "no-eol")),
        NodeValue::LineBreak => {
            let body_len = s.iter().rposition(|&b| b != b'\n' && b != b'\r');
            let body = body_len.map_or(&s[..0], |i| &s[..=i]);
            bad(
                !(body == b"\\" || (body.len() >= 2 && body.iter().all(|&b| b == b' '))),
                "not-break",
            )
            .or(bad(!matches!(s.last(), Some(b'\n' | b'\r')), "no-eol"))
        }
        NodeValue::Code(_) => {
            let ticks = s.iter().take_while(|&&b| b == b'`').count();
            bad(ticks == 0, "no-open").or(bad(
                !rec.has_attrs && !trim(s).ends_with(&vec![b'`'; ticks]),
                "no-close",
            ))
        }
        NodeValue::HtmlInline(literal) => {
            let literal = literal.as_bytes();
            bad(
                trim(first_line(s)) != trim(first_line(literal)),
                "first-line",
            )
            .or(bad(!same_last_line(s, literal), "last-line"))
        }
        NodeValue::Emph => bad(!(delimited_by(s, "*") || delimited_by(s, "_")), "delim"),
        NodeValue::Strong => bad(!(delimited_by(s, "**") || delimited_by(s, "__")), "delim"),
        NodeValue::Strikethrough | NodeValue::Subscript => bad(!delimited_by(s, "~"), "delim"),
        NodeValue::Highlight => bad(!delimited_by(s, "=="), "delim"),
        NodeValue::Insert => bad(!delimited_by(s, "++"), "delim"),
        NodeValue::Superscript => bad(!delimited_by(s, "^"), "delim"),
        NodeValue::Underline => bad(!delimited_by(s, "__"), "delim"),
        NodeValue::SpoileredText => bad(!delimited_by(s, "||"), "delim"),
        NodeValue::EscapedTag(tag) => bad(!delimited_by(s, tag), "delim"),
        NodeValue::Link(_) if first == b'[' => bad(
            !(rec.has_attrs || s.ends_with(b"]") || s.ends_with(b")")),
            "bracket-close",
        ),
        NodeValue::Link(_) if first == b'<' => bad(!s.ends_with(b">"), "angle-close"),
        NodeValue::Link(link) => {
            let raw = String::from_utf8_lossy(s).replace("\\|", "|");
            bad(
                !(link.url.contains(&raw) || s.contains(&b'&')),
                "autolink-mismatch",
            )
        }
        NodeValue::Image(_) => bad(!s.starts_with(b"!["), "no-open").or(bad(
            !(rec.has_attrs || s.ends_with(b"]") || s.ends_with(b")")),
            "no-close",
        )),
        NodeValue::FootnoteReference(_) => bad(
            !(s.starts_with(b"[^") || s.starts_with(b"^[")) || !s.ends_with(b"]"),
            "delim",
        ),
        NodeValue::ShortCode(_) => bad(!delimited_by(s, ":"), "delim"),
        NodeValue::Math(_) => bad(!(first == b'$' || first == b'\\'), "no-open").or(bad(
            !(s.ends_with(b"$") || s.ends_with(b")") || s.ends_with(b"]")),
            "no-close",
        )),
        NodeValue::Escaped => bad(!(s.len() >= 2 && first == b'\\'), "no-backslash"),
        NodeValue::WikiLink(link) => {
            let open: &[u8] = if link.embed { b"![[" } else { b"[[" };
            bad(!s.starts_with(open) || !s.ends_with(b"]]"), "delim")
        }
        NodeValue::Subtext => bad(!s.starts_with(b"-#"), "no-open"),
        NodeValue::BlockDirective(_) => bad(!s.starts_with(b":::"), "no-open"),
        NodeValue::BlockId(_) => bad(first != b'^', "no-open"),
        NodeValue::CommentInline(_) => bad(!delimited_by(s, "%%"), "delim"),
        NodeValue::MultilineComment(_) => bad(!s.starts_with(b"%%"), "no-open"),
        _ => None,
    }
}

pub fn kind(recs: &[Rec], source: &Source) -> Vec<Finding> {
    let mut out = vec![];
    for (index, rec) in recs.iter().enumerate().skip(1) {
        let Some(s) = source.slice(rec.sp) else {
            continue;
        };
        if let Some(reason) = kind_problem(recs, index, s) {
            out.push(Finding::new(
                "kind",
                &rec.kind,
                reason,
                format!(
                    "{} @ {} slice={:?}",
                    rec.fingerprint.chars().take(80).collect::<String>(),
                    rec.sp,
                    String::from_utf8_lossy(s)
                ),
            ));
        }
        if let (Some(symbol_sp), NodeValue::TaskItem(item)) = (rec.symbol_sp, &rec.value) {
            let mut buf = [0; 4];
            let symbol = item.symbol.unwrap().encode_utf8(&mut buf).as_bytes();
            if source.slice(symbol_sp) != Some(symbol) {
                out.push(Finding::new(
                    "kind",
                    &rec.kind,
                    "symbol-mismatch",
                    format!("{} @ {symbol_sp}", rec.fingerprint),
                ));
            }
        }
    }
    out
}

fn is_inline_container(rec: &Rec) -> bool {
    matches!(
        rec.value,
        NodeValue::Paragraph
            | NodeValue::Heading(_)
            | NodeValue::TableCell
            | NodeValue::Subtext
            | NodeValue::Emph
            | NodeValue::Strong
            | NodeValue::Strikethrough
            | NodeValue::Highlight
            | NodeValue::Insert
            | NodeValue::Superscript
            | NodeValue::Subscript
            | NodeValue::Underline
            | NodeValue::SpoileredText
            | NodeValue::EscapedTag(_)
            | NodeValue::Link(_)
            | NodeValue::Image(_)
            | NodeValue::Escaped
    )
}

fn without_backslashes(delimiter: &[u8]) -> Vec<u8> {
    delimiter.iter().copied().filter(|&b| b != b'\\').collect()
}

fn open_fits(rec: &Rec, open: &[u8]) -> bool {
    match &rec.value {
        NodeValue::Paragraph => open.is_empty(),
        NodeValue::Heading(h) if h.setext => open.is_empty(),
        NodeValue::Heading(h) => {
            let level = h.level as usize;
            open.len() > level
                && open[..level].iter().all(|&b| b == b'#')
                && open[level..].iter().all(|&b| is_ws(b))
        }
        NodeValue::TableCell => open.iter().all(|&b| is_cell_padding(b)),
        NodeValue::Subtext => open.starts_with(b"-#"),
        NodeValue::Emph => open == b"*" || open == b"_",
        NodeValue::Strong => open == b"**" || open == b"__",
        NodeValue::Underline => open == b"__",
        NodeValue::Strikethrough => open == b"~" || open == b"~~",
        NodeValue::Subscript => open == b"~",
        NodeValue::Highlight => open == b"==",
        NodeValue::Insert => open == b"++",
        NodeValue::Superscript => open.iter().all(|&b| b == b'^'),
        NodeValue::SpoileredText => without_backslashes(open) == b"||",
        NodeValue::EscapedTag(tag) => without_backslashes(open) == tag.as_bytes(),
        NodeValue::Link(_) => open == b"[" || open == b"<" || open.is_empty(),
        NodeValue::Image(_) => open == b"![",
        NodeValue::Escaped => open == b"\\",
        _ => true,
    }
}

fn close_fits(rec: &Rec, close: &[u8]) -> bool {
    match &rec.value {
        NodeValue::Paragraph | NodeValue::Subtext => close.iter().all(|&b| is_ws(b)),
        NodeValue::TableCell => close.iter().all(|&b| is_cell_padding(b)),
        NodeValue::Heading(h) if h.setext => strip_container_prefix(close)
            .iter()
            .all(|&b| b == b'=' || b == b'-' || is_ws(b)),
        NodeValue::Heading(_) => close.iter().all(|&b| b == b'#' || is_ws(b)),
        NodeValue::Emph => close == b"*" || close == b"_",
        NodeValue::Strong => close == b"**" || close == b"__",
        NodeValue::Underline => close == b"__",
        NodeValue::Strikethrough => close == b"~" || close == b"~~",
        NodeValue::Subscript => close == b"~",
        NodeValue::Highlight => close == b"==",
        NodeValue::Insert => close == b"++",
        NodeValue::Superscript => close.iter().all(|&b| b == b'^'),
        NodeValue::SpoileredText => without_backslashes(close) == b"||",
        NodeValue::EscapedTag(tag) => without_backslashes(close) == tag.as_bytes(),
        NodeValue::Link(_) => {
            strip_container_prefix(close).starts_with(b"]") || close == b">" || close.is_empty()
        }
        NodeValue::Image(_) => strip_container_prefix(close).starts_with(b"]"),
        NodeValue::Escaped => close.is_empty(),
        _ => true,
    }
}

pub fn tiling(recs: &[Rec], source: &Source) -> Vec<Finding> {
    let mut out = vec![];
    for (index, parent) in recs.iter().enumerate().skip(1) {
        if !is_inline_container(parent) {
            continue;
        }
        let kids = children(recs, index);
        let (Some(&first_kid), Some((parent_start, parent_end))) =
            (kids.first(), source.span(parent.sp))
        else {
            continue;
        };
        let kind = parent.kind.as_str();
        for pair in kids.windows(2) {
            let (a, b) = (&recs[pair[0]], &recs[pair[1]]);
            let (Some((_, a_end)), Some((b_start, _))) = (source.span(a.sp), source.span(b.sp))
            else {
                continue;
            };
            let tag = format!("{}->{}", a.kind, b.kind);
            let detail = format!(
                "in {kind}: {} @ {} then {} @ {}",
                a.kind, a.sp, b.kind, b.sp
            );
            let reason = if is_break(a) {
                let line_start = source.line_starts[b.sp.start.line - 1];
                let lead = &source.bytes[line_start..b_start.max(line_start)];
                let lead = lead.strip_suffix(b"\\").unwrap_or(lead);
                if b.sp.start.line != a.sp.end.line + 1 {
                    Some("break-line")
                } else if !lead.iter().all(|&c| is_ws(c) || c == b'>') {
                    Some("after-break")
                } else {
                    None
                }
            } else if b_start <= a_end {
                Some("overlap")
            } else {
                let gap = &source.bytes[a_end + 1..b_start];
                let acceptable = if is_break(b) {
                    gap.iter().all(|&c| is_ws(c))
                } else {
                    gap.is_empty() || gap == b"\\"
                };
                (!acceptable).then_some("gap")
            };
            if let Some(reason) = reason {
                out.push(Finding::new(
                    "tiling",
                    kind,
                    &format!("{reason}:{tag}"),
                    detail,
                ));
            }
        }
        let (first, last) = (&recs[first_kid], &recs[*kids.last().unwrap()]);
        let (Some((first_start, _)), Some((_, last_end))) =
            (source.span(first.sp), source.span(last.sp))
        else {
            continue;
        };
        if first_start < parent_start || last_end > parent_end {
            continue;
        }
        let open = &source.bytes[parent_start..first_start];
        let close = &source.bytes[last_end + 1..=parent_end];
        let unescaped_open = if matches!(parent.value, NodeValue::Escaped) {
            open
        } else {
            open.strip_suffix(b"\\").unwrap_or(open)
        };
        let detail = format!(
            "{kind} @ {} open={:?} close={:?}",
            parent.sp,
            String::from_utf8_lossy(open),
            String::from_utf8_lossy(close),
        );
        if !open_fits(parent, unescaped_open) && !is_task_item_paragraph(recs, index) {
            out.push(Finding::new("tiling", kind, "open", detail.clone()));
        }
        if !parent.has_attrs && !close_fits(parent, close) {
            out.push(Finding::new("tiling", kind, "close", detail));
        }
    }
    out
}

pub fn document_extent(recs: &[Rec], source: &Source) -> Vec<Finding> {
    if source.bytes.is_empty() {
        return vec![];
    }
    let sp = recs[0].sp;
    let last = source.line_count();
    let last_len = source.line_content(last - 1).len();
    if sp.start != (1, 1).into() {
        return vec![Finding::new(
            "document",
            "Document",
            "start",
            sp.to_string(),
        )];
    }
    if sp.end != (last, last_len.max(1)).into() && sp.end != (last, last_len).into() {
        return vec![Finding::new(
            "document",
            "Document",
            "end",
            format!("{sp}, wanted end {last}:{last_len}"),
        )];
    }
    vec![]
}
