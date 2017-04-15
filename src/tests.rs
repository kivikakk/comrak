use ::{Arena, parse_document, ComrakOptions};
use ::html;
use ::cm;

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
    where F: Fn(&mut ComrakOptions)
{
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    opts(&mut options);

    let root = parse_document(&arena, &input.chars().collect::<String>(), &options);
    let output = html::format_document(root, &options);
    compare_strs(&output, expected, "regular");

    let md = cm::format_document(root, &options);
    let root = parse_document(&arena, &md.chars().collect::<String>(), &options);
    let output_from_rt = html::format_document(root, &options);
    compare_strs(&output_from_rt, expected, "roundtrip");
}

#[test]
fn basic() {
    html(concat!("My **document**.\n",
                 "\n",
                 "It's mine.\n",
                 "\n",
                 "> Yes.\n",
                 "\n",
                 "## Hi!\n",
                 "\n",
                 "Okay.\n"),
         concat!("<p>My <strong>document</strong>.</p>\n",
                 "<p>It's mine.</p>\n",
                 "<blockquote>\n",
                 "<p>Yes.</p>\n",
                 "</blockquote>\n",
                 "<h2>Hi!</h2>\n",
                 "<p>Okay.</p>\n"));
}

#[test]
fn codefence() {
    html(concat!("``` rust yum\n", "fn main<'a>();\n", "```\n"),
         concat!("<pre><code class=\"language-rust\">fn main&lt;'a&gt;();\n",
                 "</code></pre>\n"));
}

#[test]
fn lists() {
    html(concat!("2. Hello.\n", "3. Hi.\n"),
         concat!("<ol start=\"2\">\n",
                 "<li>Hello.</li>\n",
                 "<li>Hi.</li>\n",
                 "</ol>\n"));

    html(concat!("- Hello.\n", "- Hi.\n"),
         concat!("<ul>\n", "<li>Hello.</li>\n", "<li>Hi.</li>\n", "</ul>\n"));
}

#[test]
fn thematic_breaks() {
    html(concat!("---\n", "\n", "- - -\n", "\n", "\n", "_        _   _\n"),
         concat!("<hr />\n", "<hr />\n", "<hr />\n"));
}

#[test]
fn setext_heading() {
    html(concat!("Hi\n", "==\n", "\n", "Ok\n", "-----\n"),
         concat!("<h1>Hi</h1>\n", "<h2>Ok</h2>\n"));
}

#[test]
fn html_block_1() {
    html(concat!("<script\n",
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
                 "*ok*\n"),
         concat!("<script\n",
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
                 "<p><em>ok</em></p>\n"));
}

#[test]
fn html_block_2() {
    html(concat!("   <!-- abc\n", "\n", "ok --> *hi*\n", "*hi*\n"),
         concat!("   <!-- abc\n",
                 "\n",
                 "ok --> *hi*\n",
                 "<p><em>hi</em></p>\n"));
}

#[test]
fn html_block_3() {
    html(concat!(" <? o\n", "k ?> *a*\n", "*a*\n"),
         concat!(" <? o\n", "k ?> *a*\n", "<p><em>a</em></p>\n"));
}

#[test]
fn html_block_4() {
    html(concat!("<!X >\n", "ok\n", "<!X\n", "um > h\n", "ok\n"),
         concat!("<!X >\n", "<p>ok</p>\n", "<!X\n", "um > h\n", "<p>ok</p>\n"));
}

#[test]
fn html_block_5() {
    html(concat!("<![CDATA[\n",
                 "\n",
                 "hm >\n",
                 "*ok*\n",
                 "]]> *ok*\n",
                 "*ok*\n"),
         concat!("<![CDATA[\n",
                 "\n",
                 "hm >\n",
                 "*ok*\n",
                 "]]> *ok*\n",
                 "<p><em>ok</em></p>\n"));
}

#[test]
fn html_block_6() {
    html(concat!(" </table>\n", "*x*\n", "\n", "ok\n", "\n", "<li\n", "*x*\n"),
         concat!(" </table>\n", "*x*\n", "<p>ok</p>\n", "<li\n", "*x*\n"));
}

#[test]
fn html_block_7() {
    html(concat!("<a b >\n",
                 "ok\n",
                 "\n",
                 "<a b=>\n",
                 "ok\n",
                 "\n",
                 "<a b \n",
                 "<a b> c\n",
                 "ok\n"),
         concat!("<a b >\n",
                 "ok\n",
                 "<p>&lt;a b=&gt;\n",
                 "ok</p>\n",
                 "<p>&lt;a b\n",
                 "<a b> c\n",
                 "ok</p>\n"));

    html(concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "\n", "ok\n"),
         concat!("<a b c=x d='y' z=\"f\" >\n", "ok\n", "<p>ok</p>\n"));
}

#[test]
fn backticks() {
    html("Some `code\\` yep.\n",
         "<p>Some <code>code\\</code> yep.</p>\n");
}

#[test]
fn backslashes() {
    html(concat!("Some \\`fake code\\`.\n",
                 "\n",
                 "Some fake linebreaks:\\\n",
                 "Yes.\\\n",
                 "See?\n",
                 "\n",
                 "Ga\\rbage.\n"),
         concat!("<p>Some `fake code`.</p>\n",
                 "<p>Some fake linebreaks:<br />\n",
                 "Yes.<br />\n",
                 "See?</p>\n",
                 "<p>Ga\\rbage.</p>\n"));
}

