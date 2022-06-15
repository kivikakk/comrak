#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include "deps/picotest/picotest.h"
#include "../../include/comrak.h"

#include "test.h"

int run_tests() {
    subtest("test_comrak_basic", test_comrak_basic);
    subtest("test_commonmark_extension_option", test_commonmark_extension_option);

    return done_testing();
}

