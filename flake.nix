{
  description = "A very basic flake";

  inputs = {
      nixpkgs.url = "github:NixOS/nixpkgs";
      flake-utils.url = "github:numtide/flake-utils";

      riscv-tests.url = "github:coastalwhite/riscv-tests-nixflake";
  };

  outputs = { self, nixpkgs, flake-utils, riscv-tests }:
    flake-utils.lib.eachDefaultSystem (system: let
        pkgs = import nixpkgs { inherit system; };
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
                ];

				shellHook = ''
                    export RISCV_TESTS="${riscv-tests.packages.${system}.default}"
                    export RISCV="${riscv-toolchain.buildPackages.gcc}"
					rustup target add riscv32i-unknown-none-elf
					rustup target add riscv32imac-unknown-none-elf
				'';
			};
        });
}
