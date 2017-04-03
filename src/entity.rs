use std::cmp::{min, Ordering};
use std::char;
use ::{isdigit, entity_data};

fn isxdigit(ch: &char) -> bool {
    "0123456789abcdefABCDEF".find(*ch).is_some()
}

pub fn unescape(text: &[char]) -> Option<(Vec<char>, usize)> {
    if text.len() >= 3 && text[0] == '#' {
        let mut codepoint: u32 = 0;
        let mut i = 0;

        let num_digits = if isdigit(&text[1]) {
            i = 1;
            while i < text.len() && isdigit(&text[i]) {
                codepoint = (codepoint * 10) + (text[i] as u32 - '0' as u32);
                codepoint = min(codepoint, 0x110000);
                i += 1;
            }
            i - 1
        } else if text[1] == 'x' || text[1] == 'X' {
            i = 2;
            while i < text.len() && isxdigit(&text[i]) {
                codepoint = (codepoint * 16) + ((text[i] as u32 | 32) % 39 - 9);
                codepoint = min(codepoint, 0x110000);
                i += 1;
            }
            i - 2
        } else {
            0
        };

        if num_digits >= 1 && num_digits <= 8 && i < text.len() && text[i] == ';' {
            if codepoint == 0 || (codepoint >= 0xD800 && codepoint <= 0xE000) ||
               codepoint >= 0x110000 {
                codepoint = 0xFFFD;
            }
            return Some((vec![char::from_u32(codepoint).unwrap_or('\u{FFFD}')], i + 1));
        }
    }

    let size = min(text.len(), entity_data::MAX_LENGTH);
    for i in entity_data::MIN_LENGTH..size {
        if text[i] == ' ' {
            return None;
        }

        if text[i] == ';' {
            return lookup(&text[..i].into_iter().collect::<String>()).map(|e| (e, i + 1));
        }
    }

    None
}

fn lookup(text: &str) -> Option<Vec<char>> {
    let mut i = entity_data::ENTITIES.len() / 2;
    let mut low = 0;
    let mut high = entity_data::ENTITIES.len() - 1;

    loop {
        let cmp = text.cmp(entity_data::ENTITIES[i].0);
        if cmp == Ordering::Equal {
            return Some(entity_data::ENTITIES[i].1.to_vec());
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
