use super::*;

use comrak::ComrakOptions;

use crate::comrak_extension_options::FFIComrakExtensionOptions;
use crate::comrak_parse_options::FFIComrakParseOptions;
use crate::comrak_render_options::FFIComrakRenderOptions;

#[repr(C)]
pub struct FFIComrakOptions {
    pub extension: *mut FFIComrakExtensionOptions,
    pub parse: *mut FFIComrakParseOptions,
    pub render: *mut FFIComrakRenderOptions,
}

#[no_mangle]
pub extern "C" fn comrak_options_new() -> *mut ComrakOptions {
    let def = ComrakOptions::default();
    to_ptr_mut(ComrakOptions::default())
}

#[no_mangle]
pub extern "C" fn comrak_options_free(options: *mut ComrakOptions) {
    assert!(!options.is_null());
    drop(unsafe { Box::from_raw(options) });
}
