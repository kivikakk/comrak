* [ ] `rustup update stable`
* [ ] ensure `cargo +stable build --release --all-features` works
* [ ] bump version in Cargo.toml
* [ ] update changelog
* [ ] `script/update-readme`
* [ ] commit, tag, push commit, but do not push tag yet
* build binaries:
  * [ ] build `aarch64-apple-darwin` on tapioca
  * [ ] cross-compile `x86_64-apple-darwin` on tapioca
  * [ ] ~~build `x86_64-pc-windows-gnu` on sencha vm~~
  * [ ] ~~build `x86_64-pc-windows-msvc` on sencha vm~~
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
* [ ] `cargo publish`
* [ ] push tag
* [ ] edit release to include changelog
