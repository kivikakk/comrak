* Bump version in `Cargo.toml`.
  * Did `tests::exercise_full_api` change? if so, it's a semver-breaking change.
* Run https://github.com/kivikakk/comrak/actions/workflows/release.yml.
* Inspect the created PR, make any changes, and merge when ready.
  * This will automatically create a new git tag, GitHub release, and publish
    to crates.io.
