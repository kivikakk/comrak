use fastrand::Rng;
use unicode_width::UnicodeWidthChar;

const WORDS: &[&str] = &[
    "a", "foo", "bar", "baz", "x", "été", "好的", "𐀀z", "1", "word", "a_b", "c*d", "e-f",
];

fn pick<'a>(rng: &mut Rng, choices: &[&'a str]) -> &'a str {
    choices[rng.usize(..choices.len())]
}

fn word(rng: &mut Rng) -> String {
    pick(rng, WORDS).to_string()
}

fn nested(rng: &mut Rng, depth: usize) -> String {
    if depth == 0 {
        word(rng)
    } else {
        let count = 1 + rng.usize(..2);
        inline(rng, depth - 1, count)
    }
}

fn atom(rng: &mut Rng, depth: usize) -> String {
    match rng.usize(..46) {
        0..=7 => word(rng),
        8 => format!("*{}*", nested(rng, depth)),
        9 => format!("_{}_", nested(rng, depth)),
        10 => format!("**{}**", nested(rng, depth)),
        11 => format!("__{}__", nested(rng, depth)),
        12 => format!("`{}`", word(rng)),
        13 => format!("`` {} ` ``", word(rng)),
        14 => format!("[{}](/u)", nested(rng, depth)),
        15 => format!("[{}](/u \"t\")", nested(rng, depth)),
        16 => format!("[{}][r]", nested(rng, depth)),
        17 => format!("[{}][]", word(rng)),
        18 => "[r]".to_string(),
        19 => format!("![{}](/i)", nested(rng, depth)),
        20 => "<http://a.b/c>".to_string(),
        21 => "<a@b.c>".to_string(),
        22 => "http://a.b/c".to_string(),
        23 => "www.a.bc".to_string(),
        24 => "a@b.cd".to_string(),
        25 => format!("<b c=\"{}\">", word(rng)),
        26 => pick(
            rng,
            &["\\*", "\\_", "\\\\", "\\[", "\\`", "\\<", "\\|", "\\~"],
        )
        .to_string(),
        27 => pick(rng, &["&amp;", "&#35;", "&ouml;", "&nosuch;", "&#x1F600;"]).to_string(),
        28 => format!("~~{}~~", nested(rng, depth)),
        29 => format!("~{}~", nested(rng, depth)),
        30 => format!("^{}^", nested(rng, depth)),
        31 => format!("=={}==", nested(rng, depth)),
        32 => format!("++{}++", nested(rng, depth)),
        33 => format!("||{}||", nested(rng, depth)),
        34 => pick(rng, &["$x$", "$$x$$", "$`x`$", "\\(x\\)", "\\[x\\]"]).to_string(),
        35 => "[^1]".to_string(),
        36 => format!("^[{}]", nested(rng, depth)),
        37 => "[[w]]".to_string(),
        38 => format!("[[w|{}]]", word(rng)),
        39 => ":fire:".to_string(),
        40 => "%%c%%".to_string(),
        41 => pick(rng, &["\"q\"", "'q'", "--", "---", "...", "a'b"]).to_string(),
        42 => format!("`{}`{{.c}}", word(rng)),
        43 => format!("[{}](/u){{.c}}", word(rng)),
        44 => pick(
            rng,
            &["[", "]", "*", "_", "`", "<", "!", "(", ")", "|", "~", "^"],
        )
        .to_string(),
        _ => pick(rng, &["<!-- c -->", "<?p?>", "<![CDATA[z]]>", "</b>"]).to_string(),
    }
}

fn inline(rng: &mut Rng, depth: usize, count: usize) -> String {
    let mut out = String::new();
    for i in 0..count.max(1) {
        if i > 0 {
            out.push_str(pick(rng, &["", "  ", " ", " ", " ", " "]));
        }
        out.push_str(&atom(rng, depth));
    }
    out
}

fn paragraph_lines(rng: &mut Rng) -> Vec<String> {
    let count = 1 + rng.usize(..3);
    let mut lines = vec![];
    for i in 0..count {
        let atoms = 1 + rng.usize(..4);
        let mut line = inline(rng, 2, atoms);
        if i + 1 < count {
            line.push_str(pick(rng, &["  ", "\\", " ", "", "", ""]));
        }
        if i > 0 {
            line.insert_str(0, pick(rng, &[" ", "   ", "", "", "", ""]));
        }
        lines.push(line);
    }
    let mut text = lines.join("\n");
    if rng.usize(..10) == 0 {
        let spanning = pick(
            rng,
            &[
                "[a\nb](/u)",
                "[a](/u\n\"t\")",
                "[a](\n/u)",
                "[a](/u \"t\nv\")",
                "[a][\nr]",
                "`a\nb`",
                "<b\nc=\"d\">",
                "*a\nb*",
                "![a\nb](/i)",
                "$a\nb$",
                "%%a\nb%%",
                "[[w\nz]]",
                "^[a\nb]",
            ],
        );
        text.push(' ');
        text.push_str(spanning);
        text.push(' ');
        text.push_str(&word(rng));
    }
    text.split('\n').map(str::to_string).collect()
}

