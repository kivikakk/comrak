use crate::nodes::{AstNode, NodeCode, NodeValue};
use adapters::SyntaxHighlighterAdapter;
use cm;
use html;
#[cfg(feature = "syntect")]
use plugins::syntect::SyntectAdapter;
use propfuzz::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use strings::build_opening_tag;
use timebomb::timeout_ms;
use {
    parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakPlugins, ComrakRenderOptions,
};

#[propfuzz]
fn fuzz_doesnt_crash(md: String) {
    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: Some("user-content-".to_string()),
            footnotes: true,
            description_lists: true,
            front_matter_delimiter: None,
        },
        parse: ComrakParseOptions {
            smart: true,
            default_info_string: Some("Rust".to_string()),
        },
        render: ComrakRenderOptions {
            hardbreaks: true,
            github_pre_lang: true,
            width: 80,
            unsafe_: true,
            escape: false,
        },
    };

    parse_document(&Arena::new(), &md, &options);
}

#[track_caller]
fn compare_strs(output: &str, expected: &str, kind: &str) {
    if output != expected {
        println!("Running {} test", kind);
        println!("Got:");
        println!("==============================");
        println!("{}", output);
        println!("==============================");
        println!();
        println!("Expected:");
        println!("==============================");
        println!("{}", expected);
        println!("==============================");
        println!();
    }
    assert_eq!(output, expected);
}

#[track_caller]
fn html(input: &str, expected: &str) {
    html_opts(input, expected, |_| ());
}

#[track_caller]
fn html_opts<F>(input: &str, expected: &str, opts: F)
where
    F: Fn(&mut ComrakOptions),
{
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    opts(&mut options);

    let root = parse_document(&arena, input, &options);
    let mut output = vec![];
    html::format_document(root, &options, &mut output).unwrap();
    compare_strs(&String::from_utf8(output).unwrap(), expected, "regular");

    let mut md = vec![];
    cm::format_document(root, &options, &mut md).unwrap();
    let root = parse_document(&arena, &String::from_utf8(md).unwrap(), &options);
    let mut output_from_rt = vec![];
    html::format_document(root, &options, &mut output_from_rt).unwrap();
    compare_strs(
        &String::from_utf8(output_from_rt).unwrap(),
        expected,
        "roundtrip",
    );
}

macro_rules! html_opts {
    ([$($optclass:ident.$optname:ident),*], $lhs:expr, $rhs:expr,) => {
        html_opts!([$($optclass.$optname),*], $lhs, $rhs)
    };
    ([$($optclass:ident.$optname:ident),*], $lhs:expr, $rhs:expr) => {
        html_opts($lhs, $rhs, |opts| {
            $(opts.$optclass.$optname = true;)*
        });
    };
}

fn html_plugins(input: &str, expected: &str, plugins: &ComrakPlugins) {
    let arena = Arena::new();
    let options = ComrakOptions::default();

    let root = parse_document(&arena, input, &options);
    let mut output = vec![];
    html::format_document_with_plugins(root, &options, &mut output, &plugins).unwrap();
    compare_strs(&String::from_utf8(output).unwrap(), expected, "regular");

    let mut md = vec![];
    cm::format_document(root, &options, &mut md).unwrap();
    let root = parse_document(&arena, &String::from_utf8(md).unwrap(), &options);
    let mut output_from_rt = vec![];
    html::format_document_with_plugins(root, &options, &mut output_from_rt, &plugins).unwrap();
    compare_strs(
        &String::from_utf8(output_from_rt).unwrap(),
        expected,
        "roundtrip",
    );
}

fn asssert_node_eq<'a>(node: &'a AstNode<'a>, location: &[usize], expected: &NodeValue) {
    let node = location
        .iter()
        .fold(node, |node, &n| node.children().nth(n).unwrap());

    let data = node.data.borrow();
    let actual = format!("{:?}", data.value);
    let expected = format!("{:?}", expected);

    compare_strs(&actual, &expected, "ast comparison");
}

#[test]
fn basic() {
    html(
        concat!(
            "My **document**.\n",
            "\n",
            "It's mine.\n",
            "\n",
            "> Yes.\n",
            "\n",
            "## Hi!\n",
            "\n",
            "Okay.\n"
        ),
        concat!(
            "<p>My <strong>document</strong>.</p>\n",
            "<p>It's mine.</p>\n",
            "<blockquote>\n",
            "<p>Yes.</p>\n",
            "</blockquote>\n",
            "<h2>Hi!</h2>\n",
            "<p>Okay.</p>\n"
        ),
    );
}

