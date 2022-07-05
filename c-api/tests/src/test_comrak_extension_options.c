#include <string.h>
#include <stdio.h>

#include "../../include/comrak.h"
#include "deps/picotest/picotest.h"
#include "test.h"
#include "test_util.h"

void test_commonmark_render_works_with_strikethrough() {
    const char* commonmark = "Hello ~~world~~ 世界!";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_strikethrough(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>Hello ~~world~~ 世界!</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_strikethrough(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p>Hello <del>world</del> 世界!</p>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_tagfilter() {
    const char* commonmark = "hi <xmp> ok\n\n<xmp>\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_tagfilter(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>Hello ~~world~~ 世界!</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_tagfilter(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p>Hello <del>world</del> 世界!</p>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_table() {
    const char* commonmark = "| a | b |\n|---|:-:|\n| c | d |\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_table(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>| a | b |\n|---|:-:|\n| c | d |</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_table(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<table>\n<thead>\n<tr>\n<th>a</th>\n<th align=\"center\">b</th>\n</tr>\n</thead>\n<tbody>\n<tr>\n<td>c</td>\n<td align=\"center\">d</td>\n</tr>\n</tbody>\n</table>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_autolink() {
    const char* commonmark = "www.autolink.com\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_autolink(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>www.autolink.com</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_autolink(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p><a href=\"http://www.autolink.com\">www.autolink.com</a></p>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_tasklist() {
    const char* commonmark = "- [ ] List item 1\n- [ ] This list item is **bold**\n- [x] There is some `code` here\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_tasklist(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>- [ ] List item 1\n- [ ] This list item is <strong>bold</strong>\n- [x] There is some <code>code</code> here</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_tasklist(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<ul>\n<li><input type=\"checkbox\" disabled=\"\" /> List item 1</li>\n<li><input type=\"checkbox\" disabled=\"\" /> This list item is <strong>bold</strong></li>\n<li><input type=\"checkbox\" disabled=\"\" checked=\"\" /> There is some <code>code</code> here</li>\n</ul>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_superscript() {
    const char* commonmark = "e = mc^2^.\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_superscript(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>e = mc^2^.</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_superscript(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p>e = mc<sup>2</sup>.</p>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_header_ids() {
    const char* commonmark = "# Hi.\n## Hi 1.\n### Hi.\n#### Hello.\n##### Hi.\n###### Hello.\n# Isn't it grand?";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_header_ids(comrak_options, "user-content-", 13);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<h1><a href=\"#hi\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi\"></a>Hi.</h1>\n<h2><a href=\"#hi-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-1\"></a>Hi 1.</h2>\n<h3><a href=\"#hi-2\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-2\"></a>Hi.</h3>\n<h4><a href=\"#hello\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello\"></a>Hello.</h4>\n<h5><a href=\"#hi-3\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-3\"></a>Hi.</h5>\n<h6><a href=\"#hello-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello-1\"></a>Hello.</h6>\n<h1><a href=\"#isnt-it-grand\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-isnt-it-grand\"></a>Isn't it grand?</h1>\n";

    str_eq(html, expected);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
}

void test_commonmark_render_works_with_footnotes() {
    const char* commonmark = "Here is a[^nowhere] footnote reference,[^1] and another.[^longnote]\n\nThis is another note.[^note]\n\n[^note]: Hi.\n\n[^1]: Here is the footnote.\n\n[^longnote]: Here's one with multiple blocks.\n\n    Subsequent paragraphs are indented.\n\n        code\n\nThis is regular content.\n\n[^unused]: This is not used.\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_footnotes(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>Here is a[^nowhere] footnote reference,[^1] and another.[^longnote]</p>\n<p>This is another note.<a href=\"Hi.\">^note</a></p>\n<p>[^1]: Here is the footnote.</p>\n<p>[^longnote]: Here's one with multiple blocks.</p>\n<pre><code>Subsequent paragraphs are indented.\n\n    code\n</code></pre>\n<p>This is regular content.</p>\n<p>[^unused]: This is not used.</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_footnotes(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p>Here is a[^nowhere] footnote reference,<sup class=\"footnote-ref\"><a href=\"#fn1\" id=\"fnref1\">1</a></sup> and another.<sup class=\"footnote-ref\"><a href=\"#fn2\" id=\"fnref2\">2</a></sup></p>\n<p>This is another note.<sup class=\"footnote-ref\"><a href=\"#fn3\" id=\"fnref3\">3</a></sup></p>\n<p>This is regular content.</p>\n<section class=\"footnotes\">\n<ol>\n<li id=\"fn1\">\n<p>Here is the footnote. <a href=\"#fnref1\" class=\"footnote-backref\">↩</a></p>\n</li>\n<li id=\"fn2\">\n<p>Here's one with multiple blocks.</p>\n<p>Subsequent paragraphs are indented.</p>\n<pre><code>code\n</code></pre>\n<a href=\"#fnref2\" class=\"footnote-backref\">↩</a>\n</li>\n<li id=\"fn3\">\n<p>Hi. <a href=\"#fnref3\" class=\"footnote-backref\">↩</a></p>\n</li>\n</ol>\n</section>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}


void test_commonmark_render_works_with_description_lists() {
    const char* commonmark = "Term 1\n\n: Definition 1\n\nTerm 2 with *inline markup*\n\n: Definition 2\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_extension_option_description_lists(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>Term 1</p>\n<p>: Definition 1</p>\n<p>Term 2 with <em>inline markup</em></p>\n<p>: Definition 2</p>\n";

    str_eq(html, expected);

    comrak_set_extension_option_description_lists(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<dl><dt>Term 1</dt>\n<dd>\n<p>Definition 1</p>\n</dd>\n<dt>Term 2 with <em>inline markup</em></dt>\n<dd>\n<p>Definition 2</p>\n</dd>\n</dl>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_extension_options() {
    test_commonmark_render_works_with_strikethrough();
    // test_commonmark_render_works_with_tagfilter(); TODO
    test_commonmark_render_works_with_table();
    test_commonmark_render_works_with_autolink();
    // test_commonmark_render_works_with_tasklist(); TODO
    test_commonmark_render_works_with_superscript();
    test_commonmark_render_works_with_header_ids();
    test_commonmark_render_works_with_footnotes();
    test_commonmark_render_works_with_description_lists();
    // test_commonmark_render_works_with_front_matter_delimiter(); TODO
}
