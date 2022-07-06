#include <string.h>
#include <stdio.h>

#include "../../include/comrak.h"
#include "deps/picotest/picotest.h"
#include "test.h"
#include "test_util.h"

void test_commonmark_render_works_with_smart() {
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

void test_commonmark_render_works_with_default_info_string() {
    const char* commonmark = "```\nfn hello();\n```\n";
    comrak_options_t * comrak_options = comrak_options_new();

    comrak_set_parse_option_default_info_string(comrak_options, "rust", 4);
    comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<pre><code class=\"language-rust\">fn hello();\n</code></pre>\n";

    str_eq(html, expected);

    comrak_options_free(comrak_options);
    comrak_str_free(html);
}

void test_commonmark_parse_options() {
    test_commonmark_render_works_with_smart();
    test_commonmark_render_works_with_default_info_string();
}
