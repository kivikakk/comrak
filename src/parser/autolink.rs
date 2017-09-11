use ctype::{isspace, isalpha, isalnum};
use nodes::{NodeValue, NodeLink, AstNode};
use parser::inlines::make_inline;
use typed_arena::Arena;
use unicode_categories::UnicodeCategories;

pub fn process_autolinks<'a>(
    arena: &'a Arena<AstNode<'a>>,
    node: &'a AstNode<'a>,
    contents: &mut String,
) {
    let len = contents.len();
    let mut i = 0;

    while i < len {
        let mut post_org = None;

        while i < len {
            match contents.as_bytes()[i] {
                b':' => {
                    post_org = url_match(arena, contents, i);
                    if post_org.is_some() {
                        break;
                    }
                }
                b'w' => {
                    post_org = www_match(arena, contents, i);
                    if post_org.is_some() {
                        break;
                    }
                }
                b'@' => {
                    post_org = email_match(arena, contents, i);
                    if post_org.is_some() {
                        break;
                    }
                }
                _ => (),
            }
            i += 1;
        }

        if let Some((post, reverse, skip)) = post_org {
            i -= reverse;
            node.insert_after(post);
            if i + skip < len {
                let remain = contents[i + skip..].to_string();
                assert!(!remain.is_empty());
                post.insert_after(make_inline(arena, NodeValue::Text(remain)));
            }
            contents.truncate(i);
            return;
        }
    }
}

fn www_match<'a>(
    arena: &'a Arena<AstNode<'a>>,
    contents: &str,
    i: usize,
) -> Option<(&'a AstNode<'a>, usize, usize)> {
    lazy_static! {
        static ref WWW_DELIMS: [bool; 256] = {
            let mut sc = [false; 256];
            for c in &[b'*', b'_', b'~', b'(', b'['] {
                sc[*c as usize] = true;
            }
            sc
        };
    }

    if i > 0 && !isspace(contents.as_bytes()[i - 1]) &&
        !WWW_DELIMS[contents.as_bytes()[i - 1] as usize]
    {
        return None;
    }

    if !contents[i..].starts_with("www.") {
        return None;
    }

    let mut link_end = match check_domain(&contents[i..]) {
        None => return None,
        Some(link_end) => link_end,
    };

    while i + link_end < contents.len() && !isspace(contents.as_bytes()[i + link_end]) {
        link_end += 1;
    }

    link_end = autolink_delim(&contents[i..], link_end);

    let mut url = "http://".to_string();
    url += &contents[i..link_end + i];

    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url: url,
            title: String::new(),
        }),
    );

    inl.append(make_inline(
        arena,
        NodeValue::Text(contents[i..link_end + i].to_string()),
    ));
    Some((inl, 0, link_end))
}

fn check_domain(data: &str) -> Option<usize> {
    let mut np = 0;
    let mut uscore1 = 0;
    let mut uscore2 = 0;

    for (i, c) in data.char_indices() {
        if c == '_' {
            uscore2 += 1;
        } else if c == '.' {
            uscore1 = uscore2;
            uscore2 = 0;
            np += 1;
        } else if !is_valid_hostchar(c) && c != '-' {
            if uscore1 == 0 && uscore2 == 0 && np > 0 {
                return Some(i);
            }
            return None;
        }
    }

    if uscore1 == 0 && uscore2 == 0 && np > 0 {
        Some(data.len())
    } else {
        None
    }
}

fn is_valid_hostchar(ch: char) -> bool {
    !ch.is_whitespace() && !ch.is_punctuation()
}

