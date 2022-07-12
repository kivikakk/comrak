* [ ] `rustup update stable`
* [ ] bump version in Cargo.toml
  * [ ] did `tests::exercise_full_api` change? if so, it's a semver-breaking change.
* [ ] update changelog
* [ ] `cargo run --example update-readme`
* [ ] commit, tag, push commit, but do not push tag yet
* build binaries:
  * [ ] `script/release-skye`
  * [ ] `script/release-skye-debian` in Parallels
  * [ ] `script\release-raven.ps1` ("Windows PowerShell" works, make sure to run with comrak root as cwd)
  * [ ] `script/release-raven-freebsd` in VMWare
  * [ ] `script/release-sarah`
* [ ] `script/assemble-releases`
* [ ] `cargo publish`
* [ ] push tag
* [ ] edit release to include changelog
