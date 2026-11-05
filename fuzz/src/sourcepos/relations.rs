use super::invariants::is_inline_footnote_paragraph;
use super::model::{children, hash, subtree_end, Finding, Rec, Source};
use super::options::TOGGLES;
use comrak::nodes::{LineColumn, NodeValue, Sourcepos};
use comrak::Options;
use fastrand::Rng;

pub const FAMILIES: &[&str] = &[
    "crlf",
    "cr",
    "final-newline",
    "extra-newlines",
    "blockquote",
    "blockquote-tight",
    "blockquote-nested",
    "bullet-item",
    "ordered-item",
    "blockquote-item",
    "multiline-blockquote",
    "directive",
    "footnote-definition",
    "prepend-paragraph",
    "prepend-blank",
    "append-paragraph",
    "line-indent",
    "trailing-space",
    "insert-letter",
    "two-byte-letter",
    "three-byte-letter",
    "nul",
    "tab-for-spaces",
    "heading",
    "table-header-cell",
    "table-body-cell",
    "option-toggle",
];

#[derive(Clone)]
struct Edit {
    offset: usize,
    delete: usize,
    insert: String,
}

fn insert(offset: usize, text: &str) -> Edit {
    Edit {
        offset,
        delete: 0,
        insert: text.to_string(),
    }
}

fn replace(offset: usize, delete: usize, text: &str) -> Edit {
    Edit {
        offset,
        delete,
        insert: text.to_string(),
    }
}

fn apply(doc: &str, edits: &[Edit]) -> String {
    let mut out = String::with_capacity(doc.len() + 16);
    let mut from = 0;
    for edit in edits {
        out.push_str(&doc[from..edit.offset]);
        out.push_str(&edit.insert);
        from = edit.offset + edit.delete;
    }
    out.push_str(&doc[from..]);
    out
}

fn map_start(edits: &[Edit], offset: usize) -> usize {
    let mut delta = 0;
    for edit in edits.iter().take_while(|e| e.offset <= offset) {
        if offset < edit.offset + edit.delete {
            return (edit.offset as isize + delta) as usize;
        }
        delta += edit.insert.len() as isize - edit.delete as isize;
    }
    (offset as isize + delta) as usize
}

fn map_end(edits: &[Edit], offset: usize) -> usize {
    let mut delta = 0;
    for edit in edits.iter().take_while(|e| e.offset <= offset) {
        if offset < edit.offset + edit.delete {
            return (edit.offset as isize + delta) as usize + edit.insert.len().max(1) - 1;
        }
        delta += edit.insert.len() as isize - edit.delete as isize;
    }
    (offset as isize + delta) as usize
}

fn inside_replacement(edits: &[Edit], offset: usize) -> bool {
    edits
        .iter()
        .any(|e| e.offset <= offset && offset < e.offset + e.delete)
}

#[derive(Clone, Copy)]
enum Normalise {
    Exact,
    LineEndings,
    Whitespace,
    InsertedLetter,
    Multibyte,
    Nul,
}

fn normalised(how: Normalise, fingerprint: &str) -> String {
    match how {
        Normalise::Exact => fingerprint.to_string(),
        Normalise::LineEndings => fingerprint.replace("\\r\\n", "\\n").replace("\\r", "\\n"),
        Normalise::Whitespace => fingerprint
            .replace("\\r", "")
            .replace("\\n", "")
            .replace("\\t", "")
            .replace(' ', ""),
        Normalise::InsertedLetter => fingerprint.replace('Q', ""),
        Normalise::Multibyte => fingerprint.replace('é', "e").replace('好', "a"),
        Normalise::Nul => fingerprint.replace('\u{fffd}', "x"),
    }
}

