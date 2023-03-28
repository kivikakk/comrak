use crate::ctype::{ispunct, isspace};
use crate::entity;
use crate::parser::AutolinkType;
use std::ptr;
use std::str;

pub fn unescape(v: &mut Vec<u8>) {
    let mut r = 0;
    let mut prev = None;
    let mut found = 0;

    while r < v.len() {
        if v[r] == b'\\' && r + 1 < v.len() && ispunct(v[r + 1]) {
            if let Some(prev) = prev {
                let window = &mut v[(prev + 1 - found)..r];
                shift_buf_left(window, found);
            }
            prev = Some(r);
            found += 1;
        }
        r += 1;
    }

    if let Some(prev) = prev {
        let window = &mut v[(prev + 1 - found)..r];
        shift_buf_left(window, found);
    }

    let new_size = v.len() - found;
    v.truncate(new_size);
}

pub fn clean_autolink(url: &[u8], kind: AutolinkType) -> Vec<u8> {
    let mut url_vec = url.to_vec();
    trim(&mut url_vec);

    if url_vec.is_empty() {
        return url_vec;
    }

    let mut buf = Vec::with_capacity(url_vec.len());
    if kind == AutolinkType::Email {
        buf.extend_from_slice(b"mailto:");
    }

    buf.extend_from_slice(&entity::unescape_html(&url_vec));
    buf
}

pub fn normalize_code(v: &[u8]) -> Vec<u8> {
    let mut r = Vec::with_capacity(v.len());
    let mut i = 0;
    let mut contains_nonspace = false;

    while i < v.len() {
        match v[i] {
            b'\r' => {
                if i + 1 == v.len() || v[i + 1] != b'\n' {
                    r.push(b' ');
                }
            }
            b'\n' => {
                r.push(b' ');
            }
            c => r.push(c),
        }
        if v[i] != b' ' && v[i] != b'\r' && v[i] != b'\n' {
            contains_nonspace = true;
        }

        i += 1
    }

    if contains_nonspace && !r.is_empty() && r[0] == b' ' && r[r.len() - 1] == b' ' {
        r.remove(0);
        r.pop();
    }

    r
}

pub fn remove_trailing_blank_lines(line: &mut String) {
    let line_bytes = line.as_bytes();
    let mut i = line.len() - 1;
    loop {
        let c = line_bytes[i];

        if c != b' ' && c != b'\t' && !is_line_end_char(c) {
            break;
        }

        if i == 0 {
            line.clear();
            return;
        }

        i -= 1;
    }

    for (i, c) in line_bytes.iter().enumerate().take(line.len()).skip(i) {
        if !is_line_end_char(*c) {
            continue;
        }

        line.truncate(i);
        break;
    }
}

pub fn is_line_end_char(ch: u8) -> bool {
    matches!(ch, 10 | 13)
}

pub fn is_space_or_tab(ch: u8) -> bool {
    matches!(ch, 9 | 32)
}

pub fn chop_trailing_hashtags(line: &mut Vec<u8>) {
    rtrim(line);

    let orig_n = line.len() - 1;
    let mut n = orig_n;

    while line[n] == b'#' {
        if n == 0 {
            return;
        }
        n -= 1;
    }

    if n != orig_n && is_space_or_tab(line[n]) {
        line.truncate(n);
        rtrim(line);
    }
}

pub fn rtrim(line: &mut Vec<u8>) {
    let spaces = line.iter().rev().take_while(|&&b| isspace(b)).count();
    let new_len = line.len() - spaces;
    line.truncate(new_len);
}

pub fn ltrim(line: &mut Vec<u8>) {
    let spaces = line.iter().take_while(|&&b| isspace(b)).count();
    shift_buf_left(line, spaces);
    let new_len = line.len() - spaces;
    line.truncate(new_len);
}

pub fn trim(line: &mut Vec<u8>) {
    ltrim(line);
    rtrim(line);
}

pub fn ltrim_slice(mut i: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = i {
        if isspace(*first) {
            i = rest;
        } else {
            break;
        }
    }
    i
}

pub fn rtrim_slice(mut i: &[u8]) -> &[u8] {
    while let [rest @ .., last] = i {
        if isspace(*last) {
            i = rest;
        } else {
            break;
        }
    }
    i
}

pub fn trim_slice(mut i: &[u8]) -> &[u8] {
    i = ltrim_slice(i);
    i = rtrim_slice(i);
    i
}

