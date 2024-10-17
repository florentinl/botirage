{
  description = "Rust Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable"; # Use unstable for latest Rust versions
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        pkgsCross = import nixpkgs {
          system = system;
          crossSystem = "aarch64-linux";
          overlays = [
            (import rust-overlay)
          ];
        };

        craneLib = (crane.mkLib pkgsCross).overrideToolchain (p: p.rust-bin.stable.latest.default);

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

        packages.botirage-aarch64 = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgsCross.pkgsBuildHost; [
            pkg-config
          ];

          buildInputs = with pkgsCross.pkgsHostHost; [
            openssl
            sqlite
          ];

          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
          CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER =
            with pkgsCross.pkgsHostHost;
            "${stdenv.cc.targetPrefix}cc";

          HOST_CC = with pkgsCross.pkgsBuildHost; "${stdenv.cc.nativePrefix}cc";
          TARGET_CC = with pkgsCross.pkgsHostHost; "${stdenv.cc.targetPrefix}cc";
        };

      }
    );
}
