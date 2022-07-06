use super::*;
use libc::{c_char, size_t};

use comrak::ComrakOptions;

use paste::paste;

macro_rules! make_bool_option_func {
    ($opt_type:ident, $name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ $opt_type _ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                value: bool,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);

                comrak_options.$opt_type.$name = value;
            }
        }
    };
}

macro_rules! make_c_char_option_func {
    ($opt_type:ident, $name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ $opt_type _ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                v: *const c_char,
                v_len: size_t,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);
                let value = unwrap_or_ret_err_code! { to_str!(v, v_len) };

                comrak_options.$opt_type.$name = Some(value.to_string());
            }
        }
    };
}

macro_rules! make_size_t_option_func {
    ($opt_type:ident, $name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ $opt_type _ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                value: size_t,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);

                comrak_options.$opt_type.$name = value;
            }
        }
    };
}


#[no_mangle]
pub extern "C" fn comrak_options_new() -> *mut ComrakOptions {
    to_ptr_mut(ComrakOptions::default())
}

#[no_mangle]
pub extern "C" fn comrak_options_free(options: *mut ComrakOptions) {
    assert!(!options.is_null());
    drop(unsafe { Box::from_raw(options) });
}


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

make_bool_option_func!(parse, smart);
make_c_char_option_func!(parse, default_info_string);

make_bool_option_func!(render, hardbreaks);
make_bool_option_func!(render, github_pre_lang);
make_size_t_option_func!(render, width);
make_bool_option_func!(render, unsafe_);
make_bool_option_func!(render, escape);