#[test]
fn codefence() {
    html(
        concat!("``` rust yum\n", "fn main<'a>();\n", "```\n"),
        concat!(
            "<pre><code class=\"language-rust\">fn main&lt;'a&gt;();\n",
            "</code></pre>\n"
        ),
    );
}

#[test]
fn syntax_highlighter_plugin() {
    pub struct MockAdapter {}

    impl SyntaxHighlighterAdapter for MockAdapter {
        fn highlight(&self, lang: Option<&str>, code: &str) -> String {
            format!("<!--{}--><span>{}</span>", lang.unwrap(), code)
        }

        fn build_pre_tag(&self, attributes: &HashMap<String, String>) -> String {
            build_opening_tag("pre", attributes)
        }

        fn build_code_tag(&self, attributes: &HashMap<String, String>) -> String {
            build_opening_tag("code", attributes)
        }
    }

    let input = concat!("``` rust yum\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre><code class=\"language-rust\"><!--rust--><span>fn main<'a>();\n</span>",
        "</code></pre>\n"
    );

    let mut plugins = ComrakPlugins::default();
    let adapter = MockAdapter {};
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
#[cfg(feature = "syntect")]
fn syntect_plugin() {
    let adapter = SyntectAdapter::new("base16-ocean.dark");

    let input = concat!("```rust\n", "fn main<'a>();\n", "```\n");
    let expected = concat!(
        "<pre style=\"background-color:#2b303b;\"><code class=\"language-rust\">\n",
        "<span style=\"color:#b48ead;\">fn </span><span style=\"color:#8fa1b3;\">main</span><span style=\"color:#c0c5ce;\">",
        "&lt;</span><span style=\"color:#b48ead;\">&#39;a</span><span style=\"color:#c0c5ce;\">&gt;();\n</span>\n",
        "</code></pre>\n"
    );

    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    html_plugins(input, expected, &plugins);
}

#[test]
fn lists() {
    html(
        concat!("2. Hello.\n", "3. Hi.\n"),
        concat!(
            "<ol start=\"2\">\n",
            "<li>Hello.</li>\n",
            "<li>Hi.</li>\n",
            "</ol>\n"
        ),
    );

    html(
        concat!("- Hello.\n", "- Hi.\n"),
        concat!("<ul>\n", "<li>Hello.</li>\n", "<li>Hi.</li>\n", "</ul>\n"),
    );
}

#[test]
fn thematic_breaks() {
    html(
        concat!("---\n", "\n", "- - -\n", "\n", "\n", "_        _   _\n"),
        concat!("<hr />\n", "<hr />\n", "<hr />\n"),
    );
}

#[test]
fn setext_heading() {
    html(
        concat!("Hi\n", "==\n", "\n", "Ok\n", "-----\n"),
        concat!("<h1>Hi</h1>\n", "<h2>Ok</h2>\n"),
    );
}

#[test]
fn html_block_1() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "<script>\n",
            "*ok* </script> *ok*\n",
            "\n",
            "*ok*\n",
            "\n",
            "*ok*\n",
            "\n",
            "<pre x>\n",
            "*ok*\n",
            "</style>\n",
            "*ok*\n",
            "<style>\n",
            "*ok*\n",
            "</style>\n",
            "\n",
            "*ok*\n"
        ),
        concat!(
            "<script>\n",
            "*ok* </script> *ok*\n",
            "<p><em>ok</em></p>\n",
            "<p><em>ok</em></p>\n",
            "<pre x>\n",
            "*ok*\n",
            "</style>\n",
            "<p><em>ok</em></p>\n",
            "<style>\n",
            "*ok*\n",
            "</style>\n",
            "<p><em>ok</em></p>\n"
        ),
    );
}

#[test]
fn html_block_2() {
    html_opts!(
        [render.unsafe_],
        concat!("   <!-- abc\n", "\n", "ok --> *hi*\n", "*hi*\n"),
        concat!(
            "   <!-- abc\n",
            "\n",
            "ok --> *hi*\n",
            "<p><em>hi</em></p>\n"
        ),
    );
}

#[test]
fn html_block_3() {
    html_opts!(
        [render.unsafe_],
        concat!(" <? o\n", "k ?> *a*\n", "*a*\n"),
        concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"),
    );
}

#[test]
fn html_block_4() {
    html_opts!(
        [render.unsafe_],
        concat!("<!X >\n", "ok\n", "<!X\n", "um > h\n", "ok\n"),
        concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"),
    );
}

#[test]
fn html_block_5() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "<![CDATA[\n",
            "\n",
            "hm >\n",
            "*ok*\n",
            "]]> *ok*\n",
            "*ok*\n"
        ),
        concat!(
            "<![CDATA[\n",
            "\n",
            "hm >\n",
            "*ok*\n",
            "]]> *ok*\n",
            "<p><em>ok</em></p>\n"
        ),
    );
}

