use std::borrow::Cow;
use std::ptr;
use std::str;

use crate::ctype::{ispunct, isspace, isspace_char};
use crate::entity;
use crate::parser::AutolinkType;

#[derive(PartialEq, Eq)]
pub enum Case {
    Preserve,
    Fold,
}

pub fn unescape(s: &mut String) {
    // SAFETY: we only shift over backslash characters, and always truncate any
    // continuation characters duplicated as a result of the shifting.
    let b = unsafe { s.as_bytes_mut() };
    let mut r = 0;
    let mut prev = None;
    let mut found = 0;

    while r < b.len() {
        if b[r] == b'\\' && r + 1 < b.len() && ispunct(b[r + 1]) {
            if b[r + 1] == b'\\' {
                r += 1;
            }

            if let Some(prev) = prev {
                let window = &mut b[(prev + 1 - found)..r];
                shift_buf_left(window, found);
            }
            prev = Some(r);
            found += 1;
        }
        r += 1;
    }

    if let Some(prev) = prev {
        let window = &mut b[(prev + 1 - found)..r];
        shift_buf_left(window, found);
    }

    if found > 0 {
        let new_size = b.len() - found;
        // HACK: see shift_buf_left.
        b[new_size] = b'\0';
        s.truncate(new_size);
    }
}

pub fn clean_autolink(mut url: &str, kind: AutolinkType) -> Cow<'_, str> {
    url = trim_slice(url);

    if url.is_empty() {
        return "".into();
    }

    if kind == AutolinkType::Email {
        let mut buf = String::with_capacity(url.len() + "mailto:".len());
        buf.push_str("mailto:");
        buf.push_str(&entity::unescape_html(url));
        buf.into()
    } else {
        entity::unescape_html(url)
    }
}

pub fn normalize_code(v: &str) -> Cow<'_, str> {
    let b = v.as_bytes();
    let mut r = String::new();
    let mut offset = 0;
    let mut i = 0;
    let mut contains_nonspace = false;

    while i < b.len() {
        match b[i] {
            b'\r' => {
                if i + 1 == v.len() || b[i + 1] != b'\n' {
                    r.push_str(&v[offset..i]);
                    r.push(' ');
                    offset = i + 1;
                }
            }
            b'\n' => {
                r.push_str(&v[offset..i]);
                r.push(' ');
                offset = i + 1;
            }
            b' ' => {}
            _ => contains_nonspace = true,
        }

        i += 1
    }

    if offset == 0 {
        if contains_nonspace && b[0] == b' ' && b[i - 1] == b' ' {
            return v[1..i - 1].into();
        } else {
            return v.into();
        }
    }

    r.push_str(&v[offset..i]);

    // SAFETY: we only shift over a space, and we are guaranteed to duplicate a
    // space in the last byte of the buffer before truncating.
    let bytes = unsafe { r.as_bytes_mut() };
    let len = bytes.len();
    if contains_nonspace && bytes[0] == b' ' && bytes[len - 1] == b' ' {
        shift_buf_left(bytes, 1);
        r.truncate(len - 2);
    }

    r.into()
}

pub fn remove_trailing_blank_lines(line: &mut String) {
    line.truncate(remove_trailing_blank_lines_ix(line));
}

pub fn remove_trailing_blank_lines_slice(line: &str) -> &str {
    &line[..remove_trailing_blank_lines_ix(line)]
}

fn remove_trailing_blank_lines_ix(line: &str) -> usize {
    let line_bytes = line.as_bytes();
    let mut i = line.len() - 1;
    loop {
        let c = line_bytes[i];

        if c != b' ' && c != b'\t' && !is_line_end_char(c) {
            break;
        }

        if i == 0 {
            return 0;
        }

        i -= 1;
    }

    for (i, c) in line_bytes.iter().enumerate().take(line.len()).skip(i) {
        if !is_line_end_char(*c) {
            continue;
        }

        return i;
    }

    line.len()
}

pub fn is_line_end_char(ch: u8) -> bool {
    matches!(ch, 10 | 13)
}

pub fn is_space_or_tab(ch: u8) -> bool {
    matches!(ch, 9 | 32)
}

