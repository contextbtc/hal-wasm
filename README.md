# hal-wasm

The [hal](https://github.com/stevenroose/hal) Bitcoin CLI compiled to
WebAssembly (`wasm32-wasip1`) and run in the browser through
[browser_wasi_shim](https://github.com/bjorn3/browser_wasi_shim).
Everything runs client-side; nothing is sent anywhere.

```sh
git submodule update --init      # fetches hal into submodules/hal/
./build.sh                       # produces web/hal.wasm
python3 -m http.server -d web    # then open http://localhost:8000
```

## How it works

hal is included unmodified as a git submodule in `submodules/hal/`. This crate compiles
hal's command implementations (`submodules/hal/src/bin/hal/cmd/` and `args.rs`) directly
via `#[path]` and supplies its own entry point (`src/main.rs`, `src/util.rs`).
The only difference from hal's own `main.rs` is that external `hal-*`
subcommands are left out, because they need PATH lookups and process spawning
that WASI doesn't provide.

`web/hal.js` exports `runHal(args, stdin)` returning `{ code, stdout, stderr }`,
so the module can be reused from other pages. `web/index.html` is a small UI
around it. The same binary also runs under any WASI runtime, e.g.
`wasmtime web/hal.wasm tx decode <hex>`.

Not supported: external `hal-*` subcommands and reading or writing files
(e.g. PSBT file paths); pass data as arguments or stdin instead.

## Building

Building needs a rustup-managed Rust >= 1.78 (for the `wasm32-wasip1` target)
and a clang with the WebAssembly backend (for the secp256k1 C code). On Nix:

```sh
nix shell nixpkgs#rustup nixpkgs#llvmPackages.clang-unwrapped \
	nixpkgs#llvmPackages.bintools-unwrapped nixpkgs#binaryen \
	-c env RUSTUP_TOOLCHAIN=stable ./build.sh
```

`cargo build` without `--target` gives a native `hal` built the same way, which
is handy for comparing output.

## Updating hal

1. `git -C hal fetch && git -C hal checkout <tag or commit>`
2. Align the dependency versions in `Cargo.toml` with `submodules/hal/Cargo.toml`
   (copying `submodules/hal/Cargo.lock` to `Cargo.lock` keeps resolution identical).
3. If hal added a command group, add it to the `match` in `src/main.rs`;
   if `submodules/hal/src/bin/hal/main.rs` or `util.rs` changed, mirror the change.

Mismatches show up as compile errors.
