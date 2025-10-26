use comrak::html::ChildRendering;
use comrak::{create_formatter, nodes::NodeValue};
use std::fmt::Write;

create_formatter!(CustomFormatter, {
    NodeValue::Emph => |context, entering| {
        if entering {
            context.write_str("<i>")?;
        } else {
            context.write_str("</i>")?;
        }
    },
    NodeValue::Strong => |context, entering| {
        context.write_str(if entering { "<b>" } else { "</b>" })?;
    },
    NodeValue::Image(ref nl) => |context, node, entering| {
        assert!(node.data(context.arena).sourcepos == (3, 1, 3, 18).into());
        if entering {
            context.write_str(&nl.url.to_uppercase())?;
        }
        return Ok(ChildRendering::Skip);
    },
});

fn main() {
    use comrak::{parse_document, Arena, Options};

    let options = Options::default();
    let mut arena = Arena::new();
    let doc = parse_document(
        &mut arena,
        "_Hello_, **world**.\n\n![title](/img.png)",
        &options,
    );

    let mut out = String::new();
    CustomFormatter::format_document(&arena, doc, &options, &mut out).unwrap();

    assert_eq!(out, "<p><i>Hello</i>, <b>world</b>.</p>\n<p>/IMG.PNG</p>\n");
}
