use crate::character_set::character_set;
use crate::ctype::{isalnum, isalpha, isspace};
use crate::nodes::{AstNode, NodeLink, NodeValue, Sourcepos};
use crate::parser::inlines::make_inline;
use std::str;
use typed_arena::Arena;
use unicode_categories::UnicodeCategories;

// TODO: this can probably be cleaned up a lot. It used to handle all three of
// {url,www,email}_match, but now just the last of those. (This is still per
// upstream cmark-gfm, so it's not easily changed without breaking compat.)
pub(crate) fn process_email_autolinks<'a>(
    arena: &'a Arena<AstNode<'a>>,
    node: &'a AstNode<'a>,
    contents_str: &mut String,
    relaxed_autolinks: bool,
    sourcepos: &mut Sourcepos,
    spx: &[(Sourcepos, usize)],
) {
    let contents = contents_str.as_bytes();
    let len = contents.len();
    let mut i = 0;

    while i < len {
        let mut post_org = None;
        let mut bracket_opening = 0;

        // cmark-gfm ignores links inside brackets, such as `[[http://example.com]`
        while i < len {
            if !relaxed_autolinks {
                match contents[i] {
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

            if contents[i] == b'@' {
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
                let remain = str::from_utf8(&contents[i + skip..]).unwrap();
                assert!(!remain.is_empty());
                Some(remain.to_string())
            } else {
                None
            };
            let initial_end_col = sourcepos.end.column;

            // Sourcepos end column `e` of the original node (set by writing
            // to `*sourcepos`) determined by advancing through `spx` until `i`
            // bytes of input are seen.
            //
            // For each element `(sp, x)` in `spx`:
            // - if remaining `i` is greater than the byte count `x`,
            //     set `i -= x` and continue.
            // - if remaining `i` is equal to the byte count `x`,
            //     set `e = sp.end.column - 1` and finish.
            // - if remaining `i` is less than the byte count `x`,
            //     assert `sp.end.column - sp.start.column + 1 == x` (1),
            //     set `e = sp.start.column + i - 1` and finish.
            //
            // (1) If `x` doesn't equal the range covered between the start
            //     and end column, there's no way to determine sourcepos within
            //     the range. This is a bug if it happens; it suggests we've
            //     matched an email autolink with some smart punctuation in it,
            //     or worse.
            //
            // NOTE: a little iffy on the way I've added `- 1` --- what we
            // calculate here technically is the start column of the linked
            // portion, then adjusted. I think this should be robust, but needs
            // checking at edges.

            let mut rem_i = i;
            for &(sp, x) in spx {
                if rem_i > x {
                    rem_i -= x;
                } else if rem_i == x {
                    sourcepos.end.column = sp.end.column - 1;
                    rem_i = 0;
                    break;
                } else {
                    // rem_i < x
                    assert_eq!(sp.end.column - sp.start.column + 1, x);
                    sourcepos.end.column = sp.start.column + rem_i - 1;
                    rem_i = 0;
                    break;
                }
            }
            assert!(rem_i == 0);

            contents_str.truncate(i);
            let nsp: Sourcepos = (
                sourcepos.end.line,
                sourcepos.end.column + 1,
                sourcepos.end.line,
                sourcepos.end.column + skip,
            )
                .into();
            post.data.borrow_mut().sourcepos = nsp;
            // Inner text gets same sourcepos as link, since there's nothing but
            // the text.
            post.first_child().unwrap().data.borrow_mut().sourcepos = nsp;

            if let Some(remain) = remain {
                post.insert_after(make_inline(
                    arena,
                    NodeValue::Text(remain.to_string()),
                    (
                        sourcepos.end.line,
                        nsp.end.column + 1,
                        sourcepos.end.line,
                        initial_end_col,
                    )
                        .into(),
                ));
            }

            return;
        }
    }
}

fn email_match<'a>(
    arena: &'a Arena<AstNode<'a>>,
    contents: &[u8],
    i: usize,
    relaxed_autolinks: bool,
) -> Option<(&'a AstNode<'a>, usize, usize)> {
    const EMAIL_OK_SET: [bool; 256] = character_set!(b".+-_");

    let size = contents.len();

    let mut auto_mailto = true;
    let mut is_xmpp = false;
    let mut rewind = 0;

    while rewind < i {
        let c = contents[i - rewind - 1];

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
        let c = contents[i + link_end];

        if isalnum(c) {
            // empty
        } else if c == b'@' {
            return None;
        } else if c == b'.' && link_end < size - i - 1 && isalnum(contents[i + link_end + 1]) {
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
        || (!isalpha(contents[i + link_end - 1]) && contents[i + link_end - 1] != b'.')
    {
        return None;
    }

    link_end = autolink_delim(&contents[i..], link_end, relaxed_autolinks);
    if link_end == 0 {
        return None;
    }

    let mut url = if auto_mailto {
        "mailto:".to_string()
    } else {
        "".to_string()
    };
    let text = str::from_utf8(&contents[i - rewind..link_end + i]).unwrap();
    url.push_str(text);

    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url,
            title: String::new(),
        }),
        (0, 1, 0, 1).into(),
    );

    inl.append(make_inline(
        arena,
        NodeValue::Text(text.to_string()),
        (0, 1, 0, 1).into(),
    ));
    Some((inl, rewind, rewind + link_end))
}

