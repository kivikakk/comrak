use ::{ispunct, isspace, entity};
use inlines::AutolinkType;

pub fn unescape(v: &mut String) {
    let mut r = 0;
    let mut w = 0;
    let sz = v.len();

    while r < sz {
        if v[r] == '\\' && r + 1 < sz && ispunct(&v[r + 1]) {
            r += 1;
        }
        if r >= sz {
            break;
        }
        v[w] = v[r];
        w += 1;
        r += 1;
    }

    v.truncate(w);
}

pub fn clean_autolink(url: &str, kind: AutolinkType) -> String {
    let mut url_vec = url.to_vec();
    trim(&mut url_vec);

    if url_vec.len() == 0 {
        return url_vec;
    }

    let mut buf = vec![];
    if kind == AutolinkType::Email {
        buf.extend_from_slice(&['m', 'a', 'i', 'l', 't', 'o', ':']);
    }

    buf.extend_from_slice(&entity::unescape_html(&url_vec));
    buf
}

pub fn normalize_whitespace(v: &mut String) {
    let mut last_char_was_space = false;
    let mut r = 0;
    let mut w = 0;

    while r < v.len() {
        if isspace(&v[r]) {
            if !last_char_was_space {
                v[w] = ' ';
                w += 1;
                last_char_was_space = true;
            }
        } else {
            v[w] = v[r];
            w += 1;
            last_char_was_space = false;
        }
        r += 1;
    }

    v.truncate(w);
}

pub fn remove_trailing_blank_lines(line: &mut String) {
    let mut i = line.len() - 1;
    loop {
        let c = line[i];

        if c != ' ' && c != '\t' && !is_line_end_char(&c) {
            break;
        }

        if i == 0 {
            line.clear();
            return;
        }

        i -= 1;
    }

    for i in i..line.len() {
        let c = line[i];

        if !is_line_end_char(&c) {
            continue;
        }

        line.truncate(i);
        break;
    }
}

pub fn is_line_end_char(ch: &u8) -> bool {
    match ch {
        &10 | &13 => true,
        _ => false,
    }
}

pub fn is_space_or_tab(ch: &u8) -> bool {
    match ch {
        &9 | &32 => true,
        _ => false,
    }
}

pub fn chop_trailing_hashtags(line: &mut String) {
    rtrim(line);

    let orig_n = line.len() - 1;
    let mut n = orig_n;

    while line[n] == '#' {
        if n == 0 {
            return;
        }
        n -= 1;
    }

    if n != orig_n && is_space_or_tab(&line[n]) {
        line.truncate(n);
        rtrim(line);
    }
}

pub fn rtrim(line: &mut String) {
    let mut len = line.len();
    while len > 0 && isspace(&line[len - 1]) {
        line.pop();
        len -= 1;
    }
}

pub fn ltrim(line: &mut String) {
    let mut len = line.len();
    while len > 0 && isspace(&line[0]) {
        line.remove(0);
        len -= 1;
    }
}

pub fn trim(line: &mut String) {
    ltrim(line);
    rtrim(line);
}

pub fn trim_slice(mut i: &str) -> &str {
    let mut len = i.len();
    while len > 0 && isspace(&i[0]) {
        i = &i[1..];
        len -= 1;
    }
    while len > 0 && isspace(&i[len - 1]) {
        i = &i[..len - 1];
        len -= 1;
    }
    i
}

pub fn clean_url(url: &str) -> String {
    let url = trim_slice(url);

    let url_len = url.len();
    if url_len == 0 {
        return vec![];
    }

    let mut b = if url[0] == '<' && url[url_len - 1] == '>' {
        entity::unescape_html(&url[1..url_len - 1])
    } else {
        entity::unescape_html(url)
    };

    unescape(&mut b);
    b
}

pub fn clean_title(title: &str) -> String {
    let title_len = title.len();
    if title_len == 0 {
        return vec![];
    }

    let first = title[0];
    let last = title[title_len - 1];

    let mut b = if (first == '\'' && last == '\'') || (first == '(' && last == ')') ||
                   (first == '"' && last == '"') {
        entity::unescape_html(&title[1..title_len - 1])
    } else {
        entity::unescape_html(title)
    };

    unescape(&mut b);
    b
}

pub fn is_blank(s: &str) -> bool {
    for c in s {
        match *c {
            '\r' | '\n' => return true,
            ' ' | '\t' => (),
            _ => return false,
        }
    }
    true
}

pub fn normalize_reference_label(i: &str) -> String {
    let i = trim_slice(i);
    let mut v = vec![];
    let mut last_was_whitespace = false;
    for c in i {
        for e in c.to_lowercase() {
            if e.is_whitespace() {
                if !last_was_whitespace {
                    last_was_whitespace = true;
                    v.push(' ');
                }
            } else {
                last_was_whitespace = false;
                v.push(e);
            }
        }
    }
    v
}
