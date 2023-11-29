{
  description = "comrak";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-23.05;

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    fenix,
    flake-utils,
    advisory-db,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      inherit (pkgs) lib;

      craneLib = crane.lib.${system};
      src = craneLib.cleanCargoSource (craneLib.path ./.);

      commonArgs = {
        inherit src;

        buildInputs =
          [
          ]
          ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
      };

      craneLibLLvmTools =
        craneLib.overrideToolchain
        (fenix.packages.${system}.complete.withComponents [
          "cargo"
          "llvm-tools"
          "rustc"
        ]);

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      comrak = craneLib.buildPackage (commonArgs
        // {
          inherit cargoArtifacts;

          doCheck = false;
        });
    in {
      checks =
        {
          inherit comrak;

          comrak-clippy = craneLib.cargoClippy (commonArgs
            // {
              inherit cargoArtifacts;
              # cargoClippyExtraArgs = "--lib --bins --examples --tests -- --deny warnings";
              # XXX Not sure if we can fix all these and retain our current MSRV.
              cargoClippyExtraArgs = "--lib --bins --examples --tests";
            });

          comrak-doc = craneLib.cargoDoc (commonArgs
            // {
              inherit cargoArtifacts;
            });

          comrak-fmt = craneLib.cargoFmt {
            inherit src;
          };

          comrak-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          comrak-nextest = craneLib.cargoNextest (commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            });
        }
        // lib.optionalAttrs (system == "x86_64-linux") {
          comrak-coverage = craneLib.cargoTarpaulin (commonArgs
            // {
              inherit cargoArtifacts;
            });
        };

      packages = {
        default = comrak;
        comrak-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs
          // {
            inherit cargoArtifacts;
          });
      };

      apps.default = flake-utils.lib.mkApp {
        drv = comrak;
      };

      formatter = pkgs.alejandra;

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks.${system};

        nativeBuildInputs = with pkgs; [
          cargo
          rustc
        ];
      };
    });
}
