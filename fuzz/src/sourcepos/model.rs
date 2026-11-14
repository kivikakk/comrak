use comrak::nodes::{LineColumn, NodeValue, Sourcepos};
use comrak::{parse_document, Arena, Options};
use std::ops::Range;

#[derive(Clone)]
pub struct Rec {
    pub kind: String,
    pub fingerprint: String,
    pub value: NodeValue,
    pub has_attrs: bool,
    pub depth: usize,
    pub parent: Option<usize>,
    pub sp: Sourcepos,
    pub symbol_sp: Option<Sourcepos>,
}

fn fingerprint(value: &NodeValue) -> String {
    let mut value = value.clone();
    match &mut value {
        NodeValue::TaskItem(t) => t.symbol_sourcepos = (0, 0, 0, 0).into(),
        NodeValue::CodeBlock(c) => c.fence_offset = 0,
        NodeValue::List(l) | NodeValue::Item(l) => {
            l.marker_offset = 0;
            l.padding = 0;
        }
        NodeValue::DescriptionItem(d) => {
            d.marker_offset = 0;
            d.padding = 0;
        }
        _ => {}
    }
    format!("{value:?}")
}

pub fn snapshot(input: &str, options: &Options) -> Vec<Rec> {
    let arena = Arena::new();
    let root = parse_document(&arena, input, options);
    let mut recs = Vec::new();
    let mut stack = vec![(root, 0, None)];
    while let Some((node, depth, parent)) = stack.pop() {
        let index = recs.len();
        {
            let ast = node.data();
            let fingerprint = fingerprint(&ast.value);
            let kind_len = fingerprint
                .find(|c: char| !c.is_alphanumeric())
                .unwrap_or(fingerprint.len());
            let symbol_sp = match &ast.value {
                NodeValue::TaskItem(t) if t.symbol.is_some() => Some(t.symbol_sourcepos),
                _ => None,
            };
            recs.push(Rec {
                kind: fingerprint[..kind_len].to_string(),
                fingerprint,
                value: ast.value.clone(),
                has_attrs: ast.attrs.is_some(),
                depth,
                parent,
                sp: ast.sourcepos,
                symbol_sp,
            });
        }
        let kids: Vec<_> = node.children().collect();
        for kid in kids.into_iter().rev() {
            stack.push((kid, depth + 1, Some(index)));
        }
    }
    recs
}

pub fn children(recs: &[Rec], index: usize) -> Vec<usize> {
    let depth = recs[index].depth;
    (index + 1..subtree_end(recs, index))
        .filter(|&i| recs[i].depth == depth + 1)
        .collect()
}

pub fn subtree_end(recs: &[Rec], index: usize) -> usize {
    let depth = recs[index].depth;
    (index + 1..recs.len())
        .find(|&i| recs[i].depth <= depth)
        .unwrap_or(recs.len())
}

pub struct Source<'a> {
    pub text: &'a str,
    pub bytes: &'a [u8],
    pub line_starts: Vec<usize>,
    pub line_ends: Vec<usize>,
    pub content_ends: Vec<usize>,
}

