// Tauri's build script generates the context that embeds tauri.conf.json
// and other assets into the binary.  It must be present verbatim.
fn main() {
    tauri_build::build()
}
