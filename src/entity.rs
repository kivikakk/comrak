use std::borrow::Cow;
use std::char;
use std::cmp::min;
use std::ops::Range;
use std::str;

use crate::scanners::{self, EntityType};

include!(concat!(env!("OUT_DIR"), "/entitydata.rs"));

fn lookup(text: &str) -> Option<&'static str> {
    entitydata::ENTITY_MAP.get(text).copied()
}

const MAX_CODEPOINT: u32 = 0x11_0000;
const UTF16_SURROGATE_RANGE: Range<u32> = 0xD800..0xE000;

fn dec_to_u32(c: u8) -> u32 {
    (c - b'0') as u32
}
fn hex_to_u32(c: u8) -> u32 {
    ((c | 32) % 39 - 9) as u32
}

pub fn unescape(text: &str) -> Option<(Cow<'static, str>, usize)> {
    let (mut codepoint, length) = match scanners::entity(text)? {
        (EntityType::Named, length, value) => {
            return lookup(value).map(|e| (e.into(), length));
        }
        (EntityType::Decimal, length, value) => {
            let codepoint = value
                .bytes()
                .fold(0, |acc, c| min(acc * 10 + dec_to_u32(c), MAX_CODEPOINT));
            (codepoint, length)
        }
        (EntityType::Hexadecimal, length, value) => {
            let codepoint = value
                .bytes()
                .fold(0, |acc, c| min(acc * 0x10 + hex_to_u32(c), MAX_CODEPOINT));
            (codepoint, length)
        }
    };

    if codepoint == 0 || UTF16_SURROGATE_RANGE.contains(&codepoint) {
        codepoint = 0xFFFD;
    }

    Some((
        char::from_u32(codepoint)
            .unwrap_or('\u{FFFD}')
            .to_string()
            .into(),
        length,
    ))
}

pub fn unescape_html(mut src: &str) -> String {
    let mut v = String::with_capacity(src.len());
    let mut p;

    while let Some(pos) = src.find('&') {
        (p, src) = src.split_at(pos);

        v.push_str(p);

        match unescape(src) {
            Some((entity, size)) => {
                v.push_str(&entity);
                src = &src[size..];
            }
            None => {
                v.push('&');
                src = &src[1..];
            }
        }
    }

    v.push_str(src);
    v
}