#[derive(Clone, Copy)]
enum View {
    Whole,
    Path(&'static [usize]),
    DropFirst(usize),
    DropLast(usize),
}

fn view(recs: &[Rec], view: View) -> Option<(usize, Vec<usize>)> {
    match view {
        View::Whole => Some((0, (1..recs.len()).collect())),
        View::Path(path) => {
            let mut current = 0;
            for &index in path {
                current = *children(recs, current).get(index)?;
            }
            Some((
                recs[current].depth,
                (current + 1..subtree_end(recs, current)).collect(),
            ))
        }
        View::DropFirst(count) => {
            let from = *children(recs, 0).get(count)?;
            Some((0, (from..recs.len()).collect()))
        }
        View::DropLast(count) => {
            let kids = children(recs, 0);
            let keep = kids.len().checked_sub(count)?;
            let to = kids.get(keep).copied().unwrap_or(recs.len());
            Some((0, (1..to).collect()))
        }
    }
}

struct Trial {
    edits: Vec<Edit>,
    old_view: View,
    new_view: View,
    normalise: Normalise,
    options: Option<Options<'static>>,
    same_lines: bool,
}

fn trial(edits: Vec<Edit>, new_view: View, normalise: Normalise) -> Trial {
    Trial {
        edits,
        old_view: View::Whole,
        new_view,
        normalise,
        options: None,
        same_lines: false,
    }
}

fn prefix_lines(source: &Source, first: &str, rest: &str, blank: &str) -> Vec<Edit> {
    (0..source.line_count())
        .filter_map(|line| {
            let is_blank = source
                .line_content(line)
                .iter()
                .all(|&b| b == b' ' || b == b'\t');
            let prefix = match (line, is_blank) {
                (0, _) => first,
                (_, true) => blank,
                (_, false) => rest,
            };
            (!prefix.is_empty()).then(|| insert(source.line_starts[line], prefix))
        })
        .collect()
}

fn fence_wrap(doc: &str, fence: &str) -> Vec<Edit> {
    let open = format!("{fence}\n");
    let close = if doc.ends_with(['\n', '\r']) {
        format!("{fence}\n")
    } else {
        format!("\n{fence}\n")
    };
    vec![insert(0, &open), insert(doc.len(), &close)]
}

fn substitute(doc: &str, from: u8, to: &str) -> Vec<Edit> {
    doc.bytes()
        .enumerate()
        .filter(|(_, b)| *b == from)
        .map(|(i, _)| replace(i, 1, to))
        .collect()
}

fn lone_newlines(doc: &str) -> impl Iterator<Item = usize> + '_ {
    let bytes = doc.as_bytes();
    (0..bytes.len()).filter(move |&i| bytes[i] == b'\n' && (i == 0 || bytes[i - 1] != b'\r'))
}

fn tab_for_spaces(source: &Source) -> Vec<Edit> {
    let mut edits = vec![];
    for line in 0..source.line_count() {
        let mut column = 0;
        let mut i = source.line_starts[line];
        let end = source.content_ends[line];
        while i < end {
            match source.bytes[i] {
                b' ' => {
                    let run = source.bytes[i..end]
                        .iter()
                        .take_while(|&&b| b == b' ')
                        .count();
                    let to_tab_stop = 4 - column % 4;
                    if run >= to_tab_stop && i + run < end && to_tab_stop > 1 {
                        edits.push(replace(i, to_tab_stop, "\t"));
                        column += to_tab_stop;
                        i += to_tab_stop;
                    } else {
                        column += run;
                        i += run;
                    }
                }
                b'\t' => {
                    column += 4 - column % 4;
                    i += 1;
                }
                _ => {
                    column += 1;
                    i += 1;
                }
            }
        }
    }
    edits
}

fn is_single_line(doc: &str) -> bool {
    !doc.is_empty()
        && !doc.starts_with([' ', '\t'])
        && !doc.trim_end_matches(['\n', '\r']).contains(['\n', '\r'])
}

