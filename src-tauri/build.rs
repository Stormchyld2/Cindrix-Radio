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
    // `ytdl_on`: the yt-dlp helper is compiled in (private build). On Android it is the bundled
    // open-source yt-dlp library, reached through crate::android.
    println!("cargo:rustc-check-cfg=cfg(ytdl_on)");
    if std::env::var("CARGO_FEATURE_YTDL").is_ok() {
        println!("cargo:rustc-cfg=ytdl_on");
    }
    tauri_build::build()
}
