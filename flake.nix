{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    riscv-tests.url = "github:coastalwhite/riscv-tests-nixflake";
  };

  outputs = { self, nixpkgs, utils, rust-overlay, riscv-tests }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = [
          "riscv32i-unknown-none-elf"
          "riscv32imac-unknown-none-elf"
          "x86_64-unknown-linux-gnu"
        ];
      };

      riscv-toolchain = import nixpkgs {            
        localSystem = system;
        crossSystem = {
            config = "riscv32-none-elf";
        };
      };
      in rec {
        devShells.default = devShells.riscv32-binutils;
        devShells.riscv32-binutils = pkgs.mkShell {
          packages = [
            riscv-toolchain.buildPackages.gcc
            rustToolchain
          ];

			    shellHook = ''
            export RISCV_TESTS="${riscv-tests.packages.${system}.default}"
            export RISCV="${riscv-toolchain.buildPackages.gcc}"
			    '';
			  };
        devShells.fuzz = pkgs.mkShell {
          packages = with pkgs; [
            rust-bin.nightly.latest.default
            cargo-fuzz
          ];
			  };
      }
    );
}