#[test]
fn html_block_6() {
    html_opts!(
        [render.unsafe_],
        concat!(" </table>\n", "*x*\n", "\n", "ok\n", "\n", "<li\n", "*x*\n"),
        concat!(" </table>\n", "*x*\n", "<p>ok</p>\n", "<li\n", "*x*\n"),
    );
}

#[test]
fn html_block_7() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "<a b >\n",
            "ok\n",
            "\n",
            "<a b=>\n",
            "ok\n",
            "\n",
            "<a b \n",
            "<a b> c\n",
            "ok\n"
        ),
        concat!(
            "<a b >\n",
            "ok\n",
            "<p>&lt;a b=&gt;\n",
            "ok</p>\n",
            "<p>&lt;a b\n",
            "<a b> c\n",
            "ok</p>\n"
        ),
    );

    html_opts!(
        [render.unsafe_],
        concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "\n", "ok\n"),
        concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "<p>ok</p>\n"),
    );
}

#[test]
fn backticks() {
    html(
        "Some `code\\` yep.\n",
        "<p>Some <code>code\\</code> yep.</p>\n",
    );
}

#[test]
fn backticks_empty_with_newline_should_be_space() {
    html("`\n`", "<p><code> </code></p>\n");
}

#[test]
fn backticks_num() {
    let input = "Some `code1`. More ``` code2 ```.\n";

    let arena = Arena::new();
    let options = ComrakOptions::default();
    let root = parse_document(&arena, input, &options);

    let code1 = NodeValue::Code(NodeCode {
        num_backticks: 1,
        literal: b"code1".to_vec(),
    });
    asssert_node_eq(root, &[0, 1], &code1);

    let code2 = NodeValue::Code(NodeCode {
        num_backticks: 3,
        literal: b"code2".to_vec(),
    });
    asssert_node_eq(root, &[0, 3], &code2);
}

#[test]
fn backslashes() {
    html(
        concat!(
            "Some \\`fake code\\`.\n",
            "\n",
            "Some fake linebreaks:\\\n",
            "Yes.\\\n",
            "See?\n",
            "\n",
            "Ga\\rbage.\n"
        ),
        concat!(
            "<p>Some `fake code`.</p>\n",
            "<p>Some fake linebreaks:<br />\n",
            "Yes.<br />\n",
            "See?</p>\n",
            "<p>Ga\\rbage.</p>\n"
        ),
    );
}

#[test]
fn entities() {
    html(
        concat!(
            "This is &amp;, &copy;, &trade;, \\&trade;, &xyz;, &NotEqualTilde;.\n",
            "\n",
            "&#8734; &#x221e;\n"
        ),
        concat!(
            "<p>This is &amp;, ©, ™, &amp;trade;, &amp;xyz;, \u{2242}\u{338}.</p>\n",
            "<p>∞ ∞</p>\n"
        ),
    );
}

#[test]
fn pointy_brace() {
    html_opts!(
        [render.unsafe_],
        concat!(
            "URI autolink: <https://www.pixiv.net>\n",
            "\n",
            "Email autolink: <bill@microsoft.com>\n",
            "\n",
            "* Inline <em>tag</em> **ha**.\n",
            "* Inline <!-- comment --> **ha**.\n",
            "* Inline <? processing instruction ?> **ha**.\n",
            "* Inline <!DECLARATION OKAY> **ha**.\n",
            "* Inline <![CDATA[ok]ha **ha** ]]> **ha**.\n"
        ),
        concat!(
            "<p>URI autolink: <a \
             href=\"https://www.pixiv.net\">https://www.pixiv.net</a></p>\n",
            "<p>Email autolink: <a \
             href=\"mailto:bill@microsoft.com\">bill@microsoft.com</a></p>\n",
            "<ul>\n",
            "<li>Inline <em>tag</em> <strong>ha</strong>.</li>\n",
            "<li>Inline <!-- comment --> <strong>ha</strong>.</li>\n",
            "<li>Inline <? processing instruction ?> <strong>ha</strong>.</li>\n",
            "<li>Inline <!DECLARATION OKAY> <strong>ha</strong>.</li>\n",
            "<li>Inline <![CDATA[ok]ha **ha** ]]> <strong>ha</strong>.</li>\n",
            "</ul>\n"
        ),
    );
}

#[test]
fn links() {
    html(
        concat!(
            "Where are you [going](https://microsoft.com (today))?\n",
            "\n",
            "[Where am I?](/here)\n"
        ),
        concat!(
            "<p>Where are you <a href=\"https://microsoft.com\" \
             title=\"today\">going</a>?</p>\n",
            "<p><a href=\"/here\">Where am I?</a></p>\n"
        ),
    );
}

