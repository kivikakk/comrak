use crate::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    nodes::Sourcepos,
};

use super::*;

#[test]
fn exercise_full_api() {
    let arena = Arena::new();
    let default_options = ComrakOptions::default();
    let default_plugins = ComrakPlugins::default();
    let node = parse_document(&arena, "# My document\n", &default_options);
    let mut buffer = vec![];

    // Use every member of the exposed API without any defaults.
    // Not looking for specific outputs, just want to know if the API changes shape.

    let _: std::io::Result<()> = format_commonmark(node, &default_options, &mut buffer);

    let _: std::io::Result<()> = format_html(node, &default_options, &mut buffer);

    let _: std::io::Result<()> =
        format_html_with_plugins(node, &default_options, &mut buffer, &default_plugins);

    let _: String = Anchorizer::new().anchorize("header".to_string());

    let _: &AstNode = parse_document(&arena, "document", &default_options);

    let _: &AstNode = parse_document_with_broken_link_callback(
        &arena,
        "document",
        &default_options,
        Some(&mut |_: &str| Some(("abc".to_string(), "xyz".to_string()))),
    );

    let _ = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: false,
            tagfilter: false,
            table: false,
            autolink: false,
            tasklist: false,
            superscript: false,
            header_ids: Some("abc".to_string()),
            footnotes: false,
            description_lists: false,
            front_matter_delimiter: None,
            #[cfg(feature = "shortcodes")]
            shortcodes: true,
        },
        parse: ComrakParseOptions {
            smart: false,
            default_info_string: Some("abc".to_string()),
            relaxed_tasklist_matching: true,
        },
        render: ComrakRenderOptions {
            hardbreaks: false,
            github_pre_lang: false,
            full_info_string: false,
            width: 123456,
            unsafe_: false,
            escape: false,
            list_style: ListStyleType::Dash,
            sourcepos: false,
        },
    };

    pub struct MockAdapter {}
    impl SyntaxHighlighterAdapter for MockAdapter {
        fn write_highlighted(
            &self,
            _output: &mut dyn Write,
            _lang: Option<&str>,
            _code: &str,
        ) -> io::Result<()> {
            unreachable!()
        }

        fn write_pre_tag(
            &self,
            _output: &mut dyn Write,
            _attributes: HashMap<String, String>,
        ) -> io::Result<()> {
            unreachable!()
        }

        fn write_code_tag(
            &self,
            _output: &mut dyn Write,
            _attributes: HashMap<String, String>,
        ) -> io::Result<()> {
            unreachable!()
        }
    }

    impl HeadingAdapter for MockAdapter {
        fn enter(
            &self,
            _output: &mut dyn Write,
            _heading: &HeadingMeta,
            _sourcepos: Option<Sourcepos>,
        ) -> io::Result<()> {
            unreachable!()
        }

        fn exit(&self, _output: &mut dyn Write, _heading: &HeadingMeta) -> io::Result<()> {
            unreachable!()
        }
    }

    let mock_adapter = MockAdapter {};

    let _ = ComrakPlugins {
        render: ComrakRenderPlugins {
            codefence_syntax_highlighter: Some(&mock_adapter),
            heading_adapter: Some(&mock_adapter),
        },
    };

    let _: String = markdown_to_html("# Yes", &default_options);

    //

    let ast = node.data.borrow();
    let _: usize = ast.sourcepos.start.line;
    let _: usize = ast.sourcepos.start.column;
    let _: usize = ast.sourcepos.end.line;
    let _: usize = ast.sourcepos.end.column;
    match &ast.value {
        nodes::NodeValue::Document => {}
        nodes::NodeValue::FrontMatter(_) => {}
        nodes::NodeValue::BlockQuote => {}
        nodes::NodeValue::List(nl) | nodes::NodeValue::Item(nl) => {
            match nl.list_type {
                nodes::ListType::Bullet => {}
                nodes::ListType::Ordered => {}
            }
            let _: usize = nl.start;
            match nl.delimiter {
                nodes::ListDelimType::Period => {}
                nodes::ListDelimType::Paren => {}
            }
            let _: u8 = nl.bullet_char;
            let _: bool = nl.tight;
        }
        nodes::NodeValue::DescriptionList => {}
        nodes::NodeValue::DescriptionItem(_ndi) => {}
        nodes::NodeValue::DescriptionTerm => {}
        nodes::NodeValue::DescriptionDetails => {}
        nodes::NodeValue::CodeBlock(ncb) => {
            let _: bool = ncb.fenced;
            let _: u8 = ncb.fence_char;
            let _: usize = ncb.fence_length;
            let _: String = ncb.info;
            let _: String = ncb.literal;
        }
        nodes::NodeValue::HtmlBlock(nhb) => {
            let _: String = nhb.literal;
        }
        nodes::NodeValue::Paragraph => {}
        nodes::NodeValue::Heading(nh) => {
            let _: u8 = nh.level;
            let _: bool = nh.setext;
        }
        nodes::NodeValue::ThematicBreak => {}
        nodes::NodeValue::FootnoteDefinition(name) => {
            let _: &String = name;
        }
        nodes::NodeValue::Table(aligns) => {
            let _: &Vec<nodes::TableAlignment> = aligns;
            match aligns[0] {
                nodes::TableAlignment::None => {}
                nodes::TableAlignment::Left => {}
                nodes::TableAlignment::Center => {}
                nodes::TableAlignment::Right => {}
            }
        }
        nodes::NodeValue::TableRow(header) => {
            let _: &bool = header;
        }
        nodes::NodeValue::TableCell => {}
        nodes::NodeValue::Text(text) => {
            let _: &String = text;
        }
        nodes::NodeValue::TaskItem(symbol) => {
            let _: &Option<char> = symbol;
        }
        nodes::NodeValue::SoftBreak => {}
        nodes::NodeValue::LineBreak => {}
        nodes::NodeValue::Code(code) => {
            let _: usize = code.num_backticks;
            let _: String = code.literal;
        }
        nodes::NodeValue::HtmlInline(html) => {
            let _: &String = html;
        }
        nodes::NodeValue::Emph => {}
        nodes::NodeValue::Strong => {}
        nodes::NodeValue::Strikethrough => {}
        nodes::NodeValue::Superscript => {}
        nodes::NodeValue::Link(nl) | nodes::NodeValue::Image(nl) => {
            let _: String = nl.url;
            let _: String = nl.title;
        }
        #[cfg(feature = "shortcodes")]
        nodes::NodeValue::ShortCode(ne) => {
            let _: &str = ne.shortcode();
        }
        nodes::NodeValue::FootnoteReference(name) => {
            let _: &String = name;
        }
    }
}
