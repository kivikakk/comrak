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

// void test_commonmark_render_works_with_default_info_string() {
//     const char* commonmark = "# Hi.\n## Hi 1.\n### Hi.\n#### Hello.\n##### Hi.\n###### Hello.\n# Isn't it grand?";
//     comrak_options_t * comrak_options = comrak_options_new();

//     comrak_set_parse_option_default_info_string(comrak_options, "user-content-", 13);
//     comrak_str_t html = comrak_commonmark_to_html(commonmark, comrak_options);
//     const char* expected = "<h1><a href=\"#hi\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi\"></a>Hi.</h1>\n<h2><a href=\"#hi-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-1\"></a>Hi 1.</h2>\n<h3><a href=\"#hi-2\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-2\"></a>Hi.</h3>\n<h4><a href=\"#hello\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello\"></a>Hello.</h4>\n<h5><a href=\"#hi-3\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hi-3\"></a>Hi.</h5>\n<h6><a href=\"#hello-1\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-hello-1\"></a>Hello.</h6>\n<h1><a href=\"#isnt-it-grand\" aria-hidden=\"true\" class=\"anchor\" id=\"user-content-isnt-it-grand\"></a>Isn't it grand?</h1>\n";

//     str_eq(html, expected);

//     comrak_options_free(comrak_options);
//     comrak_str_free(html);
// }

void test_commonmark_parse_options() {
    test_commonmark_render_works_with_smart();
    // test_commonmark_render_works_with_default_info_string(); TODO
}
