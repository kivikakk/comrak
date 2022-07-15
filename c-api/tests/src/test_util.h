#ifndef TEST_UTIL_H
#define TEST_UTIL_H

#include <stdlib.h>
#include <string.h>

#include "../../include/comrak_ffi.h"

#define str_eq(actual, expected) { \
    ok((actual).data != NULL); \
    ok((actual).len == strlen(expected)); \
    ok(!memcmp((actual).data, expected, (actual).len)); \
}

#endif // TEST_UTIL_H
