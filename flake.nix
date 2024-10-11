{
  description = "Rust Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable"; # Use unstable for latest Rust versions
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
      in
      {
        devShell = pkgs.mkShell {
          name = "rust-dev-env";

          buildInputs = [
            rust
            pkgs.pkg-config
            pkgs.openssl
            pkgs.sqlite
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        defaultPackage = pkgs.rustPlatform.buildRustPackage {
          pname = "botirage";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.openssl
            pkgs.sqlite
          ];
        };
      }
    )
    // (
      let
        pkgsArm = import nixpkgs {
          system = "aarch64-linux";
          overlays = [ (import rust-overlay) ];
        };
        pkgsCross = import nixpkgs {
          system = "x86_64-linux";
          overlays = [
            (import rust-overlay)
          ];
          crossSystem = {
            config = "aarch64-unknown-linux-gnu";
          };
        };
      in
      {
        packages.cross.botirage = pkgsCross.rustPlatform.buildRustPackage {
          pname = "botirage";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [
            pkgsCross.pkg-config
            pkgsArm.openssl
            pkgsArm.sqlite
          ];
          buildInputs = [
            pkgsArm.openssl
            pkgsArm.sqlite
          ];
        };
      }
    );
}
