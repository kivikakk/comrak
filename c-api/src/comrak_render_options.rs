use libc::size_t;

#[repr(C)]
pub struct FFIComrakRenderOptions {
    hardbreaks: bool,
    github_pre_lang: bool,
    width: size_t,
    unsafe_: bool,
    escape: bool,
}
