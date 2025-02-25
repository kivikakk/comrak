use comrak::html::ChildRendering;
use comrak::{create_formatter, nodes::NodeValue};
use std::io::Write;

create_formatter!(CustomFormatter, {
    NodeValue::Emph => |context, entering| {
        if entering {
            context.write_all(b"<i>")?;
        } else {
            context.write_all(b"</i>")?;
        }
    },
    NodeValue::Strong => |context, entering| {
        context.write_all(if entering { b"<b>" } else { b"</b>" })?;
    },
    NodeValue::Image(ref nl) => |context, node, entering| {
        assert!(node.data.borrow().sourcepos == (3, 1, 3, 18).into());
        if entering {
            context.write_all(nl.url.to_uppercase().as_bytes())?;
        }
        return Ok(ChildRendering::Skip);
    },
});

fn main() {
    use comrak::{parse_document, Arena, Options};

    let options = Options::default();
    let arena = Arena::new();
    let doc = parse_document(
        &arena,
        "_Hello_, **world**.\n\n![title](/img.png)",
        &options,
    );

    let mut buf: Vec<u8> = vec![];
    CustomFormatter::format_document(doc, &options, &mut buf).unwrap();

    assert_eq!(
        std::str::from_utf8(&buf).unwrap(),
        "<p><i>Hello</i>, <b>world</b>.</p>\n<p>/IMG.PNG</p>\n"
    );
}
