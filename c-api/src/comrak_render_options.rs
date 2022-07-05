use super::*;

use comrak::ComrakOptions;
use libc::{c_char, size_t};

use paste::paste;

#[repr(C)]
pub struct FFIComrakRenderOptions {
    hardbreaks: bool,
    github_pre_lang: bool,
    width: size_t,
    unsafe_: bool,
    escape: bool,
}


macro_rules! make_bool_render_option_func {
    ($name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ render_ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                value: bool,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);

                comrak_options.render.$name = value;
            }
        }
    };
}

macro_rules! make_size_t_render_option_func {
    ($name:ident) => {
        paste! {
            #[no_mangle]
            pub extern "C" fn [<comrak_ set_ render_ option_ $name>](
                c_comrak_options: *mut ComrakOptions,
                value: size_t,
            ) {
                let comrak_options = to_ref_mut!(c_comrak_options);

                comrak_options.render.$name = value;
            }
        }
    };
}

make_bool_render_option_func!(hardbreaks);
make_bool_render_option_func!(github_pre_lang);
make_size_t_render_option_func!(width);
make_bool_render_option_func!(unsafe_);
make_bool_render_option_func!(escape);