fn trials(family: &str, doc: &str, options: &Options<'static>) -> Vec<Trial> {
    let source = Source::new(doc);
    let wrap = |first: &str, rest: &str, blank: &str, path: &'static [usize]| {
        vec![Trial {
            same_lines: doc.contains('\t') && (first.len() % 4 != 0 || rest.len() % 4 != 0),
            ..trial(
                prefix_lines(&source, first, rest, blank),
                View::Path(path),
                Normalise::Exact,
            )
        }]
    };
    let non_empty_lines =
        || (0..source.line_count().min(16)).filter(|&line| !source.line_content(line).is_empty());
    match family {
        "crlf" => {
            let edits: Vec<_> = lone_newlines(doc).map(|i| insert(i, "\r")).collect();
            vec![trial(edits, View::Whole, Normalise::LineEndings)]
        }
        "cr" => {
            let edits: Vec<_> = lone_newlines(doc).map(|i| replace(i, 1, "\r")).collect();
            vec![trial(edits, View::Whole, Normalise::LineEndings)]
        }
        "final-newline" => {
            let edit = match doc.strip_suffix('\n') {
                Some(stripped) if !stripped.ends_with(['\n', '\r']) => {
                    replace(doc.len() - 1, 1, "")
                }
                Some(_) => return vec![],
                None if doc.ends_with('\r') => return vec![],
                None => insert(doc.len(), "\n"),
            };
            vec![trial(vec![edit], View::Whole, Normalise::Whitespace)]
        }
        "extra-newlines" => vec![trial(
            vec![insert(doc.len(), "\n\n")],
            View::Whole,
            Normalise::Whitespace,
        )],
        "blockquote" => wrap("> ", "> ", "> ", &[0]),
        "blockquote-tight" => wrap("> ", "> ", ">", &[0]),
        "blockquote-nested" => wrap("> > ", "> > ", "> > ", &[0, 0]),
        "bullet-item" => wrap("- ", "  ", "", &[0, 0]),
        "ordered-item" => wrap("1. ", "   ", "", &[0, 0]),
        "blockquote-item" => wrap("> - ", ">   ", ">   ", &[0, 0, 0]),
        "multiline-blockquote" if options.extension.multiline_block_quotes => vec![trial(
            fence_wrap(doc, ">>>"),
            View::Path(&[0]),
            Normalise::Exact,
        )],
        "directive" if options.extension.block_directive => vec![trial(
            fence_wrap(doc, ":::"),
            View::Path(&[0]),
            Normalise::Exact,
        )],
        "footnote-definition"
            if options.extension.footnotes && options.parse.leave_footnote_definitions =>
        {
            wrap("[^a]: ", "    ", "", &[0])
        }
        "multiline-blockquote" | "directive" | "footnote-definition" => vec![],
        "prepend-paragraph" => vec![trial(
            vec![insert(0, "x\n\n")],
            View::DropFirst(1),
            Normalise::Exact,
        )],
        "prepend-blank" => vec![trial(
            vec![insert(0, "\n\n")],
            View::Whole,
            Normalise::Exact,
        )],
        "append-paragraph" => {
            let tail = if doc.ends_with(['\n', '\r']) {
                "\nz\n"
            } else {
                "\n\nz\n"
            };
            vec![trial(
                vec![insert(doc.len(), tail)],
                View::DropLast(1),
                Normalise::Whitespace,
            )]
        }
        "line-indent" => non_empty_lines()
            .map(|line| Trial {
                same_lines: true,
                ..trial(
                    vec![insert(source.line_starts[line], " ")],
                    View::Whole,
                    Normalise::Whitespace,
                )
            })
            .collect(),
        "trailing-space" => non_empty_lines()
            .map(|line| {
                trial(
                    vec![insert(source.content_ends[line], " ")],
                    View::Whole,
                    Normalise::Whitespace,
                )
            })
            .collect(),
        "insert-letter" => {
            let bytes = doc.as_bytes();
            let sites: Vec<usize> = (1..bytes.len())
                .filter(|&i| {
                    bytes[i - 1].is_ascii_alphanumeric() && bytes[i].is_ascii_alphanumeric()
                })
                .collect();
            let mut rng = Rng::with_seed(hash(doc));
            (0..sites.len().min(12))
                .map(|_| {
                    trial(
                        vec![insert(sites[rng.usize(..sites.len())], "Q")],
                        View::Whole,
                        Normalise::InsertedLetter,
                    )
                })
                .collect()
        }
        "two-byte-letter" => vec![trial(
            substitute(doc, b'e', "é"),
            View::Whole,
            Normalise::Multibyte,
        )],
        "three-byte-letter" => vec![trial(
            substitute(doc, b'a', "好"),
            View::Whole,
            Normalise::Multibyte,
        )],
        "nul" => vec![trial(
            substitute(doc, b'x', "\0"),
            View::Whole,
            Normalise::Nul,
        )],
        "tab-for-spaces" => vec![trial(
            tab_for_spaces(&source),
            View::Whole,
            Normalise::Whitespace,
        )],
        "heading" if is_single_line(doc) => vec![Trial {
            old_view: View::Path(&[0]),
            ..trial(vec![insert(0, "# ")], View::Path(&[0]), Normalise::Exact)
        }],
        "table-header-cell" | "table-body-cell"
            if options.extension.table && is_single_line(doc) =>
        {
            let body_len = doc.trim_end_matches(['\n', '\r']).len();
            let (head, tail, path): (&str, &str, &'static [usize]) =
                if family == "table-header-cell" {
                    ("| ", " |\n| - |\n", &[0, 0, 0])
                } else {
                    ("| a |\n| - |\n| ", " |\n", &[0, 1, 0])
                };
            vec![Trial {
                old_view: View::Path(&[0]),
                ..trial(
                    vec![
                        insert(0, head),
                        replace(body_len, doc.len() - body_len, tail),
                    ],
                    View::Path(path),
                    Normalise::Exact,
                )
            }]
        }
        "heading" | "table-header-cell" | "table-body-cell" => vec![],
        "option-toggle" => TOGGLES
            .iter()
            .map(|(_, apply_toggle)| {
                let mut toggled = options.clone();
                apply_toggle(&mut toggled);
                Trial {
                    options: Some(toggled),
                    same_lines: true,
                    ..trial(vec![], View::Whole, Normalise::Exact)
                }
            })
            .collect(),
        _ => unreachable!("{}", family),
    }
}

