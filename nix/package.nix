{
  lib,
  rustPlatform,
}:
let
  cargoToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = "comrak";
  inherit (cargoToml.package) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../build.rs
      ../src
      ../benches
      ../examples
      ../www/Cargo.toml
      ../www/src
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  doCheck = false;
}
