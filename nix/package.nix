# Builds hal for wasm32-wasip1 and outputs hal.wasm, hal.js and hal.d.ts.
#
# Reusable from other flakes (e.g. when hal-wasm is a git submodule):
#   pkgs.callPackage ./hal-wasm/nix/package.nix { rustPlatform = ...; }
# `rustPlatform` must come from a toolchain that includes the wasm32-wasip1 std.
{ lib
, rustPlatform
, llvmPackages
, binaryen
}:

rustPlatform.buildRustPackage {
	pname = "hal-wasm";
	version = "0.1.0";

	src = lib.fileset.toSource {
		root = ../.;
		fileset = lib.fileset.unions [
			../Cargo.toml
			../Cargo.lock
			../build.rs
			../src
			../web/hal.js
			../web/hal.d.ts
			../submodules/hal
		];
	};

	cargoLock.lockFile = ../Cargo.lock;

	# The default Nix CC wrapper targets the host; the secp256k1 C code needs a
	# clang with the WebAssembly backend and no glibc headers injected.
	CC_wasm32_wasip1 = "${llvmPackages.clang-unwrapped}/bin/clang";
	AR_wasm32_wasip1 = "${llvmPackages.bintools-unwrapped}/bin/llvm-ar";
	# Unwrapped clang can't locate its builtin headers (stddef.h) in the sandbox.
	CFLAGS_wasm32_wasip1 = "-resource-dir ${llvmPackages.clang-unwrapped.lib}/lib/clang/${lib.versions.major llvmPackages.clang-unwrapped.version}";

	nativeBuildInputs = [ binaryen ];

	# cargoBuildHook always passes --target for the host platform, so build the
	# wasm target ourselves (cargoSetupHook has already vendored the deps).
	buildPhase = ''
		runHook preBuild
		cargo build -j $NIX_BUILD_CORES --offline --release --target wasm32-wasip1
		runHook postBuild
	'';

	doCheck = false;

	installPhase = ''
		runHook preInstall
		mkdir -p $out
		# Don't pass -all: it lets binaryen emit not-yet-shipped wasm proposals.
		wasm-opt -Oz --strip-debug target/wasm32-wasip1/release/hal.wasm -o $out/hal.wasm
		cp web/hal.js web/hal.d.ts $out/
		runHook postInstall
	'';

	meta = {
		description = "The hal Bitcoin CLI built for WebAssembly (WASI) to run in the browser";
		license = lib.licenses.cc0;
	};
}
