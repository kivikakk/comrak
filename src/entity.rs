use ctype::isdigit;
use entity_data;
use std::char;
use std::cmp::{min, Ordering};

use entities::ENTITIES;

fn isxdigit(ch: &u8) -> bool {
    (*ch >= b'0' && *ch <= b'9') || (*ch >= b'a' && *ch <= b'f') || (*ch >= b'A' && *ch <= b'F')
}

pub fn unescape(text: &str) -> Option<(String, usize)> {
    if text.len() >= 3 && text.as_bytes()[0] == b'#' {
        let mut codepoint: u32 = 0;
        let mut i = 0;

        let num_digits = if isdigit(text.as_bytes()[1]) {
            i = 1;
            while i < text.len() && isdigit(text.as_bytes()[i]) {
                codepoint = (codepoint * 10) + (text.as_bytes()[i] as u32 - '0' as u32);
                codepoint = min(codepoint, 0x110000);
                i += 1;
            }
            i - 1
        } else if text.as_bytes()[1] == b'x' || text.as_bytes()[1] == b'X' {
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
            return Some((char::from_u32(codepoint)
                             .unwrap_or('\u{FFFD}')
                             .to_string(),
                         i + 1));
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

fn lookup(text: &str) -> Option<&str> {
    let entity_str = format!("&{};", text);

    let entity = ENTITIES
        .iter()
        .find(|e| e.entity == entity_str);

    match entity {
        Some(e) => {
            Some(e.characters)
        }
        None => None
    }
}

pub fn unescape_html(src: &str) -> String {
    let size = src.len();
    let mut i = 0;
    let mut v = String::with_capacity(size);

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
