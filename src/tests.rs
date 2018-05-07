use {parse_document, Arena, ComrakOptions};
use cm;
use html;
#[cfg(feature = "benchmarks")]
use test::Bencher;
use timebomb::timeout_ms;

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

fn html(input: &str, expected: &str) {
    html_opts(input, expected, |_| ());
}

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

#[cfg(feature = "benchmarks")]
#[cfg_attr(feature = "benchmarks", bench)]
fn bench_progit(b: &mut Bencher) {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open("script/progit.md").unwrap();
    let mut s = String::with_capacity(524288);
    file.read_to_string(&mut s).unwrap();
    b.iter(|| {
        let arena = Arena::new();
        let root = parse_document(&arena, &s, &ComrakOptions::default());
        let mut output = vec![];
        html::format_document(root, &ComrakOptions::default(), &mut output).unwrap()
    });
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
    html(
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
    html(
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
    html(
        concat!(" <? o\n", "k ?> *a*\n", "*a*\n"),
        concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"),
    );
}

#[test]
fn html_block_4() {
    html(
        concat!("<!X >\n", "ok\n", "<!X\n", "um > h\n", "ok\n"),
        concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"),
    );
}

#[test]
fn html_block_5() {
    html(
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
    html(
        concat!(" </table>\n", "*x*\n", "\n", "ok\n", "\n", "<li\n", "*x*\n"),
        concat!(" </table>\n", "*x*\n", "<p>ok</p>\n", "<li\n", "*x*\n"),
    );
}

#[test]
fn html_block_7() {
    html(
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

    html(
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
    html(
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
fn strikethrough() {
    html_opts(
        concat!(
            "This is ~strikethrough~.\n",
            "\n",
            "As is ~~this, okay~~?\n"
        ),
        concat!(
            "<p>This is <del>strikethrough</del>.</p>\n",
            "<p>As is <del>this, okay</del>?</p>\n"
        ),
        |opts| opts.ext_strikethrough = true,
    );
}

#[test]
fn table() {
    html_opts(
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
            "</tr></tbody></table>\n"
        ),
        |opts| opts.ext_table = true,
    );
}

#[test]
fn autolink_www() {
    html_opts(
        concat!("www.autolink.com\n"),
        concat!("<p><a href=\"http://www.autolink.com\">www.autolink.com</a></p>\n"),
        |opts| opts.ext_autolink = true,
    );
}

#[test]
fn autolink_email() {
    html_opts(
        concat!("john@smith.com\n"),
        concat!("<p><a href=\"mailto:john@smith.com\">john@smith.com</a></p>\n"),
        |opts| opts.ext_autolink = true,
    );
}

#[test]
fn autolink_scheme() {
    html_opts(
        concat!("https://google.com/search\n"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a></p>\n"
        ),
        |opts| opts.ext_autolink = true,
    );
}

#[test]
fn autolink_scheme_multiline() {
    html_opts(
        concat!("https://google.com/search\nhttps://www.google.com/maps"),
        concat!(
            "<p><a href=\"https://google.com/search\">https://google.\
             com/search</a>\n<a href=\"https://www.google.com/maps\">\
             https://www.google.com/maps</a></p>\n"
        ),
        |opts| opts.ext_autolink = true,
    );
}

#[test]
fn tagfilter() {
    html_opts(
        concat!("hi <xmp> ok\n", "\n", "<xmp>\n"),
        concat!("<p>hi &lt;xmp> ok</p>\n", "&lt;xmp>\n"),
        |opts| opts.ext_tagfilter = true,
    );
}

#[test]
fn tasklist() {
    html_opts(
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
        |opts| opts.ext_tasklist = true,
    );
}

#[test]
fn tasklist_32() {
    html_opts(
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
        |opts| opts.ext_tasklist = true,
    );
}

#[test]
fn superscript() {
    html_opts(
        concat!("e = mc^2^.\n"),
        concat!("<p>e = mc<sup>2</sup>.</p>\n"),
        |opts| opts.ext_superscript = true,
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
            "###### Hello.\n"
        ),
        concat!(
            "<h1><a href=\"#hi\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi\"></a>Hi.</h1>\n",
            "<h2><a href=\"#hi-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-1\"></a>Hi 1.</h2>\n",
            "<h3><a href=\"#hi-2\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-2\"></a>Hi.</h3>\n",
            "<h4><a href=\"#hello\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello\"></a>Hello.</h4>\n",
            "<h5><a href=\"#hi-3\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-3\"></a>Hi.</h5>\n",
            "<h6><a href=\"#hello-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello-1\"></a>Hello.</h6>\n"
        ),
        |opts| opts.ext_header_ids = Some("user-content-".to_owned()),
    );
}

#[test]
fn footnotes() {
    html_opts(
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
             id=\"fnref1\">[1]</a></sup> and another.<sup class=\"footnote-ref\"><a \
             href=\"#fn2\" id=\"fnref2\">[2]</a></sup></p>\n",
            "<p>This is another note.<sup class=\"footnote-ref\"><a href=\"#fn3\" \
             id=\"fnref3\">[3]</a></sup></p>\n",
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
        |opts| opts.ext_footnotes = true,
    );
}

#[test]
fn footnote_does_not_eat_exclamation() {
    html_opts(
        concat!("Here's my footnote![^a]\n", "\n", "[^a]: Yep.\n"),
        concat!(
            "<p>Here's my footnote!<sup class=\"footnote-ref\"><a href=\"#fn1\" \
             id=\"fnref1\">[1]</a></sup></p>\n",
            "<section class=\"footnotes\">\n",
            "<ol>\n",
            "<li id=\"fn1\">\n",
            "<p>Yep. <a href=\"#fnref1\" class=\"footnote-backref\">↩</a></p>\n",
            "</li>\n",
            "</ol>\n",
            "</section>\n"
        ),
        |opts| opts.ext_footnotes = true,
    );
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
    html(
        "#  #",
        "<h1></h1>\n"
    );
}

#[test]
fn table_misparse_1() {
    html_opts(
        "a\n-b",
        "<p>a\n-b</p>\n",
        |opts| opts.ext_table = true,
    );
}

#[test]
fn table_misparse_2() {
    html_opts(
        "a\n-b\n-c",
        "<p>a\n-b\n-c</p>\n",
        |opts| opts.ext_table = true,
    );
}

#[test]
fn smart_chars() {
    html_opts(
        "Why 'hello' \"there\". It's good.",
        "<p>Why ‘hello’ “there”. It’s good.</p>\n",
        |opts| opts.smart = true,
    );

    html_opts(
        "Hm. Hm.. hm... yes- indeed-- quite---!",
        "<p>Hm. Hm.. hm… yes- indeed– quite—!</p>\n",
        |opts| opts.smart = true,
    );
}

#[test]
fn nested_tables_1() {
    html_opts(
        concat!(
            "- p\n",
            "\n",
            "    |a|b|\n",
            "    |-|-|\n",
            "    |c|d|\n",
        ),
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
            "</tr></tbody></table>\n",
            "</li>\n",
            "</ul>\n",
        ),
        |opts| opts.ext_table = true,
    );
}

#[test]
fn nested_tables_2() {
    html_opts(
        concat!(
            "- |a|b|\n",
            "  |-|-|\n",
            "  |c|d|\n",
        ),
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
            "</tr></tbody></table>\n",
            "</li>\n",
            "</ul>\n",
        ),
        |opts| opts.ext_table = true,
    );
}

#[test]
fn nested_tables_3() {
    html_opts(
        concat!(
            "> |a|b|\n",
            "> |-|-|\n",
            "> |c|d|\n",
        ),
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
            "</tr></tbody></table>\n",
            "</blockquote>\n",
        ),
        |opts| opts.ext_table = true,
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
fn safe() {
    html_opts(
        concat!(
            "[data:png](data:png/x)\n\n",
            "[data:gif](data:gif/x)\n\n",
            "[data:jpeg](data:jpeg/x)\n\n",
            "[data:webp](data:webp/x)\n\n",
            "[data:malicious](data:malicious/x)\n\n",
            "[javascript:malicious](javascript:malicious)\n\n",
            "[vbscript:malicious](vbscript:malicious)\n\n",
            "[file:malicious](file:malicious)\n\n",
        ),
        concat!(
            "<p><a href=\"data:png/x\">data:png</a></p>\n",
            "<p><a href=\"data:gif/x\">data:gif</a></p>\n",
            "<p><a href=\"data:jpeg/x\">data:jpeg</a></p>\n",
            "<p><a href=\"data:webp/x\">data:webp</a></p>\n",
            "<p><a href=\"\">data:malicious</a></p>\n",
            "<p><a href=\"\">javascript:malicious</a></p>\n",
            "<p><a href=\"\">vbscript:malicious</a></p>\n",
            "<p><a href=\"\">file:malicious</a></p>\n",
        ),
        |opts| opts.safe = true,
    )
}
