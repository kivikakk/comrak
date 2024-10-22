use std::sync::{Arc, Mutex};

use parser::BrokenLinkReference;

use crate::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    nodes::Sourcepos,
};

use super::*;

#[test]
fn exercise_full_api() {
    let arena = Arena::new();
    let default_options = Options::default();
    let default_plugins = Plugins::default();
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

    // Ensure the closure can modify its context.
    let mut blr_ctx_0 = 0;
    #[allow(deprecated)]
    let _: &AstNode = parse_document_with_broken_link_callback(
        &arena,
        "document",
        &Options::default(),
        Some(&mut |blr: BrokenLinkReference| {
            blr_ctx_0 += 1;
            let _: &str = blr.normalized;
            let _: &str = blr.original;
            Some(ResolvedReference {
                url: String::new(),
                title: String::new(),
            })
        }),
    );

    let extension = ExtensionOptions::builder()
        .strikethrough(false)
        .tagfilter(false)
        .table(false)
        .autolink(false)
        .tasklist(false)
        .superscript(false)
        .header_ids("abc".to_string())
        .footnotes(false)
        .description_lists(false)
        .math_dollars(false)
        .math_code(false)
        .maybe_front_matter_delimiter(None)
        .multiline_block_quotes(false);

    #[cfg(feature = "shortcodes")]
    let extension = extension.shortcodes(true);

    let _extension = extension
        .wikilinks_title_after_pipe(true)
        .wikilinks_title_before_pipe(true)
        .underline(true)
        .spoiler(true)
        .greentext(true);

    let parse = ParseOptions::builder()
        .smart(false)
        .default_info_string("abc".to_string())
        .relaxed_tasklist_matching(false)
        .relaxed_autolinks(false);

    let mut blr_ctx_1 = 0;
    let _parse =
        parse.broken_link_callback(Arc::new(Mutex::new(&mut |blr: BrokenLinkReference| {
            blr_ctx_1 += 1;
            let _: &str = blr.normalized;
            let _: &str = blr.original;
            Some(ResolvedReference {
                url: String::new(),
                title: String::new(),
            })
        })));

    let _render = RenderOptions::builder()
        .hardbreaks(false)
        .github_pre_lang(false)
        .full_info_string(false)
        .width(123456)
        .unsafe_(false)
        .escape(false)
        .list_style(ListStyleType::Dash)
        .sourcepos(false)
        .experimental_inline_sourcepos(false)
        .escaped_char_spans(false)
        .ignore_setext(true)
        .ignore_empty_links(true)
        .gfm_quirks(true)
        .prefer_fenced(true)
        .figure_with_caption(true);

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

    let render_plugins = RenderPlugins::builder()
        .codefence_syntax_highlighter(&mock_adapter)
        .heading_adapter(&mock_adapter);

    let _plugins = Plugins::builder().render(render_plugins.build());

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
        nodes::NodeValue::FootnoteDefinition(nfd) => {
            let _: &String = &nfd.name;
            let _: u32 = nfd.total_references;
        }
        nodes::NodeValue::Table(nt) => {
            let _: &Vec<nodes::TableAlignment> = &nt.alignments;
            let _: usize = nt.num_nonempty_cells;
            let _: usize = nt.num_rows;
            match nt.alignments[0] {
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
        nodes::NodeValue::ShortCode(nsc) => {
            let _: String = nsc.code;
            let _: String = nsc.emoji;
        }
        nodes::NodeValue::FootnoteReference(nfr) => {
            let _: String = nfr.name;
            let _: u32 = nfr.ix;
        }
        nodes::NodeValue::MultilineBlockQuote(mbc) => {
            let _: usize = mbc.fence_length;
            let _: usize = mbc.fence_offset;
        }
        nodes::NodeValue::Escaped => {}
        nodes::NodeValue::Math(math) => {
            let _: bool = math.display_math;
            let _: bool = math.dollar_math;
            let _: String = math.literal;
        }
        nodes::NodeValue::WikiLink(nl) => {
            let _: String = nl.url;
        }
        nodes::NodeValue::Underline => {}
        nodes::NodeValue::SpoileredText => {}
        nodes::NodeValue::EscapedTag(data) => {
            let _: &String = data;
        }
    }
}
