use std::mem;

use crate::nodes::Attributes;

pub fn parse_attributes(input: &str) -> Option<(Attributes, usize)> {
    let mut ci = input.char_indices();
    if ci.next()?.1 != '{' {
        return None;
    }

    enum State {
        Betwixt,
        Value(Kind, String),
    }

    enum Kind {
        Id,
        Class,
        Pair(String),
    }

    let mut state = State::Betwixt;
    let mut attrs = Attributes::default();

    loop {
        let (i, c) = ci.next()?;

        loop {
            match state {
                State::Betwixt => match c {
                    '#' => {
                        state = State::Value(Kind::Id, String::new());
                        break;
                    }
                    '.' => {
                        state = State::Value(Kind::Class, String::new());
                        break;
                    }
                    '}' => return Some((attrs, i + 1)),
                    _ => todo!(),
                },
                State::Value(ref kind, ref mut value) => match c {
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
                            Kind::Id => attrs.id = Some(mem::take(value)),
                            Kind::Class => attrs.classes.push(mem::take(value)),
                            _ => todo!(),
                        }
                        state = State::Betwixt;
                        // handle in loop
                    }
                    _ => todo!(),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial() {
        assert!(parse_attributes("").is_none());

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
                7 // XXX
            ))
        );

        assert_eq!(
            parse_attributes(
                "{#\"has space, will travel\" .\"ok\\\"en\" title=\"surely \\\"not\\\"\"}"
            ),
            Some((
                Attributes {
                    id: Some("has space, will travel".to_string()),
                    classes: vec!["ok\"en".to_string(),],
                    pairs: vec![("title".to_string(), "surely \"not\"".to_string())],
                },
                7 // XXX
            ))
        );

        assert!(parse_attributes("{#}").is_none());
        assert_eq!(parse_attributes("{}"), Some(Default::default()));
        assert!(parse_attributes("{.}").is_none());
        assert!(parse_attributes("{uh").is_none());
        assert!(parse_attributes("{yeah\nnah}").is_none());
    }
}
