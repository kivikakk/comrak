use super::*;
use libc::{c_char, size_t};

use comrak::ComrakOptions;

use paste::paste;

#[repr(C)]
pub struct FFIComrakExtensionOptions {
    strikethrough: bool,
    tagfilter: bool,
    table: bool,
    autolink: bool,
    tasklist: bool,
    superscript: bool,
    header_ids: *const c_char,
    footnotes: bool,
    description_lists: bool,
    front_matter_delimiter: *const c_char,
}

macro_rules! make_bool_option_func {
    ($name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ extension_ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                value: bool,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);

                comrak_options.extension.$name = value;
            }
        }
    };
}

macro_rules! make_c_char_option_func {
    ($name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ extension_ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                v: *const c_char,
                v_len: size_t,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);
                let value = unwrap_or_ret_err_code! { to_str!(v, v_len) };

                comrak_options.extension.$name = Some(value.to_string());
            }
        }
    };
}

make_bool_option_func!(strikethrough);
make_bool_option_func!(tagfilter);
make_bool_option_func!(table);
make_bool_option_func!(autolink);
make_bool_option_func!(tasklist);
make_bool_option_func!(superscript);
make_c_char_option_func!(header_ids);
make_bool_option_func!(footnotes);
make_bool_option_func!(description_lists);
make_c_char_option_func!(front_matter_delimiter);
