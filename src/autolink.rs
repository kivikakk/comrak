use unicode_categories::UnicodeCategories;
use std::iter::FromIterator;
use {Node, AstCell, NodeValue, BTreeSet, Arena, NodeLink};
use ctype::{isspace, isalpha};
use inlines::make_inline;

pub fn process_autolinks<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                             node: &'a Node<'a, AstCell>,
                             contents: &mut Vec<char>) {
    let len = contents.len();
    let mut i = 0;

    while i < len {
        let mut post_org = None;

        while i < len {
            match contents[i] {
                ':' => (),
                'w' => {
                    post_org = www_match(arena, &contents, i);
                    if post_org.is_some() {
                        break;
                    }
                }
                '@' => (),
                _ => (),
            }
            i += 1;
        }

        match post_org {
            Some((post, skip)) => {
                node.insert_after(post);
                if i + skip < len {
                    let remain = contents[i + skip..].to_vec();
                    assert!(remain.len() > 0);
                    post.insert_after(make_inline(arena, NodeValue::Text(remain)));
                }
                contents.truncate(i);
                return;
            }
            None => (),
        }
    }
}

fn www_match<'a>(arena: &'a Arena<Node<'a, AstCell>>,
                 contents: &[char],
                 i: usize)
                 -> Option<(&'a Node<'a, AstCell>, usize)> {
    lazy_static! {
        static ref WWW_DELIMS: BTreeSet<char> = BTreeSet::from_iter(vec![
                                                                    '*', '_', '~', '(', '[']);
    }

    if i > 0 && !isspace(&contents[i - 1]) && !WWW_DELIMS.contains(&contents[i - 1]) {
        return None;
    }

    if contents.len() - i < 4 || &contents[i..i + 4] != &['w', 'w', 'w', '.'] {
        return None;
    }

    let mut link_end = match check_domain(&contents[i..]) {
        None => return None,
        Some(link_end) => link_end,
    };

    while i + link_end < contents.len() && !isspace(&contents[i + link_end]) {
        link_end += 1;
    }

    let link_end = autolink_delim(&contents[i..], link_end);

    let mut url = vec!['h', 't', 't', 'p', ':', '/', '/'];
    url.extend_from_slice(&contents[i..link_end + i]);

    let inl = make_inline(arena,
                          NodeValue::Link(NodeLink {
                              url: url,
                              title: vec![],
                          }));

    inl.append(make_inline(arena, NodeValue::Text(contents[i..link_end + i].to_vec())));
    Some((inl, link_end))
}

fn check_domain(data: &[char]) -> Option<usize> {
    let mut i = 1;
    let mut np = 0;
    let mut uscore1 = 0;
    let mut uscore2 = 0;

    while i < data.len() - 1 {
        if data[i] == '_' {
            uscore2 += 1;
        } else if data[i] == '.' {
            uscore1 = uscore2;
            uscore2 = 0;
            np += 1;
        } else if !is_valid_hostchar(data[i]) && data[i] != '-' {
            break;
        }
        i += 1;
    }

    if uscore1 > 0 || uscore2 > 0 {
        return None;
    }

    if np > 0 {
        return Some(i);
    }

    None
}

fn is_valid_hostchar(ch: char) -> bool {
    return !ch.is_whitespace() && !ch.is_punctuation();
}

fn autolink_delim(data: &[char], mut link_end: usize) -> usize {
    lazy_static! {
        static ref LINK_END_ASSORTMENT: BTreeSet<char> = BTreeSet::from_iter(
            vec!['?', '!', '.', ',', ':', '*', '_', '~', '\'', '"']);
    }

    for i in 0..link_end {
        if data[i] == '<' {
            link_end = i;
            break;
        }
    }

    while link_end > 0 {
        let cclose = data[link_end - 1];

        let copen = match cclose {
            ')' => Some('('),
            _ => None,
        };

        if LINK_END_ASSORTMENT.contains(&cclose) {
            link_end -= 1;
        } else if cclose == ';' {
            let mut new_end = link_end - 2;

            while new_end > 0 && isalpha(&data[new_end]) {
                new_end -= 1;
            }

            if new_end < link_end - 2 && data[new_end] == '&' {
                link_end = new_end;
            } else {
                link_end -= 1;
            }
        } else if let Some(copen) = copen {
            let mut opening = 0;
            let mut closing = 0;
            for i in 0..link_end {
                if data[i] == copen {
                    opening += 1;
                } else if data[i] == cclose {
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
