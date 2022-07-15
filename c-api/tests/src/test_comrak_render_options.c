#include <string.h>
#include <stdio.h>

#include "../../include/comrak_ffi.h"
#include "deps/picotest/picotest.h"
#include "test.h"
#include "test_util.h"

void test_commonmark_render_works_with_hardbreaks() {
    const char* commonmark = ">\\\n A";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_render_option_hardbreaks(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<blockquote>\n<p><br />\nA</p>\n</blockquote>\n";

    str_eq(html, expected);

    comrak_set_render_option_hardbreaks(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<blockquote>\n<p><br />\nA</p>\n</blockquote>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_github_pre_lang() {
    const char* commonmark = "``` rust\nfn hello();\n```\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_render_option_github_pre_lang(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n";

    str_eq(html, expected);

    comrak_set_render_option_github_pre_lang(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<pre lang=\"rust\"><code>fn hello();\n</code></pre>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_width() {
    const char* commonmark = "hello hello hello hello hello hello";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_str_t roundtrip = comrak_commonmark_to_commonmark(commonmark, comrak_options);
    const char* expected = "hello hello hello hello hello hello\n";

    str_eq(roundtrip, expected);

    comrak_set_render_option_width(comrak_options, 20);
    comrak_str_t roundtrip_w_width = comrak_commonmark_to_commonmark(commonmark, comrak_options);
    const char* expected_w_width ="hello hello hello\nhello hello hello\n";

    str_eq(roundtrip_w_width, expected_w_width);

    comrak_options_free(comrak_options);
    comrak_str_free(roundtrip);
    comrak_str_free(roundtrip_w_width);
}

void test_commonmark_render_works_with_unsafe_() {
    const char* commonmark = "<script>\nalert('xyz');\n</script>";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_render_option_unsafe_(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<!-- raw HTML omitted -->\n";

    str_eq(html, expected);

    comrak_set_render_option_unsafe_(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<script>\nalert(\'xyz\');\n</script>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_works_with_escape() {
    const char* commonmark = "<i>italic text</i>";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_render_option_escape(comrak_options, false);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected ="<p><!-- raw HTML omitted -->italic text<!-- raw HTML omitted --></p>\n";

    str_eq(html, expected);

    comrak_set_render_option_escape(comrak_options, true);
    comrak_str_t html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p>&lt;i&gt;italic text&lt;/i&gt;</p>\n";

    str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
    comrak_str_free(html_w_extension);
}

void test_commonmark_render_options() {
    test_commonmark_render_works_with_hardbreaks();
    test_commonmark_render_works_with_github_pre_lang();
    test_commonmark_render_works_with_width();
    test_commonmark_render_works_with_unsafe_();
    test_commonmark_render_works_with_escape();
}