#[derive(Default)]
pub struct Outcome {
    pub tried: usize,
    pub failed: usize,
    pub findings: Vec<Finding>,
}

fn same_tree(
    family: &str,
    trial: &Trial,
    old: &[Rec],
    new: &[Rec],
    old_view: &(usize, Vec<usize>),
    new_view: &(usize, Vec<usize>),
) -> bool {
    let (old_base, old_ids) = old_view;
    let (new_base, new_ids) = new_view;
    let nothing_outside_view = match trial.new_view {
        View::Path(path)
            if !matches!(family, "table-header-cell" | "table-body-cell" | "heading") =>
        {
            new.len() == new_ids.len() + path.len() + 1
        }
        _ => true,
    };
    nothing_outside_view
        && old_ids.len() == new_ids.len()
        && old_ids.iter().zip(new_ids).all(|(&a, &b)| {
            let (x, y) = (&old[a], &new[b]);
            x.kind == y.kind
                && x.depth - old_base == y.depth - new_base
                && normalised(trial.normalise, &x.fingerprint)
                    == normalised(trial.normalise, &y.fingerprint)
                && (!trial.same_lines
                    || (x.sp.start.line == y.sp.start.line && x.sp.end.line == y.sp.end.line))
        })
}

fn is_exempt(family: &str, old: &[Rec], index: usize) -> bool {
    let rec = &old[index];
    if matches!(rec.value, NodeValue::TableCell) && children(old, index).is_empty() {
        return true;
    }
    let is_break = matches!(rec.value, NodeValue::SoftBreak | NodeValue::LineBreak);
    match family {
        "crlf" => is_break,
        "trailing-space" => is_break || rec.value.block(),
        "option-toggle" => is_inline_footnote_paragraph(old, index),
        "line-indent" | "tab-for-spaces" => {
            matches!(&rec.value, NodeValue::CodeBlock(c) if !c.fenced)
        }
        _ => false,
    }
}

