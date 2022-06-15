use libc::c_char;

#[repr(C)]
pub struct FFIComrakParseOptions {
    smart: bool,
    default_info_string: *mut c_char,
}