#[test]
fn images() {
    html(
        concat!("I am ![eating [things](/url)](http://i.imgur.com/QqK1vq7.png).\n"),
        concat!(
            "<p>I am <img src=\"http://i.imgur.com/QqK1vq7.png\" alt=\"eating things\" \
             />.</p>\n"
        ),
    );
}

#[test]
fn reference_links() {
    html(
        concat!(
            "This [is] [legit], [very][honestly] legit.\n",
            "\n",
            "[legit]: ok\n",
            "[honestly]: sure \"hm\"\n"
        ),
        concat!(
            "<p>This [is] <a href=\"ok\">legit</a>, <a href=\"sure\" title=\"hm\">very</a> \
             legit.</p>\n"
        ),
    );
}

#[test]
fn link_entity_regression() {
    html(
        "[link](&#x6A&#x61&#x76&#x61&#x73&#x63&#x72&#x69&#x70&#x74&#x3A&#x61&#x6C&#x65&#x72&#x74&#x28&#x27&#x58&#x53&#x53&#x27&#x29)",
        "<p><a href=\"&amp;#x6A&amp;#x61&amp;#x76&amp;#x61&amp;#x73&amp;#x63&amp;#x72&amp;#x69&amp;#x70&amp;#x74&amp;#x3A&amp;#x61&amp;#x6C&amp;#x65&amp;#x72&amp;#x74&amp;#x28&amp;#x27&amp;#x58&amp;#x53&amp;#x53&amp;#x27&amp;#x29\">link</a></p>\n",
    );
}

#[test]
fn strikethrough() {
    html_opts!(
        [extension.strikethrough],
        concat!(
            "This is ~strikethrough~.\n",
            "\n",
            "As is ~~this, okay~~?\n"
        ),
        concat!(
            "<p>This is <del>strikethrough</del>.</p>\n",
            "<p>As is <del>this, okay</del>?</p>\n"
        ),
    );
}

#[test]
fn table() {
    html_opts!(
        [extension.table],
        concat!("| a | b |\n", "|---|:-:|\n", "| c | d |\n"),
        concat!(
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th align=\"center\">b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td align=\"center\">d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n"
        ),
    );
}

#[test]
fn table_regression() {
    html_opts!(
        [extension.table],
        concat!("123\n", "456\n", "| a | b |\n", "| ---| --- |\n", "d | e\n"),
        concat!(
            "<p>123\n",
            "456</p>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>d</td>\n",
            "<td>e</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n"
        ),
    );
}

#[test]
fn autolink_www() {
    html_opts!(
        [extension.autolink],
        concat!("www.autolink.com\n"),
        concat!("<p><a href=\"http://www.autolink.com\">www.autolink.com</a></p>\n"),
    );
}

#[test]
fn autolink_email() {
    html_opts!(
        [extension.autolink],
        concat!("john@smith.com\n"),
        concat!("<p><a href=\"mailto:john@smith.com\">john@smith.com</a></p>\n"),
    );
}

#[test]
fn autolink_scheme() {
    html_opts!(
        [extension.autolink],
        concat!("https://google.com/search\n"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a></p>\n"
        ),
    );
}

#[test]
fn autolink_scheme_multiline() {
    html_opts!(
        [extension.autolink],
        concat!("https://google.com/search\nhttps://www.google.com/maps"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a>\n<a href=\"https://www.google.com/maps\">\
             https://www.google.com/maps</a></p>\n"
        ),
    );
}

#[test]
fn autolink_no_link_bad() {
    html_opts!(
        [extension.autolink],
        concat!("@a.b.c@. x\n", "\n", "n@. x\n"),
        concat!("<p>@a.b.c@. x</p>\n", "<p>n@. x</p>\n"),
    );
}

#[test]
fn tagfilter() {
    html_opts!(
        [render.unsafe_, extension.tagfilter],
        concat!("hi <xmp> ok\n", "\n", "<xmp>\n"),
        concat!("<p>hi &lt;xmp> ok</p>\n", "&lt;xmp>\n"),
    );
}

#[test]
fn tasklist() {
    html_opts!(
        [render.unsafe_, extension.tasklist],
        concat!(
            "* [ ] Red\n",
            "* [x] Green\n",
            "* [ ] Blue\n",
            "<!-- end list -->\n",
            "1. [ ] Bird\n",
            "2. [ ] McHale\n",
            "3. [x] Parish\n",
            "<!-- end list -->\n",
            "* [ ] Red\n",
            "  * [x] Green\n",
            "    * [ ] Blue\n"
        ),
        concat!(
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Red</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Green</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Blue</li>\n",
            "</ul>\n",
            "<!-- end list -->\n",
            "<ol>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Bird</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> McHale</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Parish</li>\n",
            "</ol>\n",
            "<!-- end list -->\n",
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Red\n",
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> Green\n",
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> Blue</li>\n",
            "</ul>\n",
            "</li>\n",
            "</ul>\n",
            "</li>\n",
            "</ul>\n"
        ),
    );
}