fn shift_buf_left(buf: &mut [u8], n: usize) {
    assert!(n <= buf.len());
    let keep = buf.len() - n;
    unsafe {
        let dst = buf.as_mut_ptr();
        let src = dst.add(n);
        ptr::copy(src, dst, keep);
    }
}

pub fn clean_url(url: &[u8]) -> Vec<u8> {
    let url = trim_slice(url);

    let url_len = url.len();
    if url_len == 0 {
        return vec![];
    }

    let mut b = entity::unescape_html(url);

    unescape(&mut b);
    b
}

pub fn clean_title(title: &[u8]) -> Vec<u8> {
    let title_len = title.len();
    if title_len == 0 {
        return vec![];
    }

    let first = title[0];
    let last = title[title_len - 1];

    let mut b = if (first == b'\'' && last == b'\'')
        || (first == b'(' && last == b')')
        || (first == b'"' && last == b'"')
    {
        entity::unescape_html(&title[1..title_len - 1])
    } else {
        entity::unescape_html(title)
    };

    unescape(&mut b);
    b
}

pub fn is_blank(s: &[u8]) -> bool {
    for &c in s {
        match c {
            10 | 13 => return true,
            32 | 9 => (),
            _ => return false,
        }
    }
    true
}

pub fn normalize_label(i: &str) -> String {
    // trim_slice only removes bytes from start and end that match isspace();
    // result is UTF-8.
    let i = unsafe { str::from_utf8_unchecked(trim_slice(i.as_bytes())) };

    let mut v = String::with_capacity(i.len());
    let mut last_was_whitespace = false;
    for c in i.chars() {
        for e in c.to_lowercase() {
            if e.is_whitespace() {
                if !last_was_whitespace {
                    last_was_whitespace = true;
                    v.push(' ');
                }
            } else {
                last_was_whitespace = false;
                v.push(e);
            }
        }
    }
    v
}

pub fn split_off_front_matter<'s>(mut s: &'s str, delimiter: &str) -> Option<(&'s str, &'s str)> {
    s = trim_start_match(s, "\u{feff}");

    if !s.starts_with(delimiter) {
        return None;
    }
    let mut start = delimiter.len();
    if s[start..].starts_with('\n') {
        start += 1;
    } else if s[start..].starts_with("\r\n") {
        start += 2;
    } else {
        return None;
    }

    start += match s[start..]
        .find(&("\n".to_string() + delimiter + "\r\n"))
        .or_else(|| s[start..].find(&("\n".to_string() + delimiter + "\n")))
    {
        Some(n) => n + 1 + delimiter.len(),
        None => return None,
    };

    start += if s[start..].starts_with('\n') {
        1
    } else if s[start..].starts_with("\r\n") {
        2
    } else {
        return None;
    };

    start += if s[start..].starts_with('\n') {
        1
    } else if s[start..].starts_with("\r\n") {
        2
    } else {
        0
    };

    Some((&s[..start], &s[start..]))
}

pub fn trim_start_match<'s>(s: &'s str, pat: &str) -> &'s str {
    s.strip_prefix(pat).unwrap_or(s)
}

#[cfg(test)]
pub mod tests {
    use super::{normalize_code, split_off_front_matter};

    #[test]
    fn normalize_code_handles_lone_newline() {
        assert_eq!(normalize_code(&[b'\n']), vec![b' ']);
    }

    #[test]
    fn normalize_code_handles_lone_space() {
        assert_eq!(normalize_code(&[b' ']), vec![b' ']);
    }

    #[test]
    fn front_matter() {
        assert_eq!(
            split_off_front_matter("---\nfoo: bar\n---\nHiiii", "---"),
            Some(("---\nfoo: bar\n---\n", "Hiiii"))
        );
        assert_eq!(
            split_off_front_matter(
                "\u{feff}!@#\r\n\r\nfoo: !@# \r\nquux\n!@#\r\n\n\nYes!\n",
                "!@#"
            ),
            Some(("!@#\r\n\r\nfoo: !@# \r\nquux\n!@#\r\n\n", "\nYes!\n"))
        );
        assert_eq!(
            split_off_front_matter(
                "\u{feff}!@#\r\n\r\nfoo: \n!@# \r\nquux\n!@#\r\n\n\nYes!\n",
                "!@#"
            ),
            Some(("!@#\r\n\r\nfoo: \n!@# \r\nquux\n!@#\r\n\n", "\nYes!\n"))
        );
    }
}
