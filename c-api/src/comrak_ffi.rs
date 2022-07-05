use super::*;

use libc::c_char;

use comrak::{
    ComrakExtensionOptions, ComrakOptions, ComrakParseOptions, ComrakPlugins, ComrakRenderOptions,
};
use std::ffi::CStr;

/// Render Commonmark to HTML, with the given options.
#[no_mangle]
pub extern "C" fn comrak_commonmark_to_html(
    c_md: *mut c_char,
    c_comrak_options: *mut ComrakOptions,
) -> Str {
    // Convert C string to Rust string
    let md = unsafe {
        assert!(!c_md.is_null());
        CStr::from_ptr(c_md).to_str().unwrap()
    };

    let comrak_options = to_ref_mut!(c_comrak_options);

    let result = comrak::markdown_to_html(md, &comrak_options);

    // return as fat pointer string
    Str::new(result)
}

fn convert_c_str_to_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(c_str).to_string_lossy().into_owned() })
    }
}
