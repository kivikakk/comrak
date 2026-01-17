use std::borrow::Cow;
use std::str;
use unicode_categories::UnicodeCategories;

use crate::Arena;
use crate::character_set::character_set;
use crate::ctype::{isalnum, isalpha, isspace};
use crate::nodes::{Node, NodeLink, NodeValue, Sourcepos};
use crate::parser::Spx;
use crate::parser::inlines::{Subject, make_inline};

pub(crate) fn process_email_autolinks<'a>(
    arena: &'a Arena<'a>,
    node: Node<'a>,
    contents: &mut Cow<'static, str>,
    relaxed_autolinks: bool,
    sourcepos: &mut Sourcepos,
    spx: &mut Spx,
) {
    let bytes = contents.as_bytes();
    let len = contents.len();
    let mut i = 0;

    while i < len {
        let mut post_org = None;
        let mut bracket_opening = 0;

        // cmark-gfm ignores links inside brackets, such as `[[http://example.com]`
        while i < len {
            if !relaxed_autolinks {
                match bytes[i] {
                    b'[' => {
                        bracket_opening += 1;
                    }
                    b']' => {
                        bracket_opening -= 1;
                    }
                    _ => (),
                }

                if bracket_opening > 0 {
                    i += 1;
                    continue;
                }
            }

            if bytes[i] == b'@' {
                post_org = email_match(arena, contents, i, relaxed_autolinks);
                if post_org.is_some() {
                    break;
                }
            }
            i += 1;
        }

        if let Some((post, reverse, skip)) = post_org {
            i -= reverse;
            node.insert_after(post);

            let remain = if i + skip < len {
                let remain = &contents[i + skip..];
                assert!(!remain.is_empty());
                Some(remain.to_string())
            } else {
                None
            };
            let initial_end_col = sourcepos.end.column;

            sourcepos.end.column = spx.consume(i);

            let nsp_end_col = spx.consume(skip);

            contents.to_mut().truncate(i);

            let nsp: Sourcepos = (
                sourcepos.end.line,
                sourcepos.end.column + 1,
                sourcepos.end.line,
                nsp_end_col,
            )
                .into();
            post.data_mut().sourcepos = nsp;
            // Inner text gets same sourcepos as link, since there's nothing but
            // the text.
            post.first_child().unwrap().data_mut().sourcepos = nsp;

            if let Some(remain) = remain {
                let mut asp: Sourcepos = (
                    sourcepos.end.line,
                    nsp.end.column + 1,
                    sourcepos.end.line,
                    initial_end_col,
                )
                    .into();
                let after = make_inline(arena, NodeValue::Text(remain.into()), asp);
                post.insert_after(after);

                let after_ast = &mut after.data_mut();
                let NodeValue::Text(ref mut text) = after_ast.value else {
                    unreachable!();
                };
                process_email_autolinks(arena, after, text, relaxed_autolinks, &mut asp, spx);
                after_ast.sourcepos = asp;
            }

            return;
        }
    }
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

    if i > 0 && !isspace(bytes[i - 1]) && !WWW_DELIMS[bytes[i - 1] as usize] {
        return None;
    }

    if !subject.input[i..].starts_with("www.") {
        return None;
    }

    // Skip over "www." for domain validation, but account for its length in the overall link
    let mut link_end = check_domain(&subject.input[i + 4..], relaxed_autolinks)? + 4;

    while i + link_end < subject.input.len() && !isspace(bytes[i + link_end]) {
        // basic test to detect whether we're in a normal markdown link - not exhaustive
        if relaxed_autolinks && bytes[i + link_end - 1] == b']' && bytes[i + link_end] == b'(' {
            return None;
        }
        link_end += 1;
    }

    link_end = autolink_delim(&subject.input[i..], link_end, relaxed_autolinks);

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

fn check_domain(data: &str, allow_short: bool) -> Option<usize> {
    let mut np = 0;
    let mut uscore1 = 0;
    let mut uscore2 = 0;

    for (i, c) in data.char_indices() {
        if c == '\\' && i < data.len() - 1 {
            // Ignore escaped characters per https://github.com/github/cmark-gfm/pull/292.
            // Not sure I love this, but it tracks upstream ..
        } else if c == '_' {
            uscore2 += 1;
        } else if c == '.' {
            uscore1 = uscore2;
            uscore2 = 0;
            np += 1;
        } else if !is_valid_hostchar(c) && c != '-' {
            if uscore1 == 0 && uscore2 == 0 && (allow_short || np > 0) {
                return Some(i);
            }
            return None;
        }
    }

    if (uscore1 > 0 || uscore2 > 0) && np <= 10 {
        None
    } else if allow_short || np > 0 {
        Some(data.len())
    } else {
        None
    }
}

fn is_valid_hostchar(ch: char) -> bool {
    !(ch.is_whitespace() || ch.is_punctuation() || ch.is_symbol())
}

fn autolink_delim(data: &str, mut link_end: usize, relaxed_autolinks: bool) -> usize {
    const LINK_END_ASSORTMENT: [bool; 256] = character_set!(b"?!.,:*_~'\"");
    // \u{2069} (Pop Directional Isolate)
    const LINK_END_UNICODE: [u8; 3] = [0xe2, 0x81, 0xa9];

    let bytes = data.as_bytes();
    for (i, &b) in bytes.iter().enumerate().take(link_end) {
        if b == b'<' {
            link_end = i;
            break;
        }
    }

    while link_end > 0 {
        let cclose = bytes[link_end - 1];

        // Allow any number of matching parentheses (as recognised in copen/cclose)
        // at the end of the URL.  If there is a greater number of closing
        // parentheses than opening ones, we remove one character from the end of
        // the link.
        let mut copen = if cclose == b')' { Some(b'(') } else { None };

        if relaxed_autolinks && copen.is_none() {
            // allow balancing of `[]` and `{}` just like `()`
            copen = if cclose == b']' {
                Some(b'[')
            } else if cclose == b'}' {
                Some(b'{')
            } else {
                None
            };
        }

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
        } else if let Some(copen) = copen {
            let mut opening = 0;
            let mut closing = 0;
            for &b in bytes.iter().take(link_end) {
                if b == copen {
                    opening += 1;
                } else if b == cclose {
                    closing += 1;
                }
            }

            if closing <= opening {
                break;
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

    if size - i < 4 || bytes[i + 1] != b'/' || bytes[i + 2] != b'/' {
        return None;
    }

    let mut rewind = 0;
    while rewind < i && isalpha(bytes[i - rewind - 1]) {
        rewind += 1;
    }

    if !relaxed_autolinks {
        let scheme = &subject.input[i - rewind..i];
        let cond = |s: &&str| size - i + rewind >= s.len() && &scheme == s;
        if !SCHEMES.iter().any(cond) {
            return None;
        }
    }

    let mut link_end = check_domain(&subject.input[i + 3..], relaxed_autolinks)? + 3;

    while link_end < size - i && !isspace(bytes[i + link_end]) {
        // basic test to detect whether we're in a normal markdown link - not exhaustive
        if relaxed_autolinks
            && link_end > 0
            && bytes[i + link_end - 1] == b']'
            && bytes[i + link_end] == b'('
        {
            return None;
        }
        link_end += 1;
    }

    link_end = autolink_delim(&subject.input[i..], link_end, relaxed_autolinks);

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
