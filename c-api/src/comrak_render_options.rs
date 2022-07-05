use comrak::ComrakOptions;
use libc::size_t;

use paste::paste;

make_bool_option_func!(render, hardbreaks);
make_bool_option_func!(render, github_pre_lang);
make_size_t_option_func!(render, width);
make_bool_option_func!(render, unsafe_);
make_bool_option_func!(render, escape);
