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

/** Convert 'text' (assumed to be a UTF-8 encoded string) from CommonMark Markdown to HTML,
 * returning a null-terminated, UTF-8-encoded string, using the options specified.
 *
 * It is the caller's responsibility to free the returned buffer.
*/
char *comrak_commonmark_to_html(const char *text, comrak_options_t *options);

#if defined(__cplusplus)
}  // extern C
#endif

#endif // PULLDOWN_CMCOMRAK_HARK_H
