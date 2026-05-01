//! build.rs — Locate the pre-built C++ pdj_audio library and tell Cargo to
//! link against it.
//!
//! The C++ engine is built via CMake from `native/audio/`.  This script
//! assumes the build directory is at `<repo>/native/audio/build` (the
//! default for `make test-cpp`).  Override with the `PDJ_AUDIO_LIB_DIR`
//! environment variable.

use std::env;
use std::path::PathBuf;

fn main() {
    // Re-run if the env var changes.
    println!("cargo:rerun-if-env-changed=PDJ_AUDIO_LIB_DIR");

    // Resolve the path to the C++ build directory.
    // Default: <repo-root>/native/audio/build
    let lib_dir = env::var_os("PDJ_AUDIO_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            // crates/pdj-engine-bridge → ../.. → repo root
            manifest
                .join("..")
                .join("..")
                .join("native")
                .join("audio")
                .join("build")
        });

    // Tell Cargo where to find the library and what to link.
    if lib_dir.exists() {
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    } else {
        // Don't fail the build if the library isn't built yet — many tasks
        // (cargo check, doc generation, IDE indexing) work without it.
        // Real linking happens when the desktop app is built.
        println!(
            "cargo:warning=pdj_audio library directory not found at {}. \
             Build it via `make test-cpp` or set PDJ_AUDIO_LIB_DIR.",
            lib_dir.display()
        );
    }

    // We dynamically link the shared library.  This avoids pulling in
    // PortAudio/libsndfile transitively as static deps.
    println!("cargo:rustc-link-lib=dylib=pdj_audio");

    // Make the dynamic linker find the library at runtime.
    if let Some(dir) = lib_dir.to_str() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{dir}");
    }
}
