{
  lib,
  rustPlatform,
  runCommand,
  comrak,
  katex,
  catppuccin,
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

  theme = variant: name: "${catppuccin.override { inherit variant; }}/bat/Catppuccin ${name}.tmTheme";
in
runCommand "comrak.ee" { } ''
  ${lib.getExe generator} ${docs} ${www} \
    "${theme "latte" "Latte"}" \
    "${theme "mocha" "Mocha"}" \
    $out
  mkdir $out/katex
  cp -r ${katex}/lib/node_modules/katex/dist/{katex.min.css,katex.min.js,fonts} $out/katex/
''