impl<'a> Source<'a> {
    pub fn new(text: &'a str) -> Self {
        let bytes = text.as_bytes();
        let mut line_starts = vec![0];
        let mut line_ends = vec![];
        let mut content_ends = vec![];
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'\n' && bytes[i] != b'\r' {
                i += 1;
                continue;
            }
            content_ends.push(i);
            i += if bytes[i] == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                2
            } else {
                1
            };
            line_ends.push(i);
            line_starts.push(i);
        }
        if line_starts.len() > 1 && line_starts.last() == Some(&bytes.len()) {
            line_starts.pop();
        } else {
            content_ends.push(bytes.len());
            line_ends.push(bytes.len());
        }
        Source {
            text,
            bytes,
            line_starts,
            line_ends,
            content_ends,
        }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn line_content(&self, line_index: usize) -> &'a [u8] {
        &self.bytes[self.line_starts[line_index]..self.content_ends[line_index]]
    }

    pub fn offset(&self, position: LineColumn) -> Option<usize> {
        if position.line == 0 || position.line > self.line_count() || position.column == 0 {
            return None;
        }
        let offset = self.line_starts[position.line - 1].checked_add(position.column - 1)?;
        (offset <= self.line_ends[position.line - 1]).then_some(offset)
    }

    fn line_offset(&self, line: usize, column: usize, max: usize) -> Result<usize, &'static str> {
        if line == 0 || column == 0 {
            return Err("zero");
        }
        if line > self.line_count() {
            return Err("out-of-bounds");
        }
        let offset = self.line_starts[line - 1]
            .checked_add(column - 1)
            .ok_or("out-of-bounds")?;
        (offset <= max).then_some(offset).ok_or("out-of-bounds")
    }

    /// Converts a sourcepos (whose end is inclusive) to a half-open byte range.
    ///
    /// A legal empty range is either represented by `end.column == start.column - 1`
    /// on the same line, or by the document sentinel handled in [`Self::document_span`].
    pub fn half_open_span(&self, sp: Sourcepos) -> Result<Range<usize>, &'static str> {
        let empty =
            sp.start.line == sp.end.line && sp.end.column.checked_add(1) == Some(sp.start.column);
        if empty {
            if sp.start.line == 0 {
                return Err("zero");
            }
            if sp.start.line > self.line_count() {
                return Err("out-of-bounds");
            }
            let at = self.line_offset(
                sp.start.line,
                sp.start.column,
                self.content_ends[sp.start.line - 1],
            )?;
            if !self.text.is_char_boundary(at) {
                return Err("splits-utf8");
            }
            return Ok(at..at);
        }
        if sp.end.column == 0 {
            return Err("zero");
        }
        if sp.start.line == 0 || sp.start.column == 0 {
            return Err("zero");
        }
        if sp.start.line > self.line_count() {
            return Err("out-of-bounds");
        }
        let start = self.line_offset(
            sp.start.line,
            sp.start.column,
            self.content_ends[sp.start.line - 1],
        )?;
        let end_line = sp.end.line;
        if end_line == 0 || end_line > self.line_count() {
            return Err("out-of-bounds");
        }
        let end_inclusive = self.line_offset(
            end_line,
            sp.end.column,
            self.line_ends[end_line - 1]
                .checked_sub(1)
                .ok_or("out-of-bounds")?,
        )?;
        if start > end_inclusive {
            return Err("inverted");
        }
        let end = end_inclusive.checked_add(1).ok_or("past-eof")?;
        if !self.text.is_char_boundary(start) || !self.text.is_char_boundary(end) {
            return Err("splits-utf8");
        }
        Ok(start..end)
    }

    /// Normalises the Document node's span, including its empty-input sentinel
    /// and its convention of ending at the last line's content.
    pub fn document_span(&self, sp: Sourcepos) -> Result<Range<usize>, &'static str> {
        if self.bytes.is_empty() {
            if sp.start != (1, 1).into() {
                return Err("start");
            }
            if sp.end != (0, 0).into() {
                return Err("end");
            }
            return Ok(0..0);
        }
        if sp.start != (1, 1).into() {
            return Err("start");
        }
        let last = self.line_count();
        let last_len = self.line_content(last - 1).len();
        let line_start = self.line_starts[last - 1];
        if sp.end == (last, last_len).into() {
            Ok(0..line_start + last_len)
        } else if last_len == 0 && sp.end == (last, 1).into() {
            Ok(0..line_start + 1)
        } else {
            Err("end")
        }
    }

    pub fn position(&self, offset: usize) -> LineColumn {
        let index = self.line_starts.partition_point(|&start| start <= offset) - 1;
        LineColumn {
            line: index + 1,
            column: offset - self.line_starts[index] + 1,
        }
    }

    pub fn span(&self, sp: Sourcepos) -> Option<(usize, usize)> {
        let (start, end) = (self.offset(sp.start)?, self.offset(sp.end)?);
        (start <= end && end < self.bytes.len().max(1)).then_some((start, end))
    }

    pub fn slice(&self, sp: Sourcepos) -> Option<&'a [u8]> {
        let (start, end) = self.span(sp)?;
        self.bytes.get(start..=end)
    }
}

#[derive(Clone, Debug)]
pub struct Finding {
    pub check: String,
    pub kind: String,
    pub reason: String,
    pub detail: String,
    pub edited: Option<String>,
}

impl Finding {
    pub fn new(check: &str, kind: &str, reason: &str, detail: String) -> Self {
        Finding {
            check: check.to_string(),
            kind: kind.to_string(),
            reason: reason.to_string(),
            detail,
            edited: None,
        }
    }

    pub fn bucket(&self) -> String {
        format!("{}/{}/{}", self.check, self.kind, self.reason)
    }
}

pub fn hash(s: &str) -> u64 {
    // https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function
    s.bytes().fold(0xcbf29ce484222325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    })
}