#[test]
fn entities() {
    html(concat!("This is &amp;, &copy;, &trade;, \\&trade;, &xyz;, &NotEqualTilde;.\n",
                 "\n",
                 "&#8734; &#x221e;\n"),
         concat!("<p>This is &amp;, ©, ™, &amp;trade;, &amp;xyz;, \u{2242}\u{338}.</p>\n",
                 "<p>∞ ∞</p>\n"));
}

#[test]
fn pointy_brace() {
    html(concat!("URI autolink: <https://www.pixiv.net>\n",
                 "\n",
                 "Email autolink: <bill@microsoft.com>\n",
                 "\n",
                 "* Inline <em>tag</em> **ha**.\n",
                 "* Inline <!-- comment --> **ha**.\n",
                 "* Inline <? processing instruction ?> **ha**.\n",
                 "* Inline <!DECLARATION OKAY> **ha**.\n",
                 "* Inline <![CDATA[ok]ha **ha** ]]> **ha**.\n"),
         concat!("<p>URI autolink: <a \
                  href=\"https://www.pixiv.net\">https://www.pixiv.net</a></p>\n",
                 "<p>Email autolink: <a \
                  href=\"mailto:bill@microsoft.com\">bill@microsoft.com</a></p>\n",
                 "<ul>\n",
                 "<li>Inline <em>tag</em> <strong>ha</strong>.</li>\n",
                 "<li>Inline <!-- comment --> <strong>ha</strong>.</li>\n",
                 "<li>Inline <? processing instruction ?> <strong>ha</strong>.</li>\n",
                 "<li>Inline <!DECLARATION OKAY> <strong>ha</strong>.</li>\n",
                 "<li>Inline <![CDATA[ok]ha **ha** ]]> <strong>ha</strong>.</li>\n",
                 "</ul>\n"));
}

#[test]
fn links() {
    html(concat!("Where are you [going](https://microsoft.com (today))?\n",
                 "\n",
                 "[Where am I?](/here)\n"),
         concat!("<p>Where are you <a href=\"https://microsoft.com\" \
                  title=\"today\">going</a>?</p>\n",
                 "<p><a href=\"/here\">Where am I?</a></p>\n"));
}

#[test]
fn images() {
    html(concat!("I am ![eating [things](/url)](http://i.imgur.com/QqK1vq7.png).\n"),
         concat!("<p>I am <img src=\"http://i.imgur.com/QqK1vq7.png\" alt=\"eating things\" \
                  />.</p>\n"));
}

#[test]
fn reference_links() {
    html(concat!("This [is] [legit], [very][honestly] legit.\n",
                 "\n",
                 "[legit]: ok\n",
                 "[honestly]: sure \"hm\"\n"),
         concat!("<p>This [is] <a href=\"ok\">legit</a>, <a href=\"sure\" title=\"hm\">very</a> \
                  legit.</p>\n"));
}

#[test]
fn strikethrough() {
    html_opts(concat!("This is ~strikethrough~.\n",
                      "\n",
                      "As is ~~this, okay~~?\n"),
              concat!("<p>This is <del>strikethrough</del>.</p>\n",
                      "<p>As is <del>this, okay</del>?</p>\n"),
              |opts| opts.ext_strikethrough = true);
}

#[test]
fn table() {
    html_opts(concat!("| a | b |\n", "|---|:-:|\n", "| c | d |\n"),
              concat!("<table>\n",
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
                      "</tr></tbody></table>\n"),
              |opts| opts.ext_table = true);
}

#[test]
fn autolink_www() {
    html_opts(concat!("www.autolink.com\n"),
              concat!("<p><a href=\"http://www.autolink.com\">www.autolink.com</a></p>\n"),
              |opts| opts.ext_autolink = true);
}

#[test]
fn autolink_email() {
    html_opts(concat!("john@smith.com\n"),
              concat!("<p><a href=\"mailto:john@smith.com\">john@smith.com</a></p>\n"),
              |opts| opts.ext_autolink = true);
}

#[test]
fn autolink_scheme() {
    html_opts(concat!("https://google.com/search\n"),
              concat!("<p><a href=\"https://google.com/search\">https://google.\
                       com/search</a></p>\n"),
              |opts| opts.ext_autolink = true);
}

#[test]
fn tagfilter() {
    html_opts(concat!("hi <xmp> ok\n", "\n", "<xmp>\n"),
              concat!("<p>hi &lt;xmp> ok</p>\n", "&lt;xmp>\n"),
              |opts| opts.ext_tagfilter = true);
}

#[test]
fn tasklist() {
    html_opts(concat!("* [ ] Red\n",
                      "* [x] Green\n",
                      "* [ ] Blue\n",
                      "<!-- end list -->\n",
                      "1. [ ] Bird\n",
                      "2. [ ] McHale\n",
                      "3. [x] Parish\n",
                      "<!-- end list -->\n",
                      "* [ ] Red\n",
                      "  * [x] Green\n",
                      "    * [ ] Blue\n"),
              concat!("<ul>\n",
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
                      "</ul>\n"),
              |opts| opts.ext_tasklist = true);
}

#[test]
fn superscript() {
    html_opts(concat!("e = mc^2^.\n"),
              concat!("<p>e = mc<sup>2</sup>.</p>\n"),
              |opts| opts.ext_superscript = true);
}
