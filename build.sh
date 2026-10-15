#!/usr/bin/env bash
# Build hal for wasm32-wasip1 and place it in web/ next to index.html.
#
# Requires a clang with the WebAssembly backend for the secp256k1 C code.
# On Nix, use the unwrapped clang, the default wrapper injects glibc headers:
#   nix shell nixpkgs#llvmPackages.clang-unwrapped nixpkgs#llvmPackages.bintools-unwrapped
set -euo pipefail

cd "$(dirname "$0")"

if [ ! -f submodules/hal/Cargo.toml ]; then
	git submodule update --init submodules/hal
fi

if command -v rustup >/dev/null; then
	rustup target add wasm32-wasip1
fi

sysroot="$(rustc --print sysroot)"
if [ ! -d "$sysroot/lib/rustlib/wasm32-wasip1" ]; then
	cat >&2 <<EOF
error: $(rustc --version) at $sysroot has no wasm32-wasip1 standard library.
Use a rustup-managed toolchain (Rust >= 1.78) and run: rustup target add wasm32-wasip1
On Nix use:
  nix shell nixpkgs#rustup nixpkgs#llvmPackages.clang-unwrapped \\
    nixpkgs#llvmPackages.bintools-unwrapped nixpkgs#binaryen -c env RUSTUP_TOOLCHAIN=stable $0
EOF
	exit 1
fi

export CC_wasm32_wasip1="${CC_wasm32_wasip1:-clang}"
export AR_wasm32_wasip1="${AR_wasm32_wasip1:-llvm-ar}"
cargo build --release --target wasm32-wasip1

wasm="target/wasm32-wasip1/release/hal.wasm"
if command -v wasm-opt >/dev/null; then
	# Don't pass -all: it lets binaryen emit not-yet-shipped wasm proposals.
	wasm-opt -Oz --strip-debug "$wasm" -o web/hal.wasm
else
	echo "wasm-opt not found, copying unoptimized binary" >&2
	cp "$wasm" web/hal.wasm
fi
ls -lh web/hal.wasm
