// Expose the hal submodule's version as HAL_VERSION, so `hal --version`
// reports hal's version rather than this wrapper crate's.
fn main() {
    let manifest = "submodules/hal/Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest);
    let toml = std::fs::read_to_string(manifest)
        .expect("hal submodule missing; run: git submodule update --init");
    let version = toml
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .and_then(|v| v.strip_suffix('"'))
        .expect("no version in submodules/hal/Cargo.toml");
    println!("cargo:rustc-env=HAL_VERSION={}", version);
}
