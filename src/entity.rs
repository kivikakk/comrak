use std::cmp::{min, Ordering};
use std::char;
use ctype::isdigit;
use entity_data;

fn isxdigit(ch: &u8) -> bool {
    (*ch >= b'0' && *ch <= b'9') || (*ch >= b'a' && *ch <= b'f') ||
    (*ch >= b'A' && *ch <= b'F')
}

pub fn unescape(text: &str) -> Option<(String, usize)> {
    if text.len() >= 3 && text.as_bytes()[0] == b'#' {
        let mut codepoint: u32 = 0;
        let mut i = 0;

        let num_digits = if isdigit(&text.as_bytes()[1]) {
            i = 1;
            while i < text.len() && isdigit(&text.as_bytes()[i]) {
                codepoint = (codepoint * 10) + (text.as_bytes()[i] as u32 - '0' as u32);
                codepoint = min(codepoint, 0x110000);
                i += 1;
            }
            i - 1
        } else if text.as_bytes()[1] == b'x' ||
                                   text.as_bytes()[1] == b'X' {
            i = 2;
            while i < text.len() && isxdigit(&text.as_bytes()[i]) {
                codepoint = (codepoint * 16) + ((text.as_bytes()[i] as u32 | 32) % 39 - 9);
                codepoint = min(codepoint, 0x110000);
                i += 1;
            }
            i - 2
        } else {
            0
        };

        if num_digits >= 1 && num_digits <= 8 && i < text.len() && text.as_bytes()[i] == b';' {
            if codepoint == 0 || (codepoint >= 0xD800 && codepoint <= 0xE000) ||
               codepoint >= 0x110000 {
                codepoint = 0xFFFD;
            }
            return Some((char::from_u32(codepoint).unwrap_or('\u{FFFD}').to_string(), i + 1));
        }
    }

    let size = min(text.len(), entity_data::MAX_LENGTH);
    for i in entity_data::MIN_LENGTH..size {
        if text.as_bytes()[i] == b' ' {
            return None;
        }

        if text.as_bytes()[i] == b';' {
            return lookup(&text[..i]).map(|e| (e.to_string(), i + 1));
        }
    }

    None
}

fn lookup(text: &str) -> Option<&'static str> {
    let mut i = entity_data::ENTITIES.len() / 2;
    let mut low = 0;
    let mut high = entity_data::ENTITIES.len() - 1;

    loop {
        let cmp = text.cmp(entity_data::ENTITIES[i].0);
        if cmp == Ordering::Equal {
            return Some(entity_data::ENTITIES[i].1);
        } else if cmp == Ordering::Less && i > low {
            let mut j = i - ((i - low) / 2);
            if j == i {
                j -= 1;
            }
            high = i - 1;
            i = j;
        } else if cmp == Ordering::Greater && i < high {
            let mut j = i + ((high - i) / 2);
            if j == i {
                j += 1;
            }
            low = i + 1;
            i = j;
        } else {
            return None;
        }
    }
}

pub fn unescape_html(src: &str) -> String {
    let mut i = 0;
    let mut v = String::new();
    let size = src.len();

    while i < size {
        let org = i;
        while i < size && src.as_bytes()[i] != b'&' {
            i += 1;
        }

        if i > org {
            if org == 0 && i >= size {
                return src.to_string();
            }

            v += &src[org..i];
        }

        if i >= size {
            return v;
        }

        i += 1;
        match unescape(&src[i..]) {
            Some((chs, size)) => {
                v += &chs;
                i += size;
            }
            None => v.push('&'),
        }
    }

    v
}
