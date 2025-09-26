ROOT:=$(shell git rev-parse --show-toplevel)
COMMIT:=$(shell git rev-parse --short HEAD)
MIN_RUNS:=25

src/scanners.rs: src/scanners.re
	re2rust -W -Werror -i --no-generation-date -o $@ $<
	cargo fmt

bench:
	cargo build --release
	(cd vendor/cmark-gfm/; make bench PROG=../../target/release/comrak)

binaries: build-comrak-branch build-comrak-main build-cmark-gfm build-pulldown-cmark build-markdown-it

build-comrak-branch:
	cargo build --release
	cp ${ROOT}/target/release/comrak ${ROOT}/benches/comrak-${COMMIT}

build-comrak-main:
	git clone https://github.com/kivikakk/comrak.git --depth 1 --single-branch ${ROOT}/vendor/comrak || true
	cd ${ROOT}/vendor/comrak && \
	cargo build --release && \
	cp ./target/release/comrak ${ROOT}/benches/comrak-main

build-cmark-gfm:
	cd ${ROOT}/vendor/cmark-gfm && \
	make && \
	cp build/src/cmark-gfm ${ROOT}/benches/cmark-gfm

build-markdown-it:
	cd ${ROOT}/vendor/markdown-it && \
	cargo build --release && \
	cp target/release/markdown-it ${ROOT}/benches/markdown-it

build-pulldown-cmark:
	cd ${ROOT}/vendor/pulldown-cmark && \
	cargo build --release && \
	cp target/release/pulldown-cmark ${ROOT}/benches/pulldown-cmark

bench-comrak: build-comrak-branch
	git clone https://github.com/progit/progit.git ${ROOT}/vendor/progit || true > /dev/null
	cd benches && \
	hyperfine --warmup 3 --min-runs ${MIN_RUNS} -L binary comrak-${COMMIT} './bench.sh ./{binary}'

bench-comrak-vs-main: build-comrak-branch build-comrak-main
	git clone https://github.com/progit/progit.git ${ROOT}/vendor/progit || true > /dev/null
	cd benches && \
	hyperfine --warmup 10 --min-runs ${MIN_RUNS} -L binary comrak-${COMMIT},comrak-main './bench.sh ./{binary}' --export-markdown ${ROOT}/bench-output.md &&\
	echo "\n\nRun on" `date -u` >> ${ROOT}/bench-output.md

bench-all: binaries
	git clone https://github.com/progit/progit.git ${ROOT}/vendor/progit || true > /dev/null
	cd benches && \
	hyperfine --warmup 10 --min-runs ${MIN_RUNS} -L binary comrak-${COMMIT},comrak-main,pulldown-cmark,cmark-gfm,markdown-it './bench.sh ./{binary}' --export-markdown ${ROOT}/bench-output.md &&\
	echo "\n\nRun on" `date -u` >> ${ROOT}/bench-output.md

