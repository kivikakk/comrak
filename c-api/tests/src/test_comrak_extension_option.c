#include <string.h>
#include <stdio.h>

#include "../../include/comrak.h"
#include "deps/picotest/picotest.h"
#include "test.h"
#include "test_util.h"

void test_commonmark_render_works_with_strikethrough() {
    const char* commonmark = "Hello ~~world~~ 世界!";
    comrak_options_t * comrak_options = comrak_options_new();

    const char* html = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected = "<p>Hello ~~world~~ 世界!</p>\n";

    c_str_eq(html, expected);

    comrak_set_extension_option_strikethrough(comrak_options, true);

    const char* html_w_extension = comrak_commonmark_to_html(commonmark, comrak_options);
    const char* expected_w_extension = "<p>Hello <del>world</del> 世界!</p>\n";

    c_str_eq(html_w_extension, expected_w_extension);

    comrak_options_free(comrak_options);
}

void test_commonmark_extension_option() {
    test_commonmark_render_works_with_strikethrough();
}
