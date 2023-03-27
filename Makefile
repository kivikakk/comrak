src/scanners.rs: src/scanners.re
	re2rust -W -Werror -i --no-generation-date -o $@ $<
	cargo fmt

bench:
	cargo build --release
	(cd vendor/cmark-gfm/; make bench PROG=../../target/release/comrak)
