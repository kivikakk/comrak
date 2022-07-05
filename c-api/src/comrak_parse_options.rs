use super::*;

use comrak::ComrakOptions;
use libc::{c_char, size_t};

use paste::paste;

#[repr(C)]
pub struct FFIComrakParseOptions {
    smart: bool,
    default_info_string: *const c_char,
}

macro_rules! make_bool_parse_option_func {
    ($name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ parse_ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                value: bool,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);

                comrak_options.parse.$name = value;
            }
        }
    };
}

macro_rules! make_c_char_parse_option_func {
    ($name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ parse_ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                v: *const c_char,
                v_len: size_t,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);
                let value = unwrap_or_ret_err_code! { to_str!(v, v_len) };

                comrak_options.parse.$name = Some(value.to_string());
            }
        }
    };
}

make_bool_parse_option_func!(smart);
make_c_char_parse_option_func!(default_info_string);
