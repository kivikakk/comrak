#ifndef COMRAK_H
#define COMRAK_H

#if defined(__cplusplus)
extern "C" {
#endif

#include <stddef.h>
#include <stdbool.h>

typedef struct comrak_ComrakOptions comrak_options_t;

// Creates new ComrakOptions struct.
comrak_options_t *comrak_options_new();

// Frees the memory allocated for ComrakOptions struct.
void comrak_options_free(comrak_options_t *options);

void comrak_set_extension_option_strikethrough(comrak_options_t *options, bool value);
void comrak_set_extension_option_tagfilter(comrak_options_t *options, bool value);
void comrak_set_extension_option_table(comrak_options_t *options, bool value);
void comrak_set_extension_option_autolink(comrak_options_t *options, bool value);
void comrak_set_extension_option_tasklist(comrak_options_t *options, bool value);
void comrak_set_extension_option_superscript(comrak_options_t *options, bool value);
void comrak_set_extension_option_header_ids(comrak_options_t *options, const char *header_id, size_t header_id_len);
void comrak_set_extension_option_footnotes(comrak_options_t *options, bool value);
void comrak_set_extension_option_description_lists(comrak_options_t *options, bool value);
void comrak_set_extension_option_front_matter_delimiter(comrak_options_t *options, const char *front_matter_delimiter, size_t front_matter_delimiter_len);

void comrak_set_parse_option_superscript(comrak_options_t *options, bool value);
void comrak_set_parse_option_default_info_string(comrak_options_t *options, const char *default_info_string, size_t default_info_string_len);

// Library-allocated UTF-8 string fat pointer.
//
// The string is not NULL-terminated.
//
// Use `comrak_str_free` function to deallocate.
typedef struct {
    // String data pointer.
    const char *data;

    // The length of the string in bytes.
    size_t len;
} comrak_str_t;

// Convert 'text' (assumed to be a UTF-8 encoded string) from Commonmark to HTML,
// returning a null-terminated, UTF-8-encoded string, using the options specified.
comrak_str_t comrak_commonmark_to_html(const char *text, comrak_options_t *options);

// Frees the memory held by the library-allocated string.
//
// This is valid to call even if `str.data == NULL` (it does nothing, like `free(NULL)`).
void comrak_str_free(comrak_str_t str);

#if defined(__cplusplus)
}  // extern C
#endif

#endif // PULLDOWN_CMCOMRAK_HARK_H
