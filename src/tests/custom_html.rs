use super::*;

#[test]
fn custom_formatter() {
    let options = Options::default();
    let arena = Arena::new();
    let doc = parse_document(&arena, "_Hello_, **world**.", &options);

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

    let mut buf: Vec<u8> = vec![];
    let mut writer = html::WriteWithLast {
        output: &mut buf,
        last_was_lf: std::cell::Cell::new(true),
    };
    let plugins = Plugins::default();
    let mut f = CustomFormatter::new(&options, &mut writer, &plugins);
    f.format(doc, false).unwrap();

    assert_eq!(
        std::str::from_utf8(&buf).unwrap(),
        "<p><i>Hello</i>, <b>world</b>.</p>\n"
    );
}
