use super::*;

use comrak::ComrakOptions;

#[no_mangle]
pub extern "C" fn comrak_options_new() -> *mut ComrakOptions {
    to_ptr_mut(ComrakOptions::default())
}

#[no_mangle]
pub extern "C" fn comrak_options_free(options: *mut ComrakOptions) {
    assert!(!options.is_null());
    drop(unsafe { Box::from_raw(options) });
}
