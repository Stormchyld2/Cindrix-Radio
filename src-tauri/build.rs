use std::path::Path;

fn main() {
    // The private build embeds yt-dlp.exe (fetched by CI into src-tauri/ytdlp/). When the file
    // is missing (local checks, production builds) an empty placeholder is used and the app
    // falls back to downloading the helper on demand.
    let src = Path::new("ytdlp/yt-dlp.exe");
    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("yt-dlp.bin");
    if src.exists() {
        std::fs::copy(src, &out).unwrap();
    } else {
        std::fs::write(&out, []).unwrap();
    }
    println!("cargo:rerun-if-changed=ytdlp/yt-dlp.exe");
    tauri_build::build()
}
