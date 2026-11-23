use finl_unicode::categories::CharacterCategories;
use memchr_n::MemchrN;
use std::borrow::Cow;
use std::str;
use std::sync::OnceLock;

use crate::Arena;
use crate::character_set::character_set;
use crate::ctype::{isalnum, isalpha, isspace};
use crate::nodes::{Node, NodeLink, NodeValue, Sourcepos};
use crate::parser::inlines::{Subject, make_inline};
use crate::parser::{Cancellation, Spx};

pub(crate) fn process_email_autolinks<'a>(
    arena: &'a Arena<'a>,
    node: Node<'a>,
    contents: &mut Cow<'static, str>,
    relaxed_autolinks: bool,
    sourcepos: &mut Sourcepos,
    spx: &mut Spx,
    cancellation: &Cancellation<'_>,
) {
    let mut literal = spx.literal_cursor();
    let Some((mut post, mut before_len, mut skip)) =
        find_email_autolink(arena, contents, relaxed_autolinks, |start, len| {
            literal.is_literal_at(start, len)
        })
    else {
        return;
    };

    let original = std::mem::take(contents);
    let mut offset = 0;
    let mut anchor = node;
    let mut current_sourcepos = *sourcepos;
    let mut first = true;

    loop {
        if cancellation.step() {
            return;
        }
        let initial_end_col = current_sourcepos.end.column;
        current_sourcepos.end.column = spx.consume(before_len);
        let nsp_start_col = spx.next_column().unwrap();
        let nsp_end_col = spx.consume(skip);

        let before = &original[offset..offset + before_len];
        if first {
            *contents = before.to_string().into();
            *sourcepos = current_sourcepos;
            first = false;
        } else {
            let before = make_inline(
                arena,
                NodeValue::Text(before.to_string().into()),
                current_sourcepos,
            );
            anchor.insert_after(before);
            anchor = before;
        }

        let nsp: Sourcepos = (
            current_sourcepos.end.line,
            nsp_start_col,
            current_sourcepos.end.line,
            nsp_end_col,
        )
            .into();
        post.data_mut().sourcepos = nsp;
        post.first_child().unwrap().data_mut().sourcepos = nsp;
        anchor.insert_after(post);
        anchor = post;

        offset += before_len + skip;
        if offset == original.len() {
            return;
        }

        let after_sourcepos: Sourcepos = (
            current_sourcepos.end.line,
            spx.next_column().unwrap(),
            current_sourcepos.end.line,
            initial_end_col,
        )
            .into();

        let mut literal = spx.literal_cursor();
        if let Some((next_post, next_before_len, next_skip)) = find_email_autolink(
            arena,
            &original[offset..],
            relaxed_autolinks,
            |start, len| literal.is_literal_at(start, len),
        ) {
            post = next_post;
            before_len = next_before_len;
            skip = next_skip;
            current_sourcepos = after_sourcepos;
        } else {
            let after = make_inline(
                arena,
                NodeValue::Text(original[offset..].to_string().into()),
                after_sourcepos,
            );
            anchor.insert_after(after);
            return;
        }
    }
}

fn find_email_autolink<'a>(
    arena: &'a Arena<'a>,
    contents: &str,
    relaxed_autolinks: bool,
    mut is_literal: impl FnMut(usize, usize) -> bool,
) -> Option<(Node<'a>, usize, usize)> {
    static MATCHER: OnceLock<MemchrN> = OnceLock::new();
    static RELAXED_MATCHER: OnceLock<MemchrN> = OnceLock::new();

    let matcher = if relaxed_autolinks {
        RELAXED_MATCHER.get_or_init(|| MemchrN::new(b"@"))
    } else {
        MATCHER.get_or_init(|| MemchrN::new(b"@[]"))
    };
    let bytes = contents.as_bytes();
    if !bytes.contains(&b'@') {
        return None;
    }
    let mut bracket_opening = 0;

    for i in matcher.iter(bytes) {
        match bytes[i] {
            b'[' if !relaxed_autolinks => bracket_opening += 1,
            b']' if !relaxed_autolinks => bracket_opening -= 1,
            b'@' => {
                if bracket_opening <= 0
                    && let Some((post, reverse, skip)) =
                        email_match(arena, contents, i, relaxed_autolinks)
                    && is_literal(i - reverse, skip)
                {
                    return Some((post, i - reverse, skip));
                }
            }
            _ => debug_assert!(false, "matcher will only report @, [, or ]"),
        }
    }

    None
}