fn indent(rng: &mut Rng) -> &'static str {
    pick(rng, &[" ", "  ", "   ", "", "", "", "", "", "", "", "", ""])
}

fn with_first_indent<'a>(rng: &mut Rng, lines: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let indentation = indent(rng);
    lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                format!("{indentation}{line}")
            } else {
                line.to_string()
            }
        })
        .collect()
}

fn prefixed(rng: &mut Rng, lines: Vec<String>, first: &str, rest: &str, lazy: bool) -> Vec<String> {
    lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                format!("{first}{line}")
            } else if line.is_empty() {
                rest.trim_end().to_string()
            } else if lazy && rng.usize(..4) == 0 {
                line
            } else {
                format!("{rest}{line}")
            }
        })
        .collect()
}

fn table_row(rng: &mut Rng, cells: usize, leading_pipe: bool) -> String {
    let mut row = vec![];
    for _ in 0..cells {
        row.push(match rng.usize(..5) {
            0 => String::new(),
            1 => "a\\|b".to_string(),
            2 => "`x\\|y`".to_string(),
            _ => inline(rng, 1, 1),
        });
    }
    let body = row.join(" | ");
    if leading_pipe {
        format!("| {body} |")
    } else {
        body
    }
}

fn table(rng: &mut Rng) -> Vec<String> {
    let columns = 1 + rng.usize(..3);
    let leading_pipe = rng.usize(..3) != 0;
    let mut delimiters = vec![];
    for _ in 0..columns {
        delimiters.push(pick(rng, &[":-", "-:", ":-:", "---"]));
    }
    let delimiters = delimiters.join(" | ");
    let delimiter_row = if leading_pipe || columns == 1 {
        format!("| {delimiters} |")
    } else {
        delimiters
    };
    let mut lines = vec![];
    if rng.usize(..5) == 0 {
        lines.push(word(rng));
    }
    lines.push(table_row(rng, columns, leading_pipe));
    lines.push(delimiter_row);
    for _ in 0..rng.usize(..3) {
        let cells = if rng.usize(..5) == 0 {
            columns + 1
        } else {
            columns
        };
        let mut row = table_row(rng, cells, leading_pipe);
        if rng.usize(..5) == 0 {
            row.insert(0, ' ');
        }
        if rng.usize(..5) == 0 {
            row.push_str("  ");
        }
        lines.push(row);
    }
    lines
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

fn grid_cell_content(rng: &mut Rng) -> Vec<String> {
    match rng.usize(..12) {
        0..=5 => {
            let atoms = 1 + rng.usize(..3);
            vec![inline(rng, 1, atoms)]
        }
        6 => paragraph_lines(rng),
        7 => vec![format!("- {}", word(rng)), format!("- {}", word(rng))],
        8 => vec!["```".to_string(), word(rng), "```".to_string()],
        9 => vec![],
        10 => vec![format!("> {}", word(rng))],
        _ => vec![word(rng), String::new(), format!("    {}", word(rng))],
    }
}

struct GridCell {
    top: usize,
    left: usize,
    bottom: usize,
    right: usize,
    lines: Vec<String>,
}

fn grid_table(rng: &mut Rng) -> Vec<String> {
    let columns = 1 + rng.usize(..3);
    let rows = 1 + rng.usize(..3);
    let mut taken = vec![vec![false; columns]; rows];
    let mut cells = vec![];
    for r in 0..rows {
        for c in 0..columns {
            if taken[r][c] {
                continue;
            }
            let right = if c + 1 < columns && !taken[r][c + 1] && rng.usize(..4) == 0 {
                c + 2
            } else {
                c + 1
            };
            let bottom =
                if r + 1 < rows && (c..right).all(|c| !taken[r + 1][c]) && rng.usize(..4) == 0 {
                    r + 2
                } else {
                    r + 1
                };
            for row in taken.iter_mut().take(bottom).skip(r) {
                for slot in row.iter_mut().take(right).skip(c) {
                    *slot = true;
                }
            }
            cells.push(GridCell {
                top: r,
                left: c,
                bottom,
                right,
                lines: grid_cell_content(rng),
            });
        }
    }

    let mut widths = vec![3; columns];
    let mut heights = vec![1; rows];
    for cell in cells.iter().filter(|cell| cell.right == cell.left + 1) {
        let need = cell
            .lines
            .iter()
            .map(|l| display_width(l))
            .max()
            .unwrap_or(0)
            + 2;
        widths[cell.left] = widths[cell.left].max(need);
    }
    for cell in cells.iter().filter(|cell| cell.bottom == cell.top + 1) {
        heights[cell.top] = heights[cell.top].max(cell.lines.len());
    }
    for cell in &cells {
        let need = cell
            .lines
            .iter()
            .map(|l| display_width(l))
            .max()
            .unwrap_or(0)
            + 2;
        let have: usize =
            widths[cell.left..cell.right].iter().sum::<usize>() + cell.right - cell.left - 1;
        if have < need {
            widths[cell.right - 1] += need - have;
        }
        let have: usize =
            heights[cell.top..cell.bottom].iter().sum::<usize>() + cell.bottom - cell.top - 1;
        if have < cell.lines.len() {
            heights[cell.bottom - 1] += cell.lines.len() - have;
        }
    }

    let mut xs = vec![0];
    for w in &widths {
        xs.push(xs.last().unwrap() + w + 1);
    }
    let mut ys = vec![0];
    for h in &heights {
        ys.push(ys.last().unwrap() + h + 1);
    }
    let (width, height) = (*xs.last().unwrap() + 1, *ys.last().unwrap() + 1);
    let mut canvas = vec![vec![Some(' '); width]; height];

    for cell in &cells {
        let (x0, x1, y0, y1) = (xs[cell.left], xs[cell.right], ys[cell.top], ys[cell.bottom]);
        for x in x0 + 1..x1 {
            canvas[y0][x] = Some('-');
            canvas[y1][x] = Some('-');
        }
        for row in canvas.iter_mut().take(y1).skip(y0 + 1) {
            row[x0] = Some('|');
            row[x1] = Some('|');
        }
    }
    for cell in &cells {
        let (x0, x1, y0, y1) = (xs[cell.left], xs[cell.right], ys[cell.top], ys[cell.bottom]);
        for (y, x) in [(y0, x0), (y0, x1), (y1, x0), (y1, x1)] {
            canvas[y][x] = Some('+');
        }
        for (i, line) in cell.lines.iter().enumerate() {
            let mut x = x0 + 2;
            for ch in line.chars() {
                let w = ch.width().unwrap_or(0);
                if w == 0 {
                    continue;
                }
                canvas[y0 + 1 + i][x] = Some(ch);
                for slot in &mut canvas[y0 + 1 + i][x + 1..x + w] {
                    *slot = None;
                }
                x += w;
            }
        }
    }

    let unspanned = |boundary: usize| {
        cells
            .iter()
            .all(|cell| cell.top >= boundary || cell.bottom <= boundary)
    };
    let mut rules = vec![];
    if rows > 1 && rng.usize(..2) == 0 {
        let boundary = 1 + rng.usize(..rows - 1);
        if unspanned(boundary) {
            rules.push(ys[boundary]);
        }
    }
    if rng.usize(..4) == 0 {
        if rows > 1 && unspanned(rows - 1) {
            rules.push(ys[rows - 1]);
        }
        rules.push(height - 1);
    }
    for &y in &rules {
        for slot in canvas[y].iter_mut().filter(|s| **s == Some('-')) {
            *slot = Some('=');
        }
    }
    let alignment_line = rules.first().copied().unwrap_or(0);
    let pluses: Vec<usize> = (0..width)
        .filter(|&x| canvas[alignment_line][x] == Some('+'))
        .collect();
    for pair in pluses.windows(2) {
        if pair[1] - pair[0] >= 4 {
            if rng.usize(..3) == 0 {
                canvas[alignment_line][pair[0] + 1] = Some(':');
            }
            if rng.usize(..3) == 0 {
                canvas[alignment_line][pair[1] - 1] = Some(':');
            }
        }
    }

    let indentation = pick(rng, &["", "", "", " ", "   "]);
    let mut lines: Vec<String> = canvas
        .into_iter()
        .map(|row| {
            format!(
                "{indentation}{}",
                row.into_iter().flatten().collect::<String>()
            )
        })
        .collect();
    if rng.usize(..8) == 0 {
        let line = rng.usize(..lines.len());
        if let Some((at, _)) = lines[line]
            .char_indices()
            .nth(rng.usize(..lines[line].chars().count()))
        {
            lines[line].remove(at);
        }
    }
    lines
}

fn fenced_code(rng: &mut Rng) -> Vec<String> {
    let fence = pick(rng, &["```", "~~~~"]);
    let info = pick(rng, &["", "rust", " rust x", "rust {.c}"]);
    let mut lines = vec![format!("{}{fence}{info}", indent(rng))];
    for _ in 0..rng.usize(..3) {
        lines.push(match rng.usize(..4) {
            0 => String::new(),
            1 => format!("  {}", word(rng)),
            _ => word(rng),
        });
    }
    if rng.usize(..4) != 0 {
        lines.push(format!("{}{fence}", indent(rng)));
    }
    lines
}

fn leaf_block(rng: &mut Rng) -> Vec<String> {
    match rng.usize(..26) {
        0..=5 => {
            let lines = paragraph_lines(rng);
            with_first_indent(rng, lines.iter().map(String::as_str))
        }
        6 => {
            let hashes = "#".repeat(1 + rng.usize(..6));
            let atoms = 1 + rng.usize(..3);
            let tail = pick(rng, &[" #", " ##  ", "", ""]);
            let attrs = if rng.usize(..8) == 0 { " {#i .c}" } else { "" };
            let body = inline(rng, 1, atoms);
            vec![format!("{}{hashes} {body}{attrs}{tail}", indent(rng))]
        }
        7 => {
            let mut lines = paragraph_lines(rng);
            let underline = pick(rng, &["===", "---"]);
            lines.push(format!("{}{underline}", indent(rng)));
            lines
        }
        8 | 9 => fenced_code(rng),
        10 => {
            let mut lines = vec![format!("    {}", word(rng))];
            if rng.usize(..2) == 0 {
                lines.push(String::new());
                lines.push(format!("      {}", word(rng)));
            }
            if rng.usize(..4) == 0 {
                lines.push("    ".to_string());
            }
            lines
        }
        11 | 12 => {
            let forms: &[&[&str]] = &[
                &["<div>", "x", "</div>"],
                &["<div>"],
                &["<pre>", "x", "", "y</pre>"],
                &["<!-- a", "b -->"],
                &["<?a", "?>"],
                &["<x y=\"z\">"],
                &["<script>", "", "</script> t"],
            ];
            let form = forms[rng.usize(..forms.len())];
            with_first_indent(rng, form.iter().copied())
        }
        13 => vec![pick(
            rng,
            &["---", "***", "___", "- - -", " * * *", "---  ", "***\t"],
        )
        .to_string()],
        14 | 15 => table(rng),
        16 => pick(
            rng,
            &[
                "[r]: /u",
                "[r]: /u \"t\"",
                "[r]:\n/u",
                "[r]: /u\n\"t\"",
                "[r]: /u\nmore",
                "[r]: /u\n[s]: /v\nmore",
            ],
        )
        .split('\n')
        .map(str::to_string)
        .collect(),
        17 => vec![pick(rng, &["-# sub", "-# **b** c", "-#  x"]).to_string()],
        18 => {
            let mut lines = vec!["%%".to_string()];
            lines.extend(paragraph_lines(rng));
            if rng.usize(..3) != 0 {
                lines.push("%%".to_string());
            }
            lines
        }
        19 => vec![format!("{} ^abc", word(rng))],
        20 => vec![format!(">{}", word(rng))],
        21 => vec!["$$".to_string(), "x".to_string(), "$$".to_string()],
        22 => vec![format!("{}<.c a=\"b\">x</.c>", indent(rng))],
        23 | 24 => grid_table(rng),
        _ => vec![word(rng), String::new(), format!(": {}", word(rng))],
    }
}

fn block_quote(rng: &mut Rng, depth: usize) -> Vec<String> {
    let inner = blocks(rng, depth - 1);
    let first = pick(rng, &[">", "> ", "> ", " > ", ">  "]);
    let rest = if rng.usize(..5) == 0 { ">" } else { first };
    let lazy = rng.usize(..3) == 0;
    prefixed(rng, inner, first, rest, lazy)
}

fn bullet_list(rng: &mut Rng, depth: usize) -> Vec<String> {
    let marker = pick(rng, &["- ", "* ", "+ ", "-   "]);
    let padding = " ".repeat(marker.len());
    let count = 1 + rng.usize(..3);
    let mut lines = vec![];
    for i in 0..count {
        let mut inner = if rng.usize(..8) == 0 {
            vec![String::new()]
        } else {
            blocks(rng, depth - 1)
        };
        if rng.usize(..3) == 0 {
            inner[0].insert_str(0, pick(rng, &["[ ] ", "[x] ", "[X] ", "[é] "]));
        }
        let lazy = rng.usize(..5) == 0;
        lines.extend(prefixed(rng, inner, marker, &padding, lazy));
        if i + 1 < count && rng.usize(..3) == 0 {
            lines.push(String::new());
        }
    }
    lines
}

fn ordered_list(rng: &mut Rng, depth: usize) -> Vec<String> {
    let mut lines = vec![];
    for i in 0..1 + rng.usize(..2) {
        let marker = format!("{}. ", i + 1);
        let padding = " ".repeat(marker.len());
        let inner = blocks(rng, depth - 1);
        lines.extend(prefixed(rng, inner, &marker, &padding, false));
    }
    lines
}

fn fenced_container(rng: &mut Rng, depth: usize, opens: &[&str], close: &str) -> Vec<String> {
    let inner = blocks(rng, depth - 1);
    let mut lines = vec![pick(rng, opens).to_string()];
    lines.extend(inner);
    if rng.usize(..3) != 0 {
        lines.push(close.to_string());
    }
    lines
}

fn block(rng: &mut Rng, depth: usize) -> Vec<String> {
    if depth == 0 {
        return leaf_block(rng);
    }
    match rng.usize(..34) {
        0..=23 => leaf_block(rng),
        24..=26 => block_quote(rng, depth),
        27 | 28 => bullet_list(rng, depth),
        29 => ordered_list(rng, depth),
        30 => fenced_container(rng, depth, &[">>>"], ">>>"),
        31 => fenced_container(rng, depth, &[":::", ":::warn", "::::"], ":::"),
        32 => {
            let inner = blocks(rng, depth - 1);
            let head = pick(
                rng,
                &["[!note]", "[!WARNING] T", "[!tip]-", "[!note] **t**"],
            );
            let mut lines = vec![format!("> {head}")];
            lines.extend(inner.into_iter().map(|line| format!("> {line}")));
            lines
        }
        _ => {
            let inner = blocks(rng, depth - 1);
            prefixed(rng, inner, "[^1]: ", "    ", false)
        }
    }
}

fn blocks(rng: &mut Rng, depth: usize) -> Vec<String> {
    let mut lines = vec![];
    for i in 0..1 + rng.usize(..3) {
        if i > 0 && rng.usize(..3) != 0 {
            lines.push(String::new());
        }
        lines.extend(block(rng, depth));
    }
    lines
}

fn with_tabs(rng: &mut Rng, line: &str) -> String {
    let spaces = line.bytes().take_while(|&b| b == b' ').count();
    if spaces >= 2 && rng.usize(..2) == 0 {
        return format!("\t{}", &line[spaces.min(4)..]);
    }
    if line.starts_with("> ") && rng.usize(..2) == 0 {
        return format!(">\t{}", &line[2..]);
    }
    if (line.starts_with("- ") || line.starts_with("* ")) && rng.usize(..2) == 0 {
        return format!("{}\t{}", &line[..1], &line[2..]);
    }
    line.to_string()
}

pub fn documents(count: usize, seed: u64) -> Vec<String> {
    let mut rng = Rng::with_seed(seed);
    let mut docs = Vec::with_capacity(count);
    for _ in 0..count {
        let depth = 1 + rng.usize(..3);
        let mut lines = blocks(&mut rng, depth);
        if rng.usize(..30) == 0 {
            let mut with_front_matter =
                vec!["---".to_string(), "a: b".to_string(), "---".to_string()];
            if rng.usize(..2) == 0 {
                with_front_matter.push(String::new());
            }
            with_front_matter.extend(lines);
            lines = with_front_matter;
        }
        if rng.usize(..5) == 0 {
            lines = lines.iter().map(|line| with_tabs(&mut rng, line)).collect();
        }
        if rng.usize(..8) == 0 {
            for line in lines.iter_mut().filter(|line| !line.is_empty()) {
                if rng.usize(..3) == 0 {
                    line.push(' ');
                }
            }
        }
        if rng.usize(..10) == 0 {
            for line in lines.iter_mut().filter(|line| line.is_empty()) {
                if rng.usize(..2) == 0 {
                    line.push_str("  ");
                }
            }
        }
        let eol = pick(
            &mut rng,
            &["\r\n", "\r", "\n", "\n", "\n", "\n", "\n", "\n", "\n", "\n"],
        );
        let mut doc = lines.join(eol);
        match rng.usize(..6) {
            0 => {}
            1 => {
                doc.push_str(eol);
                doc.push_str(eol);
            }
            _ => doc.push_str(eol),
        }
        if doc.len() <= super::MAX_DOC_LEN {
            docs.push(doc);
        }
    }
    docs
}
