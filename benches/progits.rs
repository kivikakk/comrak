#![feature(test)]

extern crate test;

use comrak::{format_html, parse_document, Arena, Options};
use divan::Bencher;
use glob::glob;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_progits(b: Bencher) {
    let mut s = String::with_capacity(12_000_000);
    for entry in glob("vendor/progit/*/*/*.markdown").unwrap() {
        let file = File::open(entry.unwrap()).unwrap();
        let mut br = BufReader::new(file);
        br.read_to_string(&mut s).unwrap();
    }

    b.bench(|| {
        let arena = Arena::new();
        let root = parse_document(&arena, &s, &Options::default());
        let mut output = String::new();
        format_html(root, &Options::default(), &mut output).unwrap()
    });
}
