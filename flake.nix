{
  description = "comrak";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;

      mkComrak =
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          craneLib = crane.mkLib pkgs;
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          commonArgs = {
            inherit src;

            buildInputs = nixpkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          comrak = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;

              doCheck = false;
            }
          );
        in
        {
          inherit
            craneLib
            src
            commonArgs
            cargoArtifacts
            comrak
            ;
        };

    in
    {

      checks = eachSystem (
        system:
        let
          inherit (mkComrak system)
            craneLib
            src
            commonArgs
            cargoArtifacts
            comrak
            ;
        in
        {
          inherit comrak;

          comrak-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              # cargoClippyExtraArgs = "--lib --bins --examples --tests -- --deny warnings";
              # XXX Not sure if we can fix all these and retain our current MSRV.
              cargoClippyExtraArgs = "--lib --bins --examples --tests";
            }
          );

          comrak-doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          comrak-fmt = craneLib.cargoFmt { inherit src; };

          comrak-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        }
      );

      packages = eachSystem (system: rec {
        default = comrak;

        inherit (mkComrak system) comrak;
      });

      formatter = eachSystem (system: nixpkgs.legacyPackages.${system}.nixfmt-rfc-style);

      devShells = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          fenixPkgs = fenix.packages.${system};

          mkShell =
            { name, toolchain }:
            pkgs.mkShell {
              inherit name;

              packages = [
                (fenixPkgs.combine (
                  with toolchain;
                  [
                    cargo
                    rustc
                    rust-analyzer
                    clippy
                    rustfmt
                    rust-src
                    llvm-tools-preview
                  ]
                  ++ [
                    fenixPkgs.targets.wasm32-unknown-unknown.latest.rust-std
                  ]
                ))
              ]
              ++ (with pkgs; [
                rust-analyzer
                clippy
                cargo-fuzz
                cargo-nextest
                cargo-flamegraph
                samply
                python3
                re2c
                hyperfine
                bacon
              ]);
            };
        in
        {
          default = mkShell {
            name = "comrak";
            toolchain = fenixPkgs.complete;
          };

          msrv = mkShell {
            name = "comrak-msrv";
            toolchain = fenixPkgs.toolchainOf {
              channel = "1.65.0";
              sha256 = "sha256-DzNEaW724O8/B8844tt5AVHmSjSQ3cmzlU4BP90oRlY=";

            };
          };
        }
      );
    };
}
