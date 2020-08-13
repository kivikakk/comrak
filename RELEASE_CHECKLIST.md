* [ ] `rustup update stable`
* [ ] ensure `cargo +stable build --release --all-features` works
* [ ] bump version in Cargo.toml
* [ ] update changelog
* [ ] `script/update-readme`
* [ ] commit, tag, but do not push yet
* build binaries:
  * [ ] build x86_64-apple-darwin on sencha
  * [ ] cross-compile x86_64-pc-windows-gnu.exe on sencha
    * [ ] ensure this actually runs and we don't need to mess with the toolchain again
  * [ ] cross-compile arm7-unknown-linux-musleabihf on sencha
    * [ ] test on muffin
  * [ ] build x86_64-unknown-linux-gnu on hannah
  * [ ] build aarch64-unknown-linux-gnu on aarch64 debian VM
* [ ] `cargo publish`
* [ ] push commit and tag
* [ ] edit release to include changelog
