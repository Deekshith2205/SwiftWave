// build.rs — swiftwave_ffi
//
// TODO (Phase 2): Uncomment the cbindgen block below to auto-generate
// the C header file `swiftwave_ffi.h` from the `extern "C"` functions in
// src/api.rs.  The header is consumed by Dart FFI's `ffigen` tool.
//
// Steps to enable:
// 1. Add `cbindgen = "0.27"` to [build-dependencies] in Cargo.toml.
// 2. Uncomment the block below.
// 3. Run `cargo build` — swiftwave_ffi.h will appear in OUT_DIR.
// 4. Point `ffigen` at the header to auto-generate Dart bindings.

fn main() {
    // TODO (Phase 2):
    // let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // cbindgen::Builder::new()
    //     .with_crate(crate_dir)
    //     .with_language(cbindgen::Language::C)
    //     .generate()
    //     .expect("Unable to generate C bindings")
    //     .write_to_file(format!("{out_dir}/swiftwave_ffi.h"));
    println!("cargo:rerun-if-changed=src/api.rs");
}