#[test]
fn tasklist_32() {
    html_opts!(
        [render.unsafe_, extension.tasklist],
        concat!(
            "- [ ] List item 1\n",
            "- [ ] This list item is **bold**\n",
            "- [x] There is some `code` here\n"
        ),
        concat!(
            "<ul>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> List item 1</li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" /> This list item is <strong>bold</strong></li>\n",
            "<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> There is some <code>code</code> here</li>\n",
            "</ul>\n"
        ),
    );
}

#[test]
fn superscript() {
    html_opts!(
        [extension.superscript],
        concat!("e = mc^2^.\n"),
        concat!("<p>e = mc<sup>2</sup>.</p>\n"),
    );
}

#[test]
fn header_ids() {
    html_opts(
        concat!(
            "# Hi.\n",
            "## Hi 1.\n",
            "### Hi.\n",
            "#### Hello.\n",
            "##### Hi.\n",
            "###### Hello.\n",
            "# Isn't it grand?"
        ),
        concat!(
            "<h1><a href=\"#hi\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi\"></a>Hi.</h1>\n",
            "<h2><a href=\"#hi-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-1\"></a>Hi 1.</h2>\n",
            "<h3><a href=\"#hi-2\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-2\"></a>Hi.</h3>\n",
            "<h4><a href=\"#hello\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello\"></a>Hello.</h4>\n",
            "<h5><a href=\"#hi-3\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-3\"></a>Hi.</h5>\n",
            "<h6><a href=\"#hello-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello-1\"></a>Hello.</h6>\n",
            "<h1><a href=\"#isnt-it-grand\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-isnt-it-grand\"></a>Isn't it grand?</h1>\n"
        ),
        |opts| opts.extension.header_ids = Some("user-content-".to_owned()),
    );
}