fn email_match<'a>(
    arena: &'a Arena<'a>,
    contents: &str,
    i: usize,
    relaxed_autolinks: bool,
) -> Option<(Node<'a>, usize, usize)> {
    const EMAIL_OK_SET: [bool; 256] = character_set!(b".+-_");

    let size = contents.len();
    let bytes = contents.as_bytes();

    let mut auto_mailto = true;
    let mut is_xmpp = false;
    let mut rewind = 0;

    while rewind < i {
        let c = bytes[i - rewind - 1];

        if isalnum(c) || EMAIL_OK_SET[c as usize] {
            rewind += 1;
            continue;
        }

        if c == b':' {
            if validate_protocol("mailto", contents, i - rewind - 1) {
                auto_mailto = false;
                rewind += 1;
                continue;
            }

            if validate_protocol("xmpp", contents, i - rewind - 1) {
                is_xmpp = true;
                auto_mailto = false;
                rewind += 1;
                continue;
            }
        }

        break;
    }

    if rewind == 0 {
        return None;
    }

    let mut link_end = 1;
    let mut np = 0;

    while link_end < size - i {
        let c = bytes[i + link_end];

        if isalnum(c) {
            // empty
        } else if c == b'@' {
            return None;
        } else if c == b'.' && link_end < size - i - 1 && isalnum(bytes[i + link_end + 1]) {
            np += 1;
        } else if c == b'/' && is_xmpp {
            // xmpp allows a `/` in the url
        } else if c != b'-' && c != b'_' {
            break;
        }

        link_end += 1;
    }

    if link_end < 2
        || np == 0
        || (!isalpha(bytes[i + link_end - 1]) && bytes[i + link_end - 1] != b'.')
    {
        return None;
    }

    link_end = autolink_delim(&contents[i..], link_end, relaxed_autolinks);
    if link_end == 0 {
        return None;
    }

    let text = &contents[i - rewind..link_end + i];
    let url = if auto_mailto {
        format!("mailto:{text}")
    } else {
        text.to_string()
    };

    let inl = make_inline(
        arena,
        NodeValue::Link(Box::new(NodeLink {
            url,
            title: String::new(),
        })),
        (0, 1, 0, 1).into(),
    );

    inl.append(make_inline(
        arena,
        NodeValue::Text(text.to_string().into()),
        (0, 1, 0, 1).into(),
    ));
    Some((inl, rewind, rewind + link_end))
}

fn validate_protocol(protocol: &str, contents: &str, cursor: usize) -> bool {
    let size = contents.len();
    let bytes = contents.as_bytes();
    let mut rewind = 0;

    while rewind < cursor && isalpha(bytes[cursor - rewind - 1]) {
        rewind += 1;
    }

    size - cursor + rewind >= protocol.len() && &contents[cursor - rewind..cursor] == protocol
}

pub fn www_match<'a>(
    subject: &mut Subject<'a, '_, '_, '_, '_, '_>,
) -> Option<(Node<'a>, usize, usize)> {
    const WWW_DELIMS: [bool; 256] = character_set!(b"*_~([");
    let i = subject.scanner.pos;
    let relaxed_autolinks = subject.options.parse.relaxed_autolinks;
    let bytes = subject.input.as_bytes();

    if i < subject.relaxed_autolink_dead_zone_end {
        return None;
    }

    if i > 0 && !isspace(bytes[i - 1]) && !WWW_DELIMS[bytes[i - 1] as usize] {
        return None;
    }

    if !subject.input[i..].starts_with("www.") {
        return None;
    }

    if !domain_delim_ok(&subject.input, i + 4, relaxed_autolinks, relaxed_autolinks) {
        return None;
    }

    let link_end = autolink_extent(subject, i, 4)? - i;
    let link_end = autolink_delim(&subject.input[i..], link_end, relaxed_autolinks);
    debug_assert!(valid_domain(
        host(&subject.input[i + 4..i + link_end.max(4)]),
        relaxed_autolinks
    ));

    let mut url = "http://".to_string();
    url.push_str(&subject.input[i..link_end + i]);

    let inl = make_inline(
        subject.arena,
        NodeValue::Link(Box::new(NodeLink {
            url,
            title: String::new(),
        })),
        (0, 1, 0, 1).into(),
    );

    inl.append(make_inline(
        subject.arena,
        NodeValue::Text(subject.input[i..link_end + i].to_string().into()),
        (0, 1, 0, 1).into(),
    ));
    Some((inl, 0, link_end))
}

