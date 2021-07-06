//! This crate provides C ABI interface for [comrak](https://crates.io/crate/comrak) crate.
//!
//! # Bindings generation
//!
//! Among library creation this crate generates `comrak_ffi.h` file, enabled by default by `cbindgen` feature,
//! which might be useful for automatic bindings generation or just with plain `C`/`C++` development.

#![allow(clippy::missing_safety_doc)]

extern crate comrak as comrak_lib;

use comrak_lib::Arena;
use comrak_lib::ComrakOptions;
use comrak_lib::parse_document;
use comrak_lib::nodes::AstNode;

extern crate libc;

use libc::{c_char, c_void};

use std::ffi::{CStr, CString};

/// Render Markdown to HTML.
#[no_mangle]
pub unsafe extern "C" fn comrak_markdown_to_html(c_md: *mut c_char) -> *mut c_char {
    // Convert C string to Rust string
    assert!(!c_md.is_null());
    let md = CStr::from_ptr(c_md).to_str().unwrap();

    let result = comrak_lib::markdown_to_html(md, &ComrakOptions::default());

    CString::from_vec_unchecked(result.into_bytes()).into_raw()
}

// #[repr(C)]
// pub struct ComrakNode<'a, T: 'a> {
//     parent: Cell<Option<&'a ComrakNode<'a, T>>>,
//     previous_sibling: Cell<Option<&'a ComrakNode<'a, T>>>,
//     next_sibling: Cell<Option<&'a ComrakNode<'a, T>>>,
//     first_child: Cell<Option<&'a ComrakNode<'a, T>>>,
//     last_child: Cell<Option<&'a ComrakNode<'a, T>>>,

//     /// The data held by the node.
//     pub data: *mut c_void,
// }
type AstNodeArena<'a> = Arena<AstNode<'a>>;

#[no_mangle]
pub unsafe extern "C" fn comrak_new_arena<'a>() -> *mut AstNodeArena<'a>{
    let arena = Arena::new();
    let boxed: Box<AstNodeArena> = Box::new(arena);
    Box::into_raw(boxed)
}

/// Parse Markdown to tree.
#[no_mangle]
pub unsafe extern "C" fn comrak_parse<'a>(c_arena: *mut AstNodeArena<'a>, c_md: *mut c_char) -> *mut &'a AstNode<'a> {
    // Convert C string to Rust string
    assert!(!c_md.is_null());
    let md = CStr::from_ptr(c_md).to_str().unwrap();

    let root = parse_document(&*c_arena, md, &ComrakOptions::default());

    let clone_root = &*root.clone();

    let boxed: Box<&AstNode> = Box::new(clone_root);

    Box::into_raw(boxed)
}

#[no_mangle]
pub unsafe extern "C" fn comrak_str_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    CString::from_raw(ptr);
}
