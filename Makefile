src/scanners.rs: src/scanners.re
	re2rust -W -Werror --case-insensitive -i --no-generation-date -8 --encoding-policy substitute -o $@ $<

bench:
	cargo build --release
	(cd vendor/cmark-gfm/; make bench PROG=../../target/release/comrak)
