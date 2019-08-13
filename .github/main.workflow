workflow "Run CI" {
  on = "push"
  resolves = ["rust"]
}

action "rust" {
  uses = "icepuma/rust-action@master"
}
