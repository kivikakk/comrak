* [ ] `rustup update stable`
* [ ] ensure `cargo +stable build --release --all-features` works
* [ ] bump version in Cargo.toml
* [ ] update changelog
* [ ] `script/update-readme`
* [ ] commit, tag, but do not push yet
* build binaries:
  * [ ] build x86_64-apple-darwin on sencha
  * [ ] build x86_64-pc-windows-gnu on sencha vm
  * [ ] cross-compile arm7-unknown-linux-musleabihf on sencha
    * [ ] test on muffin
  * [ ] build x86_64-unknown-linux-gnu on corgi
  * [ ] build aarch64-unknown-linux-gnu on corgi aarch64 chroot
* [ ] `cargo publish`
* [ ] push commit and tag
* [ ] edit release to include changelog
