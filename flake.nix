{
  description = "comrak";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;

          buildInputs = lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
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
        checks = {
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
        };

        packages = {
          default = comrak;
        };

        apps.default = flake-utils.lib.mkApp { drv = comrak; };

        formatter = pkgs.nixfmt-rfc-style;

        devShells.default = craneLib.devShell {
          name = "comrak";

          inputsFrom = builtins.attrValues self.checks.${system};

          packages = [
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.cargo-fuzz
            pkgs.python3
          ];
        };
      }
    );
}