const MAX_DOMAIN_DOTS: usize = 10;

fn host(data: &str) -> &str {
    let len = data
        .char_indices()
        .find(|&(i, c)| {
            !(c == '\\' && i + 1 < data.len()
                || c == '_'
                || c == '.'
                || c == '-'
                || is_valid_hostchar(c))
        })
        .map_or(data.len(), |(i, _)| i);
    &data[..len]
}

fn valid_domain(host: &str, allow_short: bool) -> bool {
    let mut dots = 0;
    let mut uscores = 0;
    let mut prev_uscores = 0;

    for c in host.bytes() {
        match c {
            b'_' => uscores += 1,
            b'.' => {
                prev_uscores = uscores;
                uscores = 0;
                dots += 1;
            }
            _ => {}
        }
    }

    dots > MAX_DOMAIN_DOTS || (uscores == 0 && prev_uscores == 0 && (allow_short || dots > 0))
}

fn domain_delim_ok(
    input: &str,
    start: usize,
    allow_short: bool,
    relaxed_autolinks: bool,
) -> bool {
    let host = host(&input[start..]);
    let valid = valid_domain(host, allow_short);
    let stripped = host.trim_end_matches(['.', '_']);
    if stripped.len() == host.len() {
        return valid;
    }

    let stripped_valid = valid_domain(stripped, allow_short);
    if valid != stripped_valid
        && delim_reaches(input.as_bytes(), start + host.len(), relaxed_autolinks)
    {
        stripped_valid
    } else {
        valid
    }
}

fn delim_reaches(bytes: &[u8], mut pos: usize, relaxed_autolinks: bool) -> bool {
    loop {
        let Some(&b) = bytes.get(pos) else {
            return true;
        };
        match b {
            b'<' => return true,
            _ if isspace(b) => return true,
            _ if LINK_END_ASSORTMENT[b as usize] || b == b';' || b == b')' => pos += 1,
            b']' | b'}' if relaxed_autolinks => pos += 1,
            b'&' => {
                let name = bytes[pos + 1..].iter().take_while(|&&b| isalpha(b)).count();
                if name == 0 || bytes.get(pos + 1 + name) != Some(&b';') {
                    return false;
                }
                pos += name + 2;
            }
            _ => return false,
        }
    }
}

fn autolink_extent(
    subject: &mut Subject<'_, '_, '_, '_, '_, '_>,
    i: usize,
    skip: usize,
) -> Option<usize> {
    if subject.autolink_extent.contains(&i) {
        return Some(subject.autolink_extent.end);
    }

    let bytes = subject.input.as_bytes();
    let mut end = i + skip;
    while end < bytes.len() && !isspace(bytes[end]) {
        // basic test to detect whether we're in a normal markdown link - not exhaustive
        if subject.options.parse.relaxed_autolinks && bytes[end - 1] == b']' && bytes[end] == b'(' {
            subject.relaxed_autolink_dead_zone_end = end;
            return None;
        }
        end += 1;
    }

    subject.autolink_extent = i..end;
    Some(end)
}

fn is_valid_hostchar(ch: char) -> bool {
    !(ch.is_whitespace() || ch.is_punctuation() || ch.is_symbol())
}

const LINK_END_ASSORTMENT: [bool; 256] = character_set!(b"?!.,:*_~'\"");

