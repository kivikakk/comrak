use comrak::{create_formatter, nodes::NodeValue, parse_document, Arena, Options};
use std::io::Write;

create_formatter!(CustomFormatter, |output, entering| {
    NodeValue::Emph => {
        if entering {
            output.write_all(b"<i>")?;
        } else {
            output.write_all(b"</i>")?;
        }
    },
    NodeValue::Strong => {
        if entering {
            output.write_all(b"<b>")?;
        } else {
            output.write_all(b"</b>")?;
        }
    },
});

fn main() {
    let options = Options::default();
    let arena = Arena::new();
    let doc = parse_document(&arena, "_Hello_, **world**.", &options);

    let mut buf: Vec<u8> = vec![];
    CustomFormatter::format_document(doc, &options, &mut buf).unwrap();

    assert_eq!(
        std::str::from_utf8(&buf).unwrap(),
        "<p><i>Hello</i>, <b>world</b>.</p>\n"
    );
}
