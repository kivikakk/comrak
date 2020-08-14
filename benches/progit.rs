#![feature(test)]

extern crate comrak;
extern crate test;

use comrak::{format_html, parse_document, Arena, ComrakOptions};
use test::Bencher;

#[bench]
fn bench_progit(b: &mut Bencher) {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open("script/progit.md").unwrap();
    let mut s = String::with_capacity(524288);
    file.read_to_string(&mut s).unwrap();
    b.iter(|| {
        let arena = Arena::new();
        let root = parse_document(&arena, &s, &ComrakOptions::default());
        let mut output = vec![];
        format_html(root, &ComrakOptions::default(), &mut output).unwrap()
    });
}