#[test]
fn footnotes() {
    html_opts!(
        [extension.footnotes],
        concat!(
            "Here is a[^nowhere] footnote reference,[^1] and another.[^longnote]\n",
            "\n",
            "This is another note.[^note]\n",
            "\n",
            "[^note]: Hi.\n",
            "\n",
            "[^1]: Here is the footnote.\n",
            "\n",
            "[^longnote]: Here's one with multiple blocks.\n",
            "\n",
            "    Subsequent paragraphs are indented.\n",
            "\n",
            "        code\n",
            "\n",
            "This is regular content.\n",
            "\n",
            "[^unused]: This is not used.\n"
        ),
        concat!(
            "<p>Here is a[^nowhere] footnote reference,<sup class=\"footnote-ref\"><a href=\"#fn1\" \
             id=\"fnref1\">1</a></sup> and another.<sup class=\"footnote-ref\"><a \
             href=\"#fn2\" id=\"fnref2\">2</a></sup></p>\n",
            "<p>This is another note.<sup class=\"footnote-ref\"><a href=\"#fn3\" \
             id=\"fnref3\">3</a></sup></p>\n",
            "<p>This is regular content.</p>\n",
            "<section class=\"footnotes\">\n",
            "<ol>\n",
            "<li id=\"fn1\">\n",
            "<p>Here is the footnote. <a href=\"#fnref1\" \
             class=\"footnote-backref\">↩</a></p>\n",
            "</li>\n",
            "<li id=\"fn2\">\n",
            "<p>Here's one with multiple blocks.</p>\n",
            "<p>Subsequent paragraphs are indented.</p>\n",
            "<pre><code>code\n",
            "</code></pre>\n",
            "<a href=\"#fnref2\" class=\"footnote-backref\">↩</a>\n",
            "</li>\n",
            "<li id=\"fn3\">\n",
            "<p>Hi. <a href=\"#fnref3\" \
             class=\"footnote-backref\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn footnote_does_not_eat_exclamation() {
    html_opts!(
        [extension.footnotes],
        concat!("Here's my footnote![^a]\n", "\n", "[^a]: Yep.\n"),
        concat!(
            "<p>Here's my footnote!<sup class=\"footnote-ref\"><a href=\"#fn1\" \
             id=\"fnref1\">1</a></sup></p>\n",
            "<section class=\"footnotes\">\n",
            "<ol>\n",
            "<li id=\"fn1\">\n",
            "<p>Yep. <a href=\"#fnref1\" class=\"footnote-backref\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
    );
}

#[test]
fn footnote_in_table() {
    html_opts!(
        [extension.table, extension.footnotes],
        concat!(
            "A footnote in a paragraph[^1]\n",
            "\n",
            "| Column1   | Column2 |\n",
            "| --------- | ------- |\n",
            "| foot [^1] | note    |\n",
            "\n",
            "[^1]: a footnote\n",
        ), concat!(
            "<p>A footnote in a paragraph<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup></p>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>Column1</th>\n",
            "<th>Column2</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>foot <sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup></td>\n",
            "<td>note</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "<section class=\"footnotes\">\n",
            "<ol>\n",
            "<li id=\"fn1\">\n",
            "<p>a footnote <a href=\"#fnref1\" class=\"footnote-backref\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n",
        ));
}

#[test]
fn regression_back_to_back_ranges() {
    html(
        "**bold*****bold+italic***",
        "<p><strong>bold</strong><em><strong>bold+italic</strong></em></p>\n",
    );
}

#[test]
fn pathological_emphases() {
    let mut s = String::with_capacity(50000 * 4);
    for _ in 0..50000 {
        s.push_str("*a_ ");
    }

    let mut exp = format!("<p>{}", s);
    // Right-most space is trimmed in output.
    exp.pop();
    exp += "</p>\n";

    timeout_ms(move || html(&s, &exp), 4000);
}

#[test]
fn no_panic_on_empty_bookended_atx_headers() {
    html("#  #", "<h1></h1>\n");
}

#[test]
fn table_misparse_1() {
    html_opts!([extension.table], "a\n-b", "<p>a\n-b</p>\n");
}

#[test]
fn table_misparse_2() {
    html_opts!([extension.table], "a\n-b\n-c", "<p>a\n-b\n-c</p>\n");
}

#[test]
fn smart_chars() {
    html_opts!(
        [parse.smart],
        "Why 'hello' \"there\". It's good.",
        "<p>Why ‘hello’ “there”. It’s good.</p>\n",
    );

    html_opts!(
        [parse.smart],
        "Hm. Hm.. hm... yes- indeed-- quite---!",
        "<p>Hm. Hm.. hm… yes- indeed– quite—!</p>\n",
    );
}

#[test]
fn nested_tables_1() {
    html_opts!(
        [extension.table],
        concat!("- p\n", "\n", "    |a|b|\n", "    |-|-|\n", "    |c|d|\n",),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>p</p>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td>d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn nested_tables_2() {
    html_opts!(
        [extension.table],
        concat!("- |a|b|\n", "  |-|-|\n", "  |c|d|\n",),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td>d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn nested_tables_3() {
    html_opts!(
        [extension.table],
        concat!("> |a|b|\n", "> |-|-|\n", "> |c|d|\n",),
        concat!(
            "<blockquote>\n",
            "<table>\n",
            "<thead>\n",
            "<tr>\n",
            "<th>a</th>\n",
            "<th>b</th>\n",
            "</tr>\n",
            "</thead>\n",
            "<tbody>\n",
            "<tr>\n",
            "<td>c</td>\n",
            "<td>d</td>\n",
            "</tr>\n",
            "</tbody>\n",
            "</table>\n",
            "</blockquote>\n",
        ),
    );
}

#[test]
fn no_stack_smash_html() {
    let s: String = ::std::iter::repeat('>').take(150_000).collect();
    let arena = Arena::new();
    let root = parse_document(&arena, &s, &ComrakOptions::default());
    let mut output = vec![];
    html::format_document(root, &ComrakOptions::default(), &mut output).unwrap()
}

#[test]
fn no_stack_smash_cm() {
    let s: String = ::std::iter::repeat('>').take(150_000).collect();
    let arena = Arena::new();
    let root = parse_document(&arena, &s, &ComrakOptions::default());
    let mut output = vec![];
    cm::format_document(root, &ComrakOptions::default(), &mut output).unwrap()
}

#[test]
fn cm_autolink_regression() {
    // Testing that the cm renderer handles this case without crashing
    html("<a+c:dd>", "<p><a href=\"a+c:dd\">a+c:dd</a></p>\n");
}

#[test]
fn safety() {
    html(
        concat!(
            "[data:image/png](data:image/png/x)\n\n",
            "[data:image/gif](data:image/gif/x)\n\n",
            "[data:image/jpeg](data:image/jpeg/x)\n\n",
            "[data:image/webp](data:image/webp/x)\n\n",
            "[data:malicious](data:malicious/x)\n\n",
            "[javascript:malicious](javascript:malicious)\n\n",
            "[vbscript:malicious](vbscript:malicious)\n\n",
            "[file:malicious](file:malicious)\n\n",
        ),
        concat!(
            "<p><a href=\"data:image/png/x\">data:image/png</a></p>\n",
            "<p><a href=\"data:image/gif/x\">data:image/gif</a></p>\n",
            "<p><a href=\"data:image/jpeg/x\">data:image/jpeg</a></p>\n",
            "<p><a href=\"data:image/webp/x\">data:image/webp</a></p>\n",
            "<p><a href=\"\">data:malicious</a></p>\n",
            "<p><a href=\"\">javascript:malicious</a></p>\n",
            "<p><a href=\"\">vbscript:malicious</a></p>\n",
            "<p><a href=\"\">file:malicious</a></p>\n",
        ),
    )
}

#[test]
fn link_backslash_requires_punct() {
    // Test should probably be in the spec.
    html("[a](\\ b)", "<p>[a](\\ b)</p>\n");
}

// Again, at least some of these cases are not covered by the reference
// implementation's test suite - 3 and 4 were broken in comrak.

#[test]
fn nul_replacement_1() {
    html("a\0b", "<p>a\u{fffd}b</p>\n");
}

#[test]
fn nul_replacement_2() {
    html("a\0b\0c", "<p>a\u{fffd}b\u{fffd}c</p>\n");
}

#[test]
fn nul_replacement_3() {
    html("a\0\nb", "<p>a\u{fffd}\nb</p>\n");
}

#[test]
fn nul_replacement_4() {
    html("a\0\r\nb", "<p>a\u{fffd}\nb</p>\n");
}

#[test]
fn nul_replacement_5() {
    html("a\r\n\0b", "<p>a\n\u{fffd}b</p>\n");
}

#[test]
fn description_lists() {
    html_opts!(
        [extension.description_lists],
        concat!(
            "Term 1\n",
            "\n",
            ": Definition 1\n",
            "\n",
            "Term 2 with *inline markup*\n",
            "\n",
            ": Definition 2\n"
        ),
        concat!(
            "<dl>",
            "<dt>Term 1</dt>\n",
            "<dd>\n",
            "<p>Definition 1</p>\n",
            "</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>\n",
            "<p>Definition 2</p>\n",
            "</dd>\n",
            "</dl>\n",
        ),
    );

    html_opts!(
        [extension.description_lists],
        concat!(
            "* Nested\n",
            "\n",
            "    Term 1\n\n",
            "    :   Definition 1\n\n",
            "    Term 2 with *inline markup*\n\n",
            "    :   Definition 2\n\n"
        ),
        concat!(
            "<ul>\n",
            "<li>\n",
            "<p>Nested</p>\n",
            "<dl>",
            "<dt>Term 1</dt>\n",
            "<dd>\n",
            "<p>Definition 1</p>\n",
            "</dd>\n",
            "<dt>Term 2 with <em>inline markup</em></dt>\n",
            "<dd>\n",
            "<p>Definition 2</p>\n",
            "</dd>\n",
            "</dl>\n",
            "</li>\n",
            "</ul>\n",
        ),
    );
}

#[test]
fn case_insensitive_safety() {
    html(
        "[a](javascript:a) [b](Javascript:b) [c](jaVascript:c) [d](data:xyz) [e](Data:xyz) [f](vbscripT:f) [g](FILE:g)\n",
        "<p><a href=\"\">a</a> <a href=\"\">b</a> <a href=\"\">c</a> <a href=\"\">d</a> <a href=\"\">e</a> <a href=\"\">f</a> <a href=\"\">g</a></p>\n",
    );
}

#[test]
fn exercise_full_api<'a>() {
    let arena = ::Arena::new();
    let default_options = ::ComrakOptions::default();
    let default_plugins = ::ComrakPlugins::default();
    let node = ::parse_document(&arena, "# My document\n", &default_options);
    let mut buffer = vec![];

    // Use every member of the exposed API without any defaults.
    // Not looking for specific outputs, just want to know if the API changes shape.

    let _: std::io::Result<()> = ::format_commonmark(node, &default_options, &mut buffer);

    let _: std::io::Result<()> = ::format_html(node, &default_options, &mut buffer);

    let _: std::io::Result<()> =
        ::format_html_with_plugins(node, &default_options, &mut buffer, &default_plugins);

    let _: String = ::Anchorizer::new().anchorize("header".to_string());

    let _: &AstNode = ::parse_document(&arena, "document", &default_options);

    let _: &AstNode = ::parse_document_with_broken_link_callback(
        &arena,
        "document",
        &default_options,
        Some(&mut |_: &[u8]| Some((b"abc".to_vec(), b"xyz".to_vec()))),
    );

    let _ = ::ComrakOptions {
        extension: ::ComrakExtensionOptions {
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
        },
        parse: ::ComrakParseOptions {
            smart: false,
            default_info_string: Some("abc".to_string()),
        },
        render: ::ComrakRenderOptions {
            hardbreaks: false,
            github_pre_lang: false,
            width: 123456,
            unsafe_: false,
            escape: false,
        },
    };

    pub struct MockAdapter {}
    impl SyntaxHighlighterAdapter for MockAdapter {
        fn highlight(&self, lang: Option<&str>, code: &str) -> String {
            String::from(format!("{}{}", lang.unwrap(), code))
        }

        fn build_pre_tag(&self, attributes: &HashMap<String, String>) -> String {
            build_opening_tag("pre", &attributes)
        }

        fn build_code_tag(&self, attributes: &HashMap<String, String>) -> String {
            build_opening_tag("code", &attributes)
        }
    }

    let syntax_highlighter_adapter = MockAdapter {};

    let _ = ::ComrakPlugins {
        render: ::ComrakRenderPlugins {
            codefence_syntax_highlighter: Some(&syntax_highlighter_adapter),
        },
    };

    let _: String = ::markdown_to_html("# Yes", &default_options);

    //

    let ast = node.data.borrow();
    let _ = ast.start_line;
    match &ast.value {
        ::nodes::NodeValue::Document => {}
        ::nodes::NodeValue::FrontMatter(_) => {}
        ::nodes::NodeValue::BlockQuote => {}
        ::nodes::NodeValue::List(nl) | ::nodes::NodeValue::Item(nl) => {
            match nl.list_type {
                ::nodes::ListType::Bullet => {}
                ::nodes::ListType::Ordered => {}
            }
            let _: usize = nl.start;
            match nl.delimiter {
                ::nodes::ListDelimType::Period => {}
                ::nodes::ListDelimType::Paren => {}
            }
            let _: u8 = nl.bullet_char;
            let _: bool = nl.tight;
        }
        ::nodes::NodeValue::DescriptionList => {}
        ::nodes::NodeValue::DescriptionItem(_ndi) => {}
        ::nodes::NodeValue::DescriptionTerm => {}
        ::nodes::NodeValue::DescriptionDetails => {}
        ::nodes::NodeValue::CodeBlock(ncb) => {
            let _: bool = ncb.fenced;
            let _: u8 = ncb.fence_char;
            let _: usize = ncb.fence_length;
            let _: Vec<u8> = ncb.info;
            let _: Vec<u8> = ncb.literal;
        }
        ::nodes::NodeValue::HtmlBlock(nhb) => {
            let _: Vec<u8> = nhb.literal;
        }
        ::nodes::NodeValue::Paragraph => {}
        ::nodes::NodeValue::Heading(nh) => {
            let _: u32 = nh.level;
            let _: bool = nh.setext;
        }
        ::nodes::NodeValue::ThematicBreak => {}
        ::nodes::NodeValue::FootnoteDefinition(name) => {
            let _: &Vec<u8> = name;
        }
        ::nodes::NodeValue::Table(aligns) => {
            let _: &Vec<::nodes::TableAlignment> = aligns;
            match aligns[0] {
                ::nodes::TableAlignment::None => {}
                ::nodes::TableAlignment::Left => {}
                ::nodes::TableAlignment::Center => {}
                ::nodes::TableAlignment::Right => {}
            }
        }
        ::nodes::NodeValue::TableRow(header) => {
            let _: &bool = header;
        }
        ::nodes::NodeValue::TableCell => {}
        ::nodes::NodeValue::Text(text) => {
            let _: &Vec<u8> = text;
        }
        ::nodes::NodeValue::TaskItem(checked) => {
            let _: &bool = checked;
        }
        ::nodes::NodeValue::SoftBreak => {}
        ::nodes::NodeValue::LineBreak => {}
        ::nodes::NodeValue::Code(code) => {
            let _: usize = code.num_backticks;
            let _: Vec<u8> = code.literal;
        }
        ::nodes::NodeValue::HtmlInline(html) => {
            let _: &Vec<u8> = html;
        }
        ::nodes::NodeValue::Emph => {}
        ::nodes::NodeValue::Strong => {}
        ::nodes::NodeValue::Strikethrough => {}
        ::nodes::NodeValue::Superscript => {}
        ::nodes::NodeValue::Link(nl) | ::nodes::NodeValue::Image(nl) => {
            let _: Vec<u8> = nl.url;
            let _: Vec<u8> = nl.title;
        }
        ::nodes::NodeValue::FootnoteReference(name) => {
            let _: &Vec<u8> = name;
        }
    }
}
