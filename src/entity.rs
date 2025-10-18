use entities::ENTITIES;
use std::borrow::Cow;
use std::char;
use std::cmp::min;
use std::str;

use crate::ctype::isdigit;

pub const ENTITY_MIN_LENGTH: usize = 2;
pub const ENTITY_MAX_LENGTH: usize = 32;

fn isxdigit(ch: u8) -> bool {
    (ch >= b'0' && ch <= b'9') || (ch >= b'a' && ch <= b'f') || (ch >= b'A' && ch <= b'F')
}

pub fn unescape(text: &str) -> Option<(Cow<'static, str>, usize)> {
    let bytes = text.as_bytes();
    if text.len() >= 3 && bytes[0] == b'#' {
        let mut codepoint: u32 = 0;
        let mut i = 0;

        let num_digits = if isdigit(bytes[1]) {
            i = 1;
            while i < text.len() && isdigit(bytes[i]) {
                codepoint = (codepoint * 10) + (bytes[i] as u32 - '0' as u32);
                codepoint = min(codepoint, 0x11_0000);
                i += 1;
            }
            i - 1
        } else if bytes[1] == b'x' || bytes[1] == b'X' {
            i = 2;
            while i < bytes.len() && isxdigit(bytes[i]) {
                codepoint = (codepoint * 16) + ((bytes[i] as u32 | 32) % 39 - 9);
                codepoint = min(codepoint, 0x11_0000);
                i += 1;
            }
            i - 2
        } else {
            0
        };

        if i < bytes.len()
            && bytes[i] == b';'
            && (((bytes[1] == b'x' || bytes[1] == b'X') && (1..=6).contains(&num_digits))
                || (1..=7).contains(&num_digits))
        {
            if codepoint == 0 || (0xD800..=0xE000).contains(&codepoint) || codepoint >= 0x110000 {
                codepoint = 0xFFFD;
            }
            return Some((
                char::from_u32(codepoint)
                    .unwrap_or('\u{FFFD}')
                    .to_string()
                    .into(),
                i + 1,
            ));
        }
    }

    let size = min(text.len(), ENTITY_MAX_LENGTH);
    for i in ENTITY_MIN_LENGTH..size {
        if bytes[i] == b' ' {
            return None;
        }

        if bytes[i] == b';' {
            return lookup(&text[..i]).map(|e| (e.into(), i + 1));
        }
    }

    None
}

fn lookup(text: &str) -> Option<&'static str> {
    ENTITIES
        .iter()
        .find(|e| {
            e.entity.starts_with("&")
                && e.entity.ends_with(";")
                && &e.entity[1..e.entity.len() - 1] == text
        })
        .map(|e| e.characters)
}

pub fn unescape_html(src: &str) -> Cow<'_, str> {
    let bytes = src.as_bytes();
    let size = src.len();
    let mut i = 0;
    let mut v = String::with_capacity(size);

    while i < size {
        let org = i;
        while i < size && bytes[i] != b'&' {
            i += 1;
        }

        if i > org {
            if org == 0 && i >= size {
                return src.into();
            }

            v.push_str(&src[org..i]);
        }

        if i >= size {
            return v.into();
        }

        i += 1;
        match unescape(&src[i..]) {
            Some((chs, size)) => {
                v.push_str(&chs);
                i += size;
            }
            None => v.push('&'),
        }
    }

    v.into()
}