fn autolink_delim(data: &str, mut link_end: usize) -> usize {
    lazy_static! {
        static ref LINK_END_ASSORTMENT: [bool; 256] = {
            let mut sc = [false; 256];
            for c in &[b'?', b'!', b'.', b',', b':', b'*', b'_', b'~', b'\'', b'"'] {
                sc[*c as usize] = true;
            }
            sc
        };
    }

    for i in 0..link_end {
        if data.as_bytes()[i] == b'<' {
            link_end = i;
            break;
        }
    }

    while link_end > 0 {
        let cclose = data.as_bytes()[link_end - 1];

        let copen = if cclose == b')' { Some(b'(') } else { None };

        if LINK_END_ASSORTMENT[cclose as usize] {
            link_end -= 1;
        } else if cclose == b';' {
            let mut new_end = link_end - 2;

            while new_end > 0 && isalpha(data.as_bytes()[new_end]) {
                new_end -= 1;
            }

            if new_end < link_end - 2 && data.as_bytes()[new_end] == b'&' {
                link_end = new_end;
            } else {
                link_end -= 1;
            }
        } else if let Some(copen) = copen {
            let mut opening = 0;
            let mut closing = 0;
            for i in 0..link_end {
                if data.as_bytes()[i] == copen {
                    opening += 1;
                } else if data.as_bytes()[i] == cclose {
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

fn url_match<'a>(
    arena: &'a Arena<AstNode<'a>>,
    contents: &str,
    i: usize,
) -> Option<(&'a AstNode<'a>, usize, usize)> {
    lazy_static! {
        static ref SCHEMES: Vec<&'static str> =
            vec!["http", "https", "ftp"];
    }

    let size = contents.len();

    if size - i < 4 || contents.as_bytes()[i + 1] != b'/' || contents.as_bytes()[i + 2] != b'/' {
        return None;
    }

    let mut rewind = 0;
    while rewind < i && isalpha(contents.as_bytes()[i - rewind - 1]) {
        rewind += 1;
    }

    let cond = |s: &&str| size - i + rewind >= s.len() && &&contents[i - rewind..i] == s;
    if !SCHEMES.iter().any(cond) {
        return None;
    }

    let mut link_end = match check_domain(&contents[i + 3..]) {
        None => return None,
        Some(link_end) => link_end,
    };

    while link_end < size - i && !isspace(contents.as_bytes()[i + link_end]) {
        link_end += 1;
    }

    link_end = autolink_delim(&contents[i..], link_end);

    let url = contents[i - rewind..i + link_end].to_string();
    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url: url.clone(),
            title: String::new(),
        }),
    );

    inl.append(make_inline(arena, NodeValue::Text(url)));
    Some((inl, rewind, rewind + link_end))
}

fn email_match<'a>(
    arena: &'a Arena<AstNode<'a>>,
    contents: &str,
    i: usize,
) -> Option<(&'a AstNode<'a>, usize, usize)> {
    lazy_static! {
        static ref EMAIL_OK_SET: [bool; 256] = {
            let mut sc = [false; 256];
            for c in &[b'.', b'+', b'-', b'_'] {
                sc[*c as usize] = true;
            }
            sc
        };
    }

    let size = contents.len();

    let mut rewind = 0;
    let mut ns = 0;

    while rewind < i {
        let c = contents.as_bytes()[i - rewind - 1];

        if isalnum(c) || EMAIL_OK_SET[c as usize] {
            rewind += 1;
            continue;
        }

        if c == b'/' {
            ns += 1;
        }

        break;
    }

    if rewind == 0 || ns > 0 {
        return None;
    }

    let mut link_end = 0;
    let mut nb = 0;
    let mut np = 0;

    while link_end < size - i {
        let c = contents.as_bytes()[i + link_end];

        if isalnum(c) {
            // empty
        } else if c == b'@' {
            nb += 1;
        } else if c == b'.' && link_end < size - i - 1 {
            np += 1;
        } else if c != b'-' && c != b'_' {
            break;
        }

        link_end += 1;
    }

    if link_end < 2 || nb != 1 || np == 0 ||
        (!isalpha(contents.as_bytes()[i + link_end - 1]) &&
             contents.as_bytes()[i + link_end - 1] != b'.')
    {
        return None;
    }

    link_end = autolink_delim(&contents[i..], link_end);

    let mut url = "mailto:".to_string();
    url += &contents[i - rewind..link_end + i];

    let inl = make_inline(
        arena,
        NodeValue::Link(NodeLink {
            url: url,
            title: String::new(),
        }),
    );

    inl.append(make_inline(
        arena,
        NodeValue::Text(
            contents[i - rewind..link_end + i].to_string(),
        ),
    ));
    Some((inl, rewind, rewind + link_end))
}
