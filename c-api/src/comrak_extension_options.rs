use super::*;
use libc::{c_char, size_t};

use comrak::ComrakOptions;
use comrak_options::FFIComrakOptions;

#[repr(C)]
pub struct FFIComrakExtensionOptions {
    strikethrough: bool,
    tagfilter: bool,
    table: bool,
    autolink: bool,
    tasklist: bool,
    superscript: bool,
    header_ids: *mut c_char,
    footnotes: bool,
    description_lists: bool,
    front_matter_delimiter: *mut c_char,
}

#[no_mangle]
pub extern "C" fn comrak_set_extension_option_strikethrough(
    c_comrak_options: *mut ComrakOptions,
    value: bool,
) {
    let comrak_options = to_ref_mut!(c_comrak_options);

    comrak_options.extension.strikethrough = value;
}
