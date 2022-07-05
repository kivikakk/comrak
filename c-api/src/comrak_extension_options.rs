use super::*;
use libc::{c_char, size_t};

use comrak::ComrakOptions;

use paste::paste;

make_bool_option_func!(extension, strikethrough);
make_bool_option_func!(extension, tagfilter);
make_bool_option_func!(extension, table);
make_bool_option_func!(extension, autolink);
make_bool_option_func!(extension, tasklist);
make_bool_option_func!(extension, superscript);
make_c_char_option_func!(extension, header_ids);
make_bool_option_func!(extension, footnotes);
make_bool_option_func!(extension, description_lists);
make_c_char_option_func!(extension, front_matter_delimiter);