fn autolink_delim(data: &str, mut link_end: usize, relaxed_autolinks: bool) -> usize {
    // \u{2069} (Pop Directional Isolate)
    const LINK_END_UNICODE: [u8; 3] = [0xe2, 0x81, 0xa9];

    let bytes = data.as_bytes();
    let mut parentheses = (0, 0);
    let mut brackets = (0, 0);
    let mut braces = (0, 0);
    for (i, &b) in bytes.iter().enumerate().take(link_end) {
        if b == b'<' {
            link_end = i;
            break;
        }

        match b {
            b'(' => parentheses.0 += 1,
            b')' => parentheses.1 += 1,
            b'[' => brackets.0 += 1,
            b']' => brackets.1 += 1,
            b'{' => braces.0 += 1,
            b'}' => braces.1 += 1,
            _ => (),
        }
    }

    while link_end > 0 {
        let cclose = bytes[link_end - 1];
        let counts = match cclose {
            b')' => Some(parentheses),
            b']' if relaxed_autolinks => Some(brackets),
            b'}' if relaxed_autolinks => Some(braces),
            _ => None,
        };

        if LINK_END_ASSORTMENT[cclose as usize] {
            link_end -= 1;
        } else if cclose == b';' {
            let mut new_end = link_end - 2;

            while new_end > 0 && isalpha(bytes[new_end]) {
                new_end -= 1;
            }

            if new_end < link_end - 2 && bytes[new_end] == b'&' {
                link_end = new_end;
            } else {
                link_end -= 1;
            }
        } else if let Some((opening, closing)) = counts {
            if closing <= opening {
                break;
            }

            match cclose {
                b')' => parentheses.1 -= 1,
                b']' => brackets.1 -= 1,
                b'}' => braces.1 -= 1,
                _ => unreachable!(),
            }
            link_end -= 1;
        } else if cclose == LINK_END_UNICODE[2] {
            let slice = &bytes[link_end - LINK_END_UNICODE.len()..link_end];

            if slice == LINK_END_UNICODE {
                link_end -= LINK_END_UNICODE.len();
                break;
            }

            break;
        } else {
            break;
        }
    }

    link_end
}

pub fn url_match<'a>(
    subject: &mut Subject<'a, '_, '_, '_, '_, '_>,
) -> Option<(Node<'a>, usize, usize)> {
    const SCHEMES: [&str; 3] = ["http", "https", "ftp"];

    let i = subject.scanner.pos;
    let relaxed_autolinks = subject.options.parse.relaxed_autolinks;
    let bytes = subject.input.as_bytes();
    let size = subject.input.len();

    if i < subject.relaxed_autolink_dead_zone_end {
        return None;
    }

    if size - i < 4 || bytes[i + 1] != b'/' || bytes[i + 2] != b'/' {
        return None;
    }

    let mut rewind = 0;
    while rewind < i && isalpha(bytes[i - rewind - 1]) {
        rewind += 1;
    }

    if !relaxed_autolinks {
        let scheme = &subject.input[i - rewind..i];
        let cond = |s: &&str| size - i + rewind >= s.len() && scheme.eq_ignore_ascii_case(s);
        if !SCHEMES.iter().any(cond) {
            return None;
        }

        if !subject.input[i + 3..]
            .chars()
            .next()
            .is_some_and(is_valid_hostchar)
        {
            return None;
        }
    }

    if !domain_delim_ok(&subject.input, i + 3, true, relaxed_autolinks) {
        return None;
    }

    let link_end = autolink_extent(subject, i, 3)? - i;
    let link_end = autolink_delim(&subject.input[i..], link_end, relaxed_autolinks);
    debug_assert!(valid_domain(
        host(&subject.input[i + 3..i + link_end.max(3)]),
        true
    ));

    let url = &subject.input[i - rewind..i + link_end];
    let inl = make_inline(
        subject.arena,
        NodeValue::Link(Box::new(NodeLink {
            url: url.to_string(),
            title: String::new(),
        })),
        (0, 1, 0, 1).into(),
    );

    inl.append(make_inline(
        subject.arena,
        NodeValue::Text(url.to_string().into()),
        (0, 1, 0, 1).into(),
    ));
    Some((inl, rewind, rewind + link_end))
}

#[cfg(test)]
mod tests {
    use super::find_email_autolink;
    use crate::Arena;

    #[test]
    fn email_autolink_bracket_depth() {
        let cases = [
            ("foo@example.com", false, Some((0, 15))),
            ("☃ foo@example.com", false, Some((4, 15))),
            ("[foo@example.com]", false, None),
            ("[[foo@example.com]] bar@example.com", false, Some((20, 15))),
            ("[foo@example.com]", true, Some((1, 15))),
            ("]][[foo@example.com", false, Some((4, 15))),
            ("[foo@example.com", false, None),
            ("[foo@example.com", true, Some((1, 15))),
            ("no email or brackets", false, None),
        ];

        for (contents, relaxed, expected) in cases {
            let arena = Arena::new();
            let actual = find_email_autolink(&arena, contents, relaxed, |_, _| true)
                .map(|(_, before_len, skip)| (before_len, skip));
            assert_eq!(actual, expected, "{contents:?}, relaxed: {relaxed}");
        }
    }
}
