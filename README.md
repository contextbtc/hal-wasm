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
so the module can be reused from other pages (types in `web/hal.d.ts`). It
imports `@bjorn3/browser_wasi_shim` by its bare name and loads `hal.wasm` next
to itself (`new URL("./hal.wasm", import.meta.url)`), so bundlers like Vite
resolve both; `web/index.html` is a small UI around it that maps the shim to a
CDN with an import map. The same binary also runs under any WASI runtime, e.g.
`wasmtime web/hal.wasm tx decode <hex>`.

Not supported: external `hal-*` subcommands and reading or writing files
(e.g. PSBT file paths); pass data as arguments or stdin instead.

## Building

With Nix:

```sh
nix build                        # result/{hal.wasm,hal.js,hal.d.ts}
nix develop -c ./build.sh        # or: dev shell + build.sh, produces web/hal.wasm
```

Without the flake, building needs a rustup-managed Rust >= 1.78 (for the `wasm32-wasip1` target)
and a clang with the WebAssembly backend (for the secp256k1 C code). For example:

```sh
nix shell nixpkgs#rustup nixpkgs#llvmPackages.clang-unwrapped \
	nixpkgs#llvmPackages.bintools-unwrapped nixpkgs#binaryen \
	-c env RUSTUP_TOOLCHAIN=stable ./build.sh
```

`cargo build` without `--target` gives a native `hal` built the same way, which
is handy for comparing output.

## Updating hal

1. `git -C submodules/hal fetch && git -C submodules/hal checkout <tag or commit>`
2. Align the dependency versions in `Cargo.toml` with `submodules/hal/Cargo.toml`
   (copying `submodules/hal/Cargo.lock` to `Cargo.lock` keeps resolution identical).
3. If hal added a command group, add it to the `match` in `src/main.rs`;
   if `submodules/hal/src/bin/hal/main.rs` or `util.rs` changed, mirror the change.

Mismatches show up as compile errors.

## Using from another project

Add this repository as a git submodule and build it with its Nix package,
passing a `rustPlatform` whose toolchain includes the `wasm32-wasip1` std:

```sh
git submodule add https://github.com/contextbtc/hal-wasm hal-wasm
git submodule update --init --recursive
```

```nix
# flake.nix of the consuming project
inputs.self.submodules = true;  # include hal-wasm and its nested hal submodule
...
halWasm = pkgs.callPackage ./hal-wasm/nix/package.nix {
  rustPlatform = pkgs.makeRustPlatform { cargo = rustToolchain; rustc = rustToolchain; };
};
```

Copy `${halWasm}/*` next to your sources and `import { runHal } from "./hal.js"`,
with `@bjorn3/browser_wasi_shim` installed as a dependency.
