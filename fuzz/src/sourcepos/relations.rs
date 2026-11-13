use super::invariants::{is_inline_footnote_paragraph, is_placeholder};
use super::model::{children, hash, subtree_end, Finding, Rec, Source};
use super::options::TOGGLES;
use comrak::nodes::{LineColumn, NodeValue, Sourcepos};
use comrak::Options;
use fastrand::Rng;
use unicode_width::UnicodeWidthChar;

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
    "table-pipe",
    "table-preface",
    "grid-cell",
    "grid-second-cell",
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
    Pipe,
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
        Normalise::Pipe => fingerprint.replace('|', ""),
    }
}

#[derive(Clone, Copy)]
enum View {
    Whole,
    Path(&'static [usize]),
    DropFirst(usize),
    DropLast(usize),
    LastParagraph,
}

fn subtree(recs: &[Rec], index: usize, base: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
    (index..subtree_end(recs, index)).map(move |i| (i, recs[i].depth - base))
}

fn with_root_definitions(
    recs: &[Rec],
    after: usize,
    mut viewed: Vec<(usize, usize)>,
) -> Vec<(usize, usize)> {
    for kid in children(recs, 0) {
        if kid > after && matches!(recs[kid].value, NodeValue::FootnoteDefinition(_)) {
            viewed.extend(subtree(recs, kid, 0));
        }
    }
    viewed
}

fn view(recs: &[Rec], view: View) -> Option<Vec<(usize, usize)>> {
    match view {
        View::Whole => Some(subtree(recs, 0, 0).skip(1).collect()),
        View::Path(path) => {
            let mut current = 0;
            for &index in path {
                current = *children(recs, current).get(index)?;
            }
            let viewed = subtree(recs, current, recs[current].depth)
                .skip(1)
                .collect();
            Some(with_root_definitions(recs, current, viewed))
        }
        View::DropFirst(count) => {
            let from = *children(recs, 0).get(count)?;
            Some((from..recs.len()).map(|i| (i, recs[i].depth)).collect())
        }
        View::DropLast(count) => {
            let kids = children(recs, 0);
            let keep = kids.len().checked_sub(count)?;
            let to = kids.get(keep).copied().unwrap_or(recs.len());
            Some((1..to).map(|i| (i, recs[i].depth)).collect())
        }
        View::LastParagraph => {
            let current = (0..recs.len()).rev().find(|&i| {
                matches!(recs[i].value, NodeValue::Paragraph)
                    && !is_inline_footnote_paragraph(recs, i)
            })?;
            let viewed = subtree(recs, current, recs[current].depth).collect();
            Some(with_root_definitions(recs, current, viewed))
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
    inserted_at: Option<usize>,
}

fn trial(edits: Vec<Edit>, new_view: View, normalise: Normalise) -> Trial {
    Trial {
        edits,
        old_view: View::Whole,
        new_view,
        normalise,
        options: None,
        same_lines: false,
        inserted_at: None,
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

fn letter_sites(doc: &str) -> Vec<usize> {
    let bytes = doc.as_bytes();
    (1..bytes.len())
        .filter(|&i| bytes[i - 1].is_ascii_alphanumeric() && bytes[i].is_ascii_alphanumeric())
        .collect()
}

fn table_cell_edits(doc: &str, head: &str, tail: &str, inner: Option<Edit>) -> Vec<Edit> {
    let body_len = doc.trim_end_matches(['\n', '\r']).len();
    let mut edits = vec![insert(0, head)];
    edits.extend(inner);
    edits.push(replace(body_len, doc.len() - body_len, tail));
    edits
}

fn continuation_prefix(line: &[u8]) -> String {
    let mut prefix = String::new();
    let mut rest = line;
    loop {
        let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
        let marker = match rest {
            [b'>', ..] | [b' ' | b'\t', ..] => 1,
            [b'-' | b'+' | b'*', b' ' | b'\t', ..] => 1,
            _ if digits > 0
                && matches!(rest.get(digits), Some(b'.' | b')'))
                && matches!(rest.get(digits + 1), Some(b' ' | b'\t')) =>
            {
                digits + 1
            }
            _ => return prefix,
        };
        for &b in &rest[..marker] {
            prefix.push(match b {
                b'>' | b'\t' => b as char,
                _ => ' ',
            });
        }
        rest = &rest[marker..];
    }
}

fn has_bare_pipe(doc: &str) -> bool {
    let mut bytes = doc.bytes();
    while let Some(b) = bytes.next() {
        match b {
            b'\\' => {
                bytes.next();
            }
            b'|' => return true,
            _ => {}
        }
    }
    false
}

fn is_grid_wrap(family: &str) -> bool {
    matches!(family, "grid-cell" | "grid-second-cell")
}

fn grid_wrap(doc: &str, source: &Source, second: bool) -> Vec<Edit> {
    let widths: Vec<usize> = (0..source.line_count())
        .map(|line| {
            String::from_utf8_lossy(source.line_content(line))
                .chars()
                .map(|c| c.width().unwrap_or(0))
                .sum()
        })
        .collect();
    let width = widths.iter().copied().max().unwrap_or(0) + 2;
    let border = format!(
        "{}+{}+\n",
        if second { "+---" } else { "" },
        "-".repeat(width)
    );

    let mut edits = vec![insert(0, &border)];
    for (line, &w) in widths.iter().enumerate() {
        let prefix = match (second, line) {
            (false, _) => "| ",
            (true, 0) => "| x | ",
            (true, _) => "|   | ",
        };
        edits.push(insert(source.line_starts[line], prefix));
        edits.push(insert(
            source.content_ends[line],
            &format!("{}|", " ".repeat(width - 1 - w)),
        ));
    }
    let last = source.line_count() - 1;
    let eol = if source.line_ends[last] > source.content_ends[last] {
        ""
    } else {
        "\n"
    };
    edits.push(insert(doc.len(), &format!("{eol}{border}")));
    edits
}

const HEADER_CELL: (&str, &str, &[usize]) = ("| ", " |\n| - |\n", &[0, 0, 0]);
const BODY_CELL: (&str, &str, &[usize]) = ("| a |\n| - |\n| ", " |\n", &[0, 1, 0]);

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
            let sites = letter_sites(doc);
            let mut rng = Rng::with_seed(hash(doc));
            (0..sites.len().min(12))
                .map(|_| {
                    let site = sites[rng.usize(..sites.len())];
                    Trial {
                        inserted_at: Some(site),
                        ..trial(
                            vec![insert(site, "Q")],
                            View::Whole,
                            Normalise::InsertedLetter,
                        )
                    }
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
            let (head, tail, path) = if family == "table-header-cell" {
                HEADER_CELL
            } else {
                BODY_CELL
            };
            vec![Trial {
                old_view: View::Path(&[0]),
                ..trial(
                    table_cell_edits(doc, head, tail, None),
                    View::Path(path),
                    Normalise::Exact,
                )
            }]
        }
        "table-pipe" if options.extension.table && is_single_line(doc) && !has_bare_pipe(doc) => {
            let sites = letter_sites(doc);
            let mut rng = Rng::with_seed(hash(doc));
            (0..sites.len().min(12))
                .map(|attempt| {
                    let site = sites[rng.usize(..sites.len())];
                    let (head, tail, path) = if attempt % 2 == 0 {
                        HEADER_CELL
                    } else {
                        BODY_CELL
                    };
                    Trial {
                        old_view: View::Path(&[0]),
                        inserted_at: Some(site),
                        ..trial(
                            table_cell_edits(doc, head, tail, Some(insert(site, "\\|"))),
                            View::Path(path),
                            Normalise::Pipe,
                        )
                    }
                })
                .collect()
        }
        "table-preface" if options.extension.table => {
            let prefix = continuation_prefix(source.line_content(source.line_count() - 1));
            let eol = if doc.ends_with(['\n', '\r']) {
                ""
            } else {
                "\n"
            };
            vec![Trial {
                old_view: View::LastParagraph,
                ..trial(
                    vec![insert(
                        doc.len(),
                        &format!("{eol}{prefix}| a |\n{prefix}| - |\n"),
                    )],
                    View::LastParagraph,
                    Normalise::Exact,
                )
            }]
        }
        "heading" | "table-header-cell" | "table-body-cell" | "table-pipe" | "table-preface" => {
            vec![]
        }
        "grid-cell" | "grid-second-cell"
            if options.extension.grid_table
                && !doc.is_empty()
                && !doc.contains('\t')
                && (0..source.line_count())
                    .all(|line| !source.line_content(line).ends_with(b" ")) =>
        {
            let second = family == "grid-second-cell";
            vec![trial(
                grid_wrap(doc, &source, second),
                View::Path(if second { &[0, 0, 1] } else { &[0, 0, 0] }),
                Normalise::Exact,
            )]
        }
        "grid-cell" | "grid-second-cell" => vec![],
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
    old_view: &[(usize, usize)],
    new_view: &[(usize, usize)],
) -> bool {
    let nothing_outside_view = match trial.new_view {
        View::Path(path)
            if !matches!(
                family,
                "table-header-cell"
                    | "table-body-cell"
                    | "table-pipe"
                    | "heading"
                    | "grid-second-cell"
            ) =>
        {
            new.len() == new_view.len() + path.len() + 1
        }
        _ => true,
    };
    nothing_outside_view
        && old_view.len() == new_view.len()
        && old_view
            .iter()
            .zip(new_view)
            .all(|(&(a, a_depth), &(b, b_depth))| {
                let (x, y) = (&old[a], &new[b]);
                x.kind == y.kind
                    && a_depth == b_depth
                    && normalised(trial.normalise, &x.fingerprint)
                        == normalised(trial.normalise, &y.fingerprint)
                    && (!trial.same_lines
                        || (x.sp.start.line == y.sp.start.line && x.sp.end.line == y.sp.end.line))
            })
}

fn footnote_definitions(doc: &str, options: &Options<'static>) -> Vec<Rec> {
    let mut options = options.clone();
    options.parse.leave_footnote_definitions = true;
    let recs = super::model::snapshot(doc, &options);
    (0..recs.len())
        .filter(|&i| matches!(recs[i].value, NodeValue::FootnoteDefinition(_)))
        .flat_map(|i| subtree(&recs, i, recs[i].depth))
        .map(|(i, depth)| Rec {
            depth,
            ..recs[i].clone()
        })
        .collect()
}

fn same_definitions(trial: &Trial, old: &[Rec], new: &[Rec]) -> bool {
    old.len() == new.len()
        && old.iter().zip(new).all(|(x, y)| {
            x.kind == y.kind
                && x.depth == y.depth
                && normalised(trial.normalise, &x.fingerprint)
                    == normalised(trial.normalise, &y.fingerprint)
        })
}

fn is_exempt(family: &str, old: &[Rec], index: usize) -> bool {
    let rec = &old[index];
    let is_break = matches!(rec.value, NodeValue::SoftBreak | NodeValue::LineBreak);
    match family {
        "crlf" => is_break,
        "trailing-space" => is_break || rec.value.block(),
        "option-toggle" | "grid-cell" | "grid-second-cell" => {
            is_inline_footnote_paragraph(old, index)
        }
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
    let hidden_definitions = options.extension.footnotes
        && !options.parse.leave_footnote_definitions
        && doc.contains("[^");
    let old_definitions = hidden_definitions.then(|| footnote_definitions(doc, options));
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
        if !same_tree(family, &trial, old, &new, &old_view, &new_view)
            || old_definitions.as_ref().is_some_and(|old_definitions| {
                let new_definitions =
                    footnote_definitions(&edited, trial.options.as_ref().unwrap_or(options));
                !same_definitions(&trial, old_definitions, &new_definitions)
            })
        {
            outcome.failed += 1;
            continue;
        }
        for (&(a, _), &(b, _)) in old_view.iter().zip(&new_view) {
            if is_exempt(family, old, a) {
                continue;
            }
            let (x, y) = (&old[a], &new[b]);
            let placeholder = is_placeholder(old, a);
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
                if trial
                    .inserted_at
                    .is_some_and(|site| site == start || site == end + 1)
                {
                    return;
                }
                let at_eol = |offset: usize, line: usize| {
                    let content_end = source.content_ends[line - 1];
                    (is_grid_wrap(family) && offset >= content_end)
                        .then(|| map_start(&trial.edits, content_end))
                };
                let want_end = edited_source.position(
                    at_eol(end, old_sp.end.line)
                        .map_or_else(|| map_end(&trial.edits, end), |eol| eol - 2),
                );
                let want_start = if placeholder {
                    LineColumn {
                        column: want_end.column + 1,
                        ..want_end
                    }
                } else {
                    edited_source.position(
                        at_eol(start, old_sp.start.line)
                            .map_or_else(|| map_start(&trial.edits, start), |eol| eol - 1),
                    )
                };
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
