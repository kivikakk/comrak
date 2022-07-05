use super::*;

use comrak::ComrakOptions;
use libc::{c_char, size_t};

use paste::paste;

make_bool_option_func!(parse, smart);
make_c_char_option_func!(parse, default_info_string);
