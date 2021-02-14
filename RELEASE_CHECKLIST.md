* [ ] `rustup update stable`
* [ ] ensure `cargo +stable build --release --all-features` works
* [ ] bump version in Cargo.toml
  * [ ] did `tests::exercise_full_api` change? if so, it's a semver-breaking change.
* [ ] update changelog
* [ ] `script/update-readme`
* [ ] commit, tag, push commit, but do not push tag yet
* build binaries:
  * [ ] build `aarch64-apple-darwin` on tapioca
  * [ ] cross-compile `x86_64-apple-darwin` on tapioca
  * [ ] build `x86_64-pc-windows-msvc` on raven
  * [ ] build `x86_64-pc-windows-gnu` on raven
    * `rustup run stable-gnu cargo build --release --target=x86_64-pc-windows-gnu` does the trick. You may need to `rustup toolchain install stable-x86_64-pc-windows-gnu` first? Unclear.
  * [ ] cross-compile `armv7-unknown-linux-musleabihf` on tapioca
    * `brew install arm-linux-gnueabihf-binutils` and add something like this to `~/.cargo/config.toml`:
    
      ```toml
      [build]

      [target.armv7-unknown-linux-musleabihf]
      linker = "/usr/local/bin/arm-linux-gnueabihf-ld"
      ```
    * [ ] test on muffin
  * [ ] build `x86_64-unknown-linux-gnu` on ishtar
  * [ ] build `aarch64-unknown-linux-gnu` on tapioca debian VM
  * [ ] build `x86_64-unknown-freebsd` on talia
* [ ] `cargo publish`
* [ ] push tag
* [ ] edit release to include changelog
