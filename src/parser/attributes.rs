use std::mem;

use crate::nodes::Attributes;

pub fn parse_attributes(input: &str) -> Option<(Attributes, usize)> {
    let mut ci = input.char_indices();
    if ci.next()?.1 != '{' {
        return None;
    }

    enum State {
        Betwixt,
        Value(Kind, String, Quote),
        Key(String),
        PostQuote,
    }

    #[derive(Default)]
    enum Kind {
        #[default]
        Id,
        Class,
        Pair(String),
    }

    enum Quote {
        Bare,
        Quoted,
        QuotedEscaped,
    }

    let mut state = State::Betwixt;
    let mut attrs = Attributes::default();

    loop {
        let (i, c) = ci.next()?;

        loop {
            match state {
                State::Betwixt => match c {
                    '#' => {
                        state = State::Value(Kind::Id, String::new(), Quote::Bare);
                        break;
                    }
                    '.' => {
                        state = State::Value(Kind::Class, String::new(), Quote::Bare);
                        break;
                    }
                    'a'..='z' | 'A'..='Z' => {
                        state = State::Key(c.to_string());
                        break;
                    }
                    '}' => return Some((attrs, i + 1)),
                    ' ' | '\t' => break,
                    _ => return None,
                },
                State::Value(ref mut kind, ref mut value, Quote::Bare) => match c {
                    '"' if value.is_empty() => {
                        state = State::Value(mem::take(kind), mem::take(value), Quote::Quoted);
                        break;
                    }
                    // An id can contain [literally anything] in HTML5, just so long as it's non-empty.
                    // Consider looking at how Pandoc parses this part.
                    // Per above, classes are also literally anything, *except* ASCII whitespace.
                    //
                    // [literally anything]: https://html.spec.whatwg.org/multipage/dom.html#the-id-attribute
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | ':' | '.' => {
                        value.push(c);
                        break;
                    }
                    ' ' | '}' => {
                        match kind {
                            Kind::Id => {
                                if value.is_empty() {
                                    return None;
                                }
                                attrs.id = Some(mem::take(value));
                            }
                            Kind::Class => {
                                if value.is_empty() {
                                    return None;
                                }
                                attrs.classes.push(mem::take(value));
                            }
                            Kind::Pair(key) => attrs.pairs.push((mem::take(key), mem::take(value))),
                        }
                        state = State::Betwixt;
                        // handle in loop
                    }
                    _ => todo!(),
                },
                State::Value(ref mut kind, ref mut value, Quote::Quoted) => match c {
                    '"' => {
                        // XXX: duplicates above, wish it didn't.
                        match kind {
                            Kind::Id => attrs.id = Some(mem::take(value)),
                            Kind::Class => attrs.classes.push(mem::take(value)),
                            Kind::Pair(key) => attrs.pairs.push((mem::take(key), mem::take(value))),
                        }
                        state = State::PostQuote;
                        break;
                    }
                    '\\' => {
                        state =
                            State::Value(mem::take(kind), mem::take(value), Quote::QuotedEscaped);
                        break;
                    }
                    _ => {
                        value.push(c);
                        break;
                    }
                },
                State::Value(ref mut kind, ref mut value, Quote::QuotedEscaped) => {
                    value.push(c);
                    state = State::Value(mem::take(kind), mem::take(value), Quote::Quoted);
                    break;
                }
                State::Key(ref mut key) => match c {
                    // "Except where otherwise specified, attribute values on HTML elements may be any string value,
                    // including the empty string, and there is no restriction on what text can be specified in such
                    // attribute values."
                    // ¯\_(ツ)_/¯ I'm not driving
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | ':' | '.' => {
                        key.push(c);
                        break;
                    }
                    '=' => {
                        state =
                            State::Value(Kind::Pair(mem::take(key)), String::new(), Quote::Bare);
                        break;
                    }
                    _ => return None,
                },
                State::PostQuote => match c {
                    // Require a space after quote before anything else.
                    ' ' | '}' => {
                        state = State::Betwixt;
                        // handle in loop
                    }
                    _ => return None,
                },
            }
        }
    }
}

