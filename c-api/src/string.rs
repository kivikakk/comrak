use super::*;

use libc::{c_char, size_t};

// Can't use CStr and CString as the transfer type because UTF-8
// strings can contain "internal" NULLs.
#[repr(C)]
pub struct Str {
    data: *const c_char,
    len: size_t,
}

impl Str {
    pub fn new(string: String) -> Self {
        Str {
            len: string.len(),
            data: Box::into_raw(string.into_boxed_str()) as *const c_char,
        }
    }
}

impl Drop for Str {
    fn drop(&mut self) {
        if self.data == ptr::null() {
            return;
        }
        let bytes = unsafe { slice::from_raw_parts_mut(self.data as *mut c_char, self.len) };

        drop(unsafe { Box::from_raw(bytes) });
    }
}

#[no_mangle]
pub extern "C" fn comrak_str_free(string: Str) {
    drop(string);
}