fn validate_protocol(protocol: &str, contents: &[u8], cursor: usize) -> bool {
    let size = contents.len();
    let mut rewind = 0;

    while rewind < cursor && isalpha(contents[cursor - rewind - 1]) {
        rewind += 1;
    }

    size - cursor + rewind >= protocol.len()
        && &contents[cursor - rewind..cursor] == protocol.as_bytes()
}

pub fn www_match<'a>(
    arena: &'a Arena<AstNode<'a>>,
    contents: &[u8],
    i: usize,
    relaxed_autolinks: bool,
) -> Option<(&'a AstNode<'a>, usize, usize)> {
    const WWW_DELIMS: [bool; 256] = character_set!(b"*_~([");

    if i > 0 && !isspace(contents[i - 1]) && !WWW_DELIMS[contents[i - 1] as usize] {
        return None;
    }

    if !contents[i..].starts_with(b"www.") {
        return None;
    }

    let mut link_end = match check_domain(&contents[i..], false) {
        None => return None,
        Some(link_end) => link_end,
    };

    while i + link_end < contents.len() && !isspace(contents[i + link_end]) {
        // basic test to detect whether we're in a normal markdown link - not exhaustive
        if relaxed_autolinks && contents[i + link_end - 1] == b']' && contents[i + link_end] == b'('
        {
            return None;
        }
        link_end += 1;
    }

    link_end = autolink_delim(&contents[i..], link_end, relaxed_autolinks);

    let mut url = "http://".to_string();
    url.push_str(str::from_utf8(&contents[i..link_end + i]).unwrap());

    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url,
            title: String::new(),
        }),
        (0, 1, 0, 1).into(),
    );

    inl.append(make_inline(
        arena,
        NodeValue::Text(
            str::from_utf8(&contents[i..link_end + i])
                .unwrap()
                .to_string(),
        ),
        (0, 1, 0, 1).into(),
    ));
    Some((inl, 0, link_end))
}

fn check_domain(data: &[u8], allow_short: bool) -> Option<usize> {
    let mut np = 0;
    let mut uscore1 = 0;
    let mut uscore2 = 0;

    for (i, c) in unsafe { str::from_utf8_unchecked(data) }.char_indices() {
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

fn autolink_delim(data: &[u8], mut link_end: usize, relaxed_autolinks: bool) -> usize {
    const LINK_END_ASSORTMENT: [bool; 256] = character_set!(b"?!.,:*_~'\"");

    for (i, &b) in data.iter().enumerate().take(link_end) {
        if b == b'<' {
            link_end = i;
            break;
        }
    }

    while link_end > 0 {
        let cclose = data[link_end - 1];

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

            while new_end > 0 && isalpha(data[new_end]) {
                new_end -= 1;
            }

            if new_end < link_end - 2 && data[new_end] == b'&' {
                link_end = new_end;
            } else {
                link_end -= 1;
            }
        } else if let Some(copen) = copen {
            let mut opening = 0;
            let mut closing = 0;
            for &b in data.iter().take(link_end) {
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
        } else {
            break;
        }
    }

    link_end
}

pub fn url_match<'a>(
    arena: &'a Arena<AstNode<'a>>,
    contents: &[u8],
    i: usize,
    relaxed_autolinks: bool,
) -> Option<(&'a AstNode<'a>, usize, usize)> {
    const SCHEMES: [&[u8]; 3] = [b"http", b"https", b"ftp"];

    let size = contents.len();

    if size - i < 4 || contents[i + 1] != b'/' || contents[i + 2] != b'/' {
        return None;
    }

    let mut rewind = 0;
    while rewind < i && isalpha(contents[i - rewind - 1]) {
        rewind += 1;
    }

    if !relaxed_autolinks {
        let cond = |s: &&[u8]| size - i + rewind >= s.len() && &&contents[i - rewind..i] == s;
        if !SCHEMES.iter().any(cond) {
            return None;
        }
    }

    let mut link_end = match check_domain(&contents[i + 3..], true) {
        None => return None,
        Some(link_end) => link_end,
    };

    while link_end < size - i && !isspace(contents[i + link_end]) {
        // basic test to detect whether we're in a normal markdown link - not exhaustive
        if relaxed_autolinks
            && link_end > 0
            && contents[i + link_end - 1] == b']'
            && contents[i + link_end] == b'('
        {
            return None;
        }
        link_end += 1;
    }

    link_end = autolink_delim(&contents[i..], link_end, relaxed_autolinks);

    let url = str::from_utf8(&contents[i - rewind..i + link_end])
        .unwrap()
        .to_string();
    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url: url.clone(),
            title: String::new(),
        }),
        (0, 1, 0, 1).into(),
    );

    inl.append(make_inline(
        arena,
        NodeValue::Text(url),
        (0, 1, 0, 1).into(),
    ));
    Some((inl, rewind, rewind + link_end))
}
