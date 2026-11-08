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
    // `ytdl_on`: the yt-dlp helper is compiled in (feature on, and not an Android build, where
    // the open-source Android library will be used instead).
    println!("cargo:rustc-check-cfg=cfg(ytdl_on)");
    let android = std::env::var("CARGO_CFG_TARGET_OS").map(|o| o == "android").unwrap_or(false);
    if std::env::var("CARGO_FEATURE_YTDL").is_ok() && !android {
        println!("cargo:rustc-cfg=ytdl_on");
    }
    tauri_build::build()
}