pub fn parse_off_attributes(content: &mut String) -> Option<Attributes> {
    let mut ci = content.char_indices().rev();
    // This is a very mid check. We can track state backwards
    // to know exactly when to attempt the parse; we care only
    // about '}', '"', '\', '}'. XXX
    let mut seen_close = false;

    loop {
        let (i, c) = ci.next()?;

        seen_close = seen_close || c == '}';

        if c == '{' && seen_close {
            if let Some((attrs, j)) = parse_attributes(&content[i..]) {
                // There should be nothing but whitespace (if anything) after.
                // Feeling generous so it can be Unicode whitespace even.
                // XXX: We can possibly fold this assertion into the above state
                // tracking? :inuthonk:
                if !content[i + j..].chars().all(char::is_whitespace) {
                    return None;
                }

                // Either there's nothing left (fine!) ORRRRR there's *at least*
                // one (but possibly multiple) whitespace, which we truncate.
                let Some((mut i, c)) = ci.next() else {
                    content.truncate(i);
                    return Some(attrs);
                };

                if !c.is_whitespace() {
                    return None;
                }

                for (i2, c) in ci {
                    if !c.is_whitespace() {
                        break;
                    }
                    i = i2;
                }

                content.truncate(i);
                return Some(attrs);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fundamentals() {
        assert_eq!(parse_attributes(""), None);

        assert_eq!(
            parse_attributes("{#henlo} there"),
            Some((
                Attributes {
                    id: Some("henlo".to_string()),
                    ..Default::default()
                },
                8
            ))
        );

        assert_eq!(
            parse_attributes("{.oken}"),
            Some((
                Attributes {
                    classes: vec!["oken".to_string()],
                    ..Default::default()
                },
                7
            ))
        );

        assert_eq!(
            parse_attributes("{data-thingy=ya}"),
            Some((
                Attributes {
                    pairs: vec![("data-thingy".to_string(), "ya".to_string())],
                    ..Default::default()
                },
                16
            ))
        );

        assert_eq!(
            parse_attributes("{.oken #yip m=26 .sure x=3 #yap} oh!"),
            Some((
                Attributes {
                    id: Some("yap".to_string()),
                    classes: vec!["oken".to_string(), "sure".to_string()],
                    pairs: vec![
                        ("m".to_string(), "26".to_string()),
                        ("x".to_string(), "3".to_string())
                    ]
                },
                32
            ))
        );

        assert_eq!(
            parse_attributes(
                "{#\"has space, will travel\" .\"ok\\\"en\" title=\"是非 \\\"not\\\"\"}"
            ),
            Some((
                Attributes {
                    id: Some("has space, will travel".to_string()),
                    classes: vec!["ok\"en".to_string(),],
                    pairs: vec![("title".to_string(), "是非 \"not\"".to_string())],
                },
                60
            ))
        );

        assert_eq!(parse_attributes("{#}"), None);
        assert_eq!(parse_attributes("{}"), Some((Attributes::default(), 2)));
        assert_eq!(parse_attributes("{.}"), None);
        assert_eq!(parse_attributes("{uh"), None);
        assert_eq!(parse_attributes("{yeah\nnah}"), None);

        assert_eq!(
            parse_attributes("{hi=}"),
            Some((
                Attributes {
                    pairs: vec![("hi".to_string(), String::new())],
                    ..Default::default()
                },
                5
            ))
        );

        // XXX: I kind of feel like this should be equivalent to above.
        // Check Pandoc.
        assert_eq!(parse_attributes("{hi}"), None);
    }

    fn assert_parse_off(input: &str, expected_str: &str, expected_attrs: Option<Attributes>) {
        let mut s = input.to_string();
        let attrs = parse_off_attributes(&mut s);
        assert_eq!(s, expected_str);
        assert_eq!(attrs, expected_attrs);
    }

    #[test]
    fn parse_off() {
        assert_parse_off("hi", "hi", None);
        assert_parse_off(
            "hi  {#yay}",
            "hi",
            Some(Attributes {
                id: Some("yay".to_string()),
                ..Default::default()
            }),
        );
    }
}