/// Chop any trailing sequence of `#` characters from an ATX heading line.
///
/// Returns the possibly-chopped line and a boolean indicating whether
/// trailing hashes were removed (i.e. the heading had a closing sequence).
pub fn chop_trailing_hashes(mut line: &str) -> (&str, bool) {
    line = rtrim_slice(line);

    let orig_n = line.len() - 1;
    let mut n = orig_n;

    let bytes = line.as_bytes();
    while bytes[n] == b'#' {
        if n == 0 {
            return (line, false);
        }
        n -= 1;
    }

    if n != orig_n && is_space_or_tab(bytes[n]) {
        (rtrim_slice(&line[..n]), true)
    } else {
        (line, false)
    }
}

pub fn rtrim(line: &mut String) -> usize {
    let spaces = line
        .as_bytes()
        .iter()
        .rev()
        .take_while(|&&b| isspace(b))
        .count();
    let new_len = line.len() - spaces;
    line.truncate(new_len);
    spaces
}

pub fn ltrim(line: &mut String) -> usize {
    let spaces = ltrim_count(line);
    remove_from_start(line, spaces);
    spaces
}

#[inline]
pub fn ltrim_count(line: &str) -> usize {
    line.as_bytes().iter().take_while(|&&c| isspace(c)).count()
}

pub fn remove_from_start(s: &mut String, n: usize) {
    if n == 0 {
        return;
    }

    if !s.is_char_boundary(n) {
        panic!("remove_from_start results in non UTF-8 string");
    }

    // SAFETY: we've asserted s[n] is a valid UTF-8 boundary, and we truncate
    // any duplicated continuation characters.
    let bytes = unsafe { s.as_bytes_mut() };
    shift_buf_left(bytes, n);
    let new_len = bytes.len() - n;
    // HACK: see shift_buf_left.
    bytes[new_len] = b'\0';
    s.truncate(new_len);
}

pub fn trim(line: &mut String) {
    ltrim(line);
    rtrim(line);
}

pub fn ltrim_slice(i: &str) -> &str {
    // Compared to upstream, this additionally trims U+000C FORM FEED.
    i.trim_start_matches(isspace_char)
}

pub fn rtrim_slice(i: &str) -> &str {
    i.trim_end_matches(isspace_char)
}

pub fn rtrim_cow(s: &mut Cow<str>) {
    match s {
        Cow::Borrowed(ref mut str) => *str = rtrim_slice(str),
        Cow::Owned(string) => {
            rtrim(string);
        }
    }
}

pub fn trim_slice(i: &str) -> &str {
    rtrim_slice(ltrim_slice(i))
}

pub fn trim_cow(s: &mut Cow<str>) {
    match s {
        Cow::Borrowed(ref mut str) => *str = trim_slice(str),
        Cow::Owned(string) => trim(string),
    }
}

// HACK: Using this function safely on a buffer obtained from
// String::as_bytes_mut() requires care when truncating it.
//
// Say the string ends in a multibyte character, i.e. the last byte is a UTF-8
// continuation byte. That byte will be repeated at the end of the buffer when
// it's shifted left by one at a time; therefore the truncation point won't be
// a valid UTF-8 character boundary, and String::truncate will panic. In such
// cases, set the byte immediately after the retained portion to b'\0' (or any
// non-continuation byte!).
fn shift_buf_left(buf: &mut [u8], n: usize) {
    if n == 0 {
        return;
    }
    assert!(n <= buf.len());
    let keep = buf.len() - n;
    // SAFETY: we can copy `keep` bytes from `dst+n` to `dst`, as the full size
    // of the `dst` buffer is `keep+n`.
    unsafe {
        let dst = buf.as_mut_ptr();
        let src = dst.add(n);
        ptr::copy(src, dst, keep);
    }
}

pub fn clean_url(url: &str) -> Cow<'static, str> {
    let url = trim_slice(url);

    if url.is_empty() {
        return "".into();
    }

    let mut b = entity::unescape_html(url).into_owned();
    unescape(&mut b);
    b.into()
}

pub fn clean_title(title: &str) -> Cow<'static, str> {
    let title_len = title.len();
    if title_len == 0 {
        return "".into();
    }

    let bytes = title.as_bytes();
    let first = bytes[0];
    let last = bytes[title_len - 1];

    let mut b = if (first == b'\'' && last == b'\'')
        || (first == b'(' && last == b')')
        || (first == b'"' && last == b'"')
    {
        entity::unescape_html(&title[1..title_len - 1])
    } else {
        entity::unescape_html(title)
    }
    .into_owned();

    unescape(&mut b);
    b.into()
}

