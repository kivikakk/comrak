use libc::c_char;

use comrak::{
    ComrakExtensionOptions, ComrakOptions, ComrakParseOptions, ComrakPlugins, ComrakRenderOptions,
};
use std::ffi::CStr;
use std::ptr;

/// Render Commonmark to HTML, with the given options.
#[no_mangle]
pub extern "C" fn comrak_commonmark_to_html(
    c_md: *mut c_char,
    c_comrak_options: *mut ComrakOptions,
) -> *mut c_char {
    // Convert C string to Rust string
    let md = unsafe {
        assert!(!c_md.is_null());
        CStr::from_ptr(c_md).to_str().unwrap()
    };

    let comrak_options = to_ref_mut!(c_comrak_options);

    // .is_null() {
    //     ComrakOptions::default()
    // } else {
    //     let comrak_options_extension = unsafe {
    //         let c_comrak_options_extension = (*c_comrak_options).extension;
    //         if c_comrak_options_extension.is_null() {
    //             ComrakExtensionOptions::default()
    //         } else {
    //             println!("OH DANG: {:}", (*c_comrak_options_extension).strikethrough);
    //             ComrakExtensionOptions {
    //                 strikethrough: (*c_comrak_options_extension).strikethrough,
    //                 tagfilter: (*c_comrak_options_extension).tagfilter,
    //                 table: (*c_comrak_options_extension).table,
    //                 autolink: (*c_comrak_options_extension).autolink,
    //                 tasklist: (*c_comrak_options_extension).tasklist,
    //                 superscript: (*c_comrak_options_extension).superscript,
    //                 header_ids: convert_c_str_to_string((*c_comrak_options_extension).header_ids),
    //                 footnotes: (*c_comrak_options_extension).footnotes,
    //                 description_lists: (*c_comrak_options_extension).description_lists,
    //                 front_matter_delimiter: convert_c_str_to_string(
    //                     (*c_comrak_options_extension).front_matter_delimiter,
    //                 ),
    //             }
    //         }
    //     };

    //     let comrak_options_parse = unsafe {
    //         let c_comrak_options_parse = (*c_comrak_options).parse;
    //         if c_comrak_options_parse.is_null() {
    //             ComrakParseOptions::default()
    //         } else {
    //             ComrakParseOptions {
    //                 smart: (*c_comrak_options_parse).smart,
    //                 default_info_string: convert_c_str_to_string(
    //                     (*c_comrak_options_parse).default_info_string,
    //                 ),
    //             }
    //         }
    //     };

    //     let comrak_options_render = unsafe {
    //         let c_comrak_options_render = (*c_comrak_options).render;
    //         if c_comrak_options_render.is_null() {
    //             ComrakRenderOptions::default()
    //         } else {
    //             ComrakRenderOptions {
    //                 hardbreaks: (*c_comrak_options_render).hardbreaks,
    //                 github_pre_lang: (*c_comrak_options_render).github_pre_lang,
    //                 width: (*c_comrak_options_render).width,
    //                 unsafe_: (*c_comrak_options_render).unsafe_,
    //                 escape: (*c_comrak_options_render).escape,
    //             }
    //         }
    //     };

    // ComrakOptions {
    //     extension: comrak_options_extension,
    //     parse: ComrakParseOptions::default(),
    //     render: ComrakRenderOptions::default(),
    // }
    // };

    let result = comrak::markdown_to_html(md, &comrak_options);

    // Convert to C string and return
    Box::into_raw(result.into_boxed_str()) as *mut c_char
}

fn convert_c_str_to_string(c_str: *const c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(c_str).to_string_lossy().into_owned() })
    }
}
