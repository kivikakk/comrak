* [ ] `rustup update stable`
* [ ] ensure `cargo +stable build --release --all-features` works
* [ ] bump version in Cargo.toml
* [ ] update changelog
* [ ] `script/update-readme`
* [ ] commit, tag, push commit, but do not push tag yet
* build binaries:
  * [ ] build `x86_64-apple-darwin` on sencha
  * [ ] build `x86_64-pc-windows-gnu` on sencha vm
  * [ ] build `x86_64-pc-windows-msvc` on sencha vm
  * [ ] cross-compile `arm7-unknown-linux-musleabihf` on sencha
    * [ ] test on muffin
  * [ ] build `x86_64-unknown-linux-gnu` on corgi/raven
  * [ ] build `aarch64-unknown-linux-gnu` on corgi aarch64 chroot (in /root)
    * cross-compile doesn't work great; produces a binary but it won't run
* [ ] `cargo publish`
* [ ] push tag
* [ ] edit release to include changelog
