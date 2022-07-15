#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include "deps/picotest/picotest.h"
#include "../../include/comrak_ffi.h"

#include "test.h"

int run_tests() {
    subtest("test_comrak_basic", test_comrak_basic);
    subtest("test_commonmark_extension_options", test_commonmark_extension_options);
    subtest("test_commonmark_parse_options", test_commonmark_parse_options);
    subtest("test_commonmark_render_options", test_commonmark_render_options);

    return done_testing();
}

