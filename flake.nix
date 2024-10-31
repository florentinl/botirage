{
  description = "Rust Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable"; # Use unstable for latest Rust versions
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    (
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

          craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);
          craneLibCross = (crane.mkLib pkgsCross).overrideToolchain (p: p.rust-bin.stable.latest.default);

          rustDev = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };

          mkNixosModuleFromPkg =
            {
              pkg,
            }:
            {
              config,
              lib,
              ...
            }:
            {
              options.services.botirage = {
                enable = lib.mkOption {
                  type = lib.types.bool;
                  default = false;
                  description = "Enable the Botirage service";
                };

                telegram_api_key = lib.mkOption {
                  type = lib.types.str;
                  default = "";
                  description = "Telegram API key for the bot";
                };

                database_path = lib.mkOption {
                  type = lib.types.str;
                  default = "/etc/botirage/botirage.db";
                  description = "Path to the database file";
                };
              };

              config = lib.mkIf config.services.botirage.enable {
                systemd.services.botirage = {
                  enable = true;
                  description = "CybersecDealerBot";
                  wantedBy = [ "multi-user.target" ];
                  environment = {
                    TELOXIDE_TOKEN = config.services.botirage.telegram_api_key;
                    DATABASE_PATH = config.services.botirage.database_path;
                  };
                  serviceConfig = {
                    Type = "simple";
                    ExecStart = lib.getExe pkg;
                    Restart = "always";
                  };
                };
              };
            };
        in
        {
          devShell = pkgs.mkShell {
            name = "rust-dev-env";

            nativeBuildInputs = [
              pkgs.pkg-config
            ];

            buildInputs = [
              rustDev
              pkgs.sqlite
            ];
          };

          defaultPackage = craneLib.buildPackage {
            src = craneLibCross.cleanCargoSource ./.;
            strictDeps = true;

            nativeBuildInputs = with pkgs.pkgsBuildHost; [
              pkg-config
            ];

            buildInputs = with pkgs.pkgsHostHost; [
              sqlite
            ];

            meta = {
              mainProgram = "botirage";
            };
          };

          packages.botirage-cross-aarch64 = craneLibCross.buildPackage {
            src = craneLibCross.cleanCargoSource ./.;
            strictDeps = true;

            nativeBuildInputs = with pkgsCross.pkgsBuildHost; [
              pkg-config
            ];

            buildInputs = with pkgsCross.pkgsHostHost; [
              sqlite
            ];

            CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER = "qemu-aarch64";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER =
              with pkgsCross.pkgsHostHost;
              "${stdenv.cc.targetPrefix}cc";

            HOST_CC = with pkgsCross.pkgsBuildHost; "${stdenv.cc.nativePrefix}cc";
            TARGET_CC = with pkgsCross.pkgsHostHost; "${stdenv.cc.targetPrefix}cc";

            meta = {
              mainProgram = "botirage";
            };
          };

          nixosModule = mkNixosModuleFromPkg { pkg = self.defaultPackage.${system}; };
          nixosModuleCrossAarch64 = mkNixosModuleFromPkg {
            pkg = self.packages.${system}.botirage-cross-aarch64;
          };
        }
      )
      // {
        hydraJobs = {
          build = self.defaultPackage.aarch64-linux;
        };

        checks = {
          botirage-aarch64 = self.defaultPackage.aarch64-linux;
        };
      }
    );
}
