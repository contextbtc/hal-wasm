{
	inputs = {
		# Include submodules/hal in the flake source.
		self.submodules = true;
		nixpkgs.url = "nixpkgs/nixos-25.05";
		flake-utils.url = "github:numtide/flake-utils";
		fenix = {
			url = "github:nix-community/fenix";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, flake-utils, fenix }:
		flake-utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs {
					inherit system;
				};

				fenixPkgs = fenix.packages.${system};
				rustToolchain = fenixPkgs.combine [
					fenixPkgs.stable.cargo
					fenixPkgs.stable.rustc
					fenixPkgs.stable.rust-src
					fenixPkgs.stable.clippy
					fenixPkgs.stable.rustfmt
					fenixPkgs.targets.wasm32-wasip1.stable.rust-std
				];
				rustPlatform = pkgs.makeRustPlatform {
					cargo = rustToolchain;
					rustc = rustToolchain;
				};

				hal-wasm = pkgs.callPackage ./nix/package.nix { inherit rustPlatform; };
			in
			{
				packages = {
					default = hal-wasm;
					inherit hal-wasm;
				};

				devShells.default = pkgs.mkShell {
					packages = [
						rustToolchain
						fenixPkgs.stable.rust-analyzer
						pkgs.git
						pkgs.llvmPackages.clang-unwrapped
						pkgs.llvmPackages.bintools-unwrapped
						pkgs.binaryen
					];

					CC_wasm32_wasip1 = "${pkgs.llvmPackages.clang-unwrapped}/bin/clang";
					AR_wasm32_wasip1 = "${pkgs.llvmPackages.bintools-unwrapped}/bin/llvm-ar";
					inherit (hal-wasm) CFLAGS_wasm32_wasip1;
				};
			}
		);
}