pub fn is_blank(s: &str) -> bool {
    for c in s.as_bytes() {
        match c {
            10 | 13 => return true,
            32 | 9 => (),
            _ => return false,
        }
    }
    true
}

pub fn normalize_label(i: &str, casing: Case) -> String {
    let i = trim_slice(i);

    let mut v = String::with_capacity(i.len());
    let mut last_was_whitespace = false;
    for c in i.chars() {
        if c.is_whitespace() {
            if !last_was_whitespace {
                last_was_whitespace = true;
                v.push(' ');
            }
        } else {
            last_was_whitespace = false;
            v.push(c);
        }
    }

    if casing == Case::Fold {
        caseless::default_case_fold_str(&v)
    } else {
        v
    }
}

#[test]
fn normalize_label_fold_test() {
    assert_eq!(normalize_label("Abc   \t\ndef", Case::Preserve), "Abc def");
    assert_eq!(normalize_label("Abc   \t\ndef", Case::Fold), "abc def");
    assert_eq!(normalize_label("Straẞe", Case::Preserve), "Straẞe");
    assert_eq!(normalize_label("Straẞe", Case::Fold), "strasse");
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
        .or_else(|| s[start..].find(&("\n".to_string() + delimiter))) // delimiter followed by EOF
    {
        Some(n) => n + 1 + delimiter.len(),
        None => return None,
    };

    if start == s.len() {
        return Some((s, ""));
    }

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

pub fn count_newlines(input: &str) -> (usize, usize) {
    let bytes = input.as_bytes();
    let mut num_lines = 0;
    let mut last_line_start = 0;
    let mut i = 0;
    while i < input.len() {
        match bytes[i] {
            b'\r' if i + 1 < input.len() && bytes[i + 1] == b'\n' => {
                i += 1;
                num_lines += 1;
                last_line_start = i + 1;
            }
            b'\r' | b'\n' => {
                num_lines += 1;
                last_line_start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let last_line_len = input.len() - last_line_start;
    (num_lines, last_line_len)
}

pub fn newlines_of(s: &str) -> usize {
    if s.ends_with("\r\n") {
        2
    } else if s.ends_with("\r") || s.ends_with("\n") {
        1
    } else {
        0
    }
}

#[cfg(test)]
pub mod tests {
    use super::{
        count_newlines, ltrim, normalize_code, normalize_label, shift_buf_left,
        split_off_front_matter,
    };
    use crate::strings::Case;

    #[test]
    fn normalize_code_handles_lone_newline() {
        assert_eq!(normalize_code("\n"), " ");
    }

    #[test]
    fn normalize_code_handles_lone_space() {
        assert_eq!(normalize_code(" "), " ");
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

    #[test]
    fn normalize_label_lowercase() {
        assert_eq!(normalize_label("  Foo\u{A0}BAR  ", Case::Fold), "foo bar");
        assert_eq!(normalize_label("  FooİBAR  ", Case::Fold), "fooi\u{307}bar");
    }

    #[test]
    fn normalize_label_preserve() {
        assert_eq!(
            normalize_label("  Foo\u{A0}BAR  ", Case::Preserve),
            "Foo BAR"
        );
        assert_eq!(normalize_label("  FooİBAR  ", Case::Preserve), "FooİBAR");
    }

    #[test]
    fn shift_buf_left_ok() {
        let mut b: Vec<u8>;

        b = vec![1, 2, 3, 4, 5, 6];
        shift_buf_left(&mut b, 1);
        assert_eq!(b, vec![2, 3, 4, 5, 6, 6]);

        shift_buf_left(&mut b, 2);
        assert_eq!(b, vec![4, 5, 6, 6, 6, 6]);
    }

    #[test]
    fn ltrim_ok() {
        let mut s: String;

        s = "okay".to_string();
        ltrim(&mut s);
        assert_eq!(s, "okay");

        s = "   okay".to_string();
        ltrim(&mut s);
        assert_eq!(s, "okay");
    }

    #[test]
    fn count_newlines_ok() {
        assert_eq!((0, 7), count_newlines("abcdefg"));
        assert_eq!((2, 0), count_newlines("abc\ndefg\n"));
        assert_eq!((3, 2), count_newlines("abc\rde\nfg\nhi"));
    }
}
