{
  description = "comrak";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      fenix,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      eachSystem = nixpkgs.lib.genAttrs systems;
    in
    {

      packages = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          cargoToml = pkgs.lib.importTOML ./Cargo.toml;

          mkComrak =
            pkgs:
            pkgs.rustPlatform.buildRustPackage {
              pname = "comrak";
              inherit (cargoToml.package) version;

              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;

              doCheck = false;
            };
        in
        rec {
          default = comrak;

          comrak = mkComrak pkgs;
          comrak-musl = mkComrak pkgs.pkgsStatic;
        }
      );

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
              channel = "1.70.0";
              sha256 = "sha256-gdYqng0y9iHYzYPAdkC/ka3DRny3La/S5G8ASj0Ayyc=";
            };
          };
        }
      );
    };
}
