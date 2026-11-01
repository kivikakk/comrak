{
  lib,
  rustPlatform,
  runCommand,
  comrak,
}:
let
  generator = rustPlatform.buildRustPackage {
    pname = "comrak-www";
    inherit (comrak) version src;
    cargoLock.lockFile = ../Cargo.lock;
    cargoBuildFlags = [
      "-p"
      "comrak-www"
    ];
    doCheck = false;
    meta.mainProgram = "comrak-www";
  };

  docs = lib.fileset.toSource {
    root = ../docs;
    fileset = ../docs;
  };

  www = lib.fileset.toSource {
    root = ../www;
    fileset = ../www;
  };
in
runCommand "comrak.ee" { } ''
  ${lib.getExe generator} ${docs} ${www} $out
''