fn format_position(position: LineColumn) -> String {
    format!("{}:{}", position.line, position.column)
}

pub fn run(family: &str, doc: &str, options: &Options<'static>, old: &[Rec]) -> Outcome {
    let mut outcome = Outcome::default();
    let source = Source::new(doc);
    for trial in trials(family, doc, options) {
        outcome.tried += 1;
        let edited = apply(doc, &trial.edits);
        let edited_source = Source::new(&edited);
        let new = super::model::snapshot(&edited, trial.options.as_ref().unwrap_or(options));
        let (Some(old_view), Some(new_view)) =
            (view(old, trial.old_view), view(&new, trial.new_view))
        else {
            outcome.failed += 1;
            continue;
        };
        if !same_tree(family, &trial, old, &new, &old_view, &new_view) {
            outcome.failed += 1;
            continue;
        }
        for (&a, &b) in old_view.1.iter().zip(&new_view.1) {
            if is_exempt(family, old, a) {
                continue;
            }
            let (x, y) = (&old[a], &new[b]);
            let mut compare = |old_sp: Sourcepos, new_sp: Sourcepos, label: &str| {
                let (Some(start), Some(end)) =
                    (source.offset(old_sp.start), source.offset(old_sp.end))
                else {
                    return;
                };
                if family == "crlf" && start >= source.content_ends[old_sp.start.line - 1] {
                    return;
                }
                if family == "tab-for-spaces"
                    && (inside_replacement(&trial.edits, start)
                        || inside_replacement(&trial.edits, end))
                {
                    return;
                }
                if family == "insert-letter"
                    && (trial.edits[0].offset == start || trial.edits[0].offset == end + 1)
                {
                    return;
                }
                let want_start = edited_source.position(map_start(&trial.edits, start));
                let want_end = edited_source.position(map_end(&trial.edits, end));
                let which = match (new_sp.start != want_start, new_sp.end != want_end) {
                    (false, false) => return,
                    (true, true) => "both",
                    (true, false) => "start",
                    (false, true) => "end",
                };
                let mut finding = Finding::new(
                    family,
                    &x.kind,
                    &format!("{label}{which}"),
                    format!(
                        "was {old_sp}, expected {}-{}, got {new_sp}",
                        format_position(want_start),
                        format_position(want_end),
                    ),
                );
                finding.edited = Some(edited.clone());
                outcome.findings.push(finding);
            };
            compare(x.sp, y.sp, "");
            if let (Some(old_symbol), Some(new_symbol)) = (x.symbol_sp, y.symbol_sp) {
                compare(old_symbol, new_symbol, "symbol-");
            }
        }
    }
    outcome
}

pub fn chars(doc: &str, options: &Options<'static>, old: &[Rec]) -> Vec<Finding> {
    let mut char_options = options.clone();
    char_options.parse.sourcepos_chars = true;
    let new = super::model::snapshot(doc, &char_options);
    let source = Source::new(doc);
    let column = |position: LineColumn| -> Option<usize> {
        let mut offset = source.offset(position)?;
        let line_start = source.line_starts[position.line - 1];
        while offset > line_start && !doc.is_char_boundary(offset) {
            offset -= 1;
        }
        Some(doc[line_start..offset].chars().count() + 1)
    };
    let mut out = vec![];
    for (x, y) in old.iter().zip(&new).skip(1) {
        let mut flag = |reason: &str, detail: String| {
            out.push(Finding::new("chars", &x.kind, reason, detail));
        };
        if x.sp.start.line != y.sp.start.line || x.sp.end.line != y.sp.end.line {
            flag("line-changed", format!("{} vs {}", x.sp, y.sp));
        } else if let (Some(start), Some(end)) = (column(x.sp.start), column(x.sp.end)) {
            if y.sp.start.column != start || y.sp.end.column != end {
                flag(
                    "column",
                    format!(
                        "bytes {}, chars {}, expected columns {start}-{end}",
                        x.sp, y.sp
                    ),
                );
            }
        }
    }
    out
}
