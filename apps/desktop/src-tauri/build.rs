// Tauri's build script generates the context that embeds tauri.conf.json
// and other assets into the binary.  It must be present verbatim.
use std::env;
use std::path::PathBuf;

fn main() {
    // Resolve the path to the C++ build directory.
    let lib_dir = env::var_os("PDJ_AUDIO_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            // apps/desktop/src-tauri → ../../.. → repo root
            manifest
                .join("..")
                .join("..")
                .join("..")
                .join("native")
                .join("audio")
                .join("build")
        });

    if lib_dir.exists() {
        if let Some(dir) = lib_dir.to_str() {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir);
        }
        
        // Copy the library to a local resources folder for bundling.
        let lib_name = "libpdj_audio.dylib";
        let src_path = lib_dir.join(lib_name);
        let dest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("resources");
        let dest_path = dest_dir.join(lib_name);
        
        if src_path.exists() {
            std::fs::create_dir_all(&dest_dir).unwrap();
            std::fs::copy(&src_path, &dest_path).unwrap();
            println!("cargo:rerun-if-changed={}", src_path.display());
        }
    }

    // Also add RPATH for bundled version (macOS).
    #[cfg(target_os = "macos")]
    {
        // Tauri bundles external binaries in Contents/Resources/resources/ by default
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Resources");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Resources/resources");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
    }

    tauri_build::build()
}
