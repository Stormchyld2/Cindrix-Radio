//! Web video search and download-for-later.
//!
//! Sources that allow downloading (Internet Archive, PeerTube via Sepia Search) are always
//! built in. The YouTube source lives behind the `ytdl` cargo feature, which is on in the
//! private build and left out of production with `--no-default-features`. YouTube results can
//! always be watched online through the embed, with or without the feature.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VideoHit {
    pub id: String,
    /// "youtube" | "archive" | "peertube"
    pub source: String,
    pub title: String,
    pub channel: String,
    pub duration_secs: Option<u64>,
    pub thumbnail: Option<String>,
    pub page_url: String,
    pub embed_url: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct VideoSource { pub id: &'static str, pub label: &'static str, pub can_download: bool }

pub fn sources() -> Vec<VideoSource> {
    let mut v = Vec::new();
    #[cfg(feature = "ytdl")]
    v.push(VideoSource { id: "youtube", label: "YouTube", can_download: true });
    #[cfg(not(feature = "ytdl"))]
    v.push(VideoSource { id: "youtube", label: "YouTube (watch only)", can_download: false });
    v.push(VideoSource { id: "archive", label: "Internet Archive", can_download: true });
    v.push(VideoSource { id: "peertube", label: "PeerTube", can_download: true });
    v
}

fn client(total: Option<Duration>) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CindrixRadio/0.1")
        .connect_timeout(Duration::from_secs(20))
        .timeout(total)
        .build()
        .map_err(|e| e.to_string())
}

fn get_json(url: reqwest::Url) -> Result<Value, String> {
    let body = client(Some(Duration::from_secs(25)))?
        .get(url).send().and_then(|r| r.error_for_status()).and_then(|r| r.text())
        .map_err(|e| format!("Search unavailable: {e}"))?;
    serde_json::from_str(&body).map_err(|e| format!("Unexpected answer from the server: {e}"))
}

/// A JSON field that may be a string or a list of strings.
fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a.first().map(text).unwrap_or_default(),
        _ => String::new(),
    }
}

fn is_http(u: &str) -> bool { u.starts_with("http://") || u.starts_with("https://") }

fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() < 200 && id.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

// ---------------------------------------------------------------- search

pub fn search(data_dir: &Path, source: &str, query: &str, limit: u32, offset: u32) -> Result<Vec<VideoHit>, String> {
    let q = query.trim();
    if q.is_empty() { return Err("Type something to search for.".into()); }
    match source {
        "archive" => search_archive(q, limit, offset),
        "peertube" => search_peertube(q, limit, offset),
        "youtube" => youtube_search(data_dir, q, limit, offset),
        _ => Err("Unknown video source".into()),
    }
}

pub fn parse_archive_search(v: &Value) -> Vec<VideoHit> {
    v["response"]["docs"].as_array().map(|docs| docs.iter().filter_map(|d| {
        let id = text(&d["identifier"]);
        if !safe_id(&id) { return None; }
        let title = text(&d["title"]);
        Some(VideoHit {
            thumbnail: Some(format!("https://archive.org/services/img/{id}")),
            page_url: format!("https://archive.org/details/{id}"),
            embed_url: Some(format!("https://archive.org/embed/{id}")),
            title: if title.is_empty() { id.clone() } else { title },
            channel: text(&d["creator"]),
            duration_secs: None,
            source: "archive".into(),
            id,
        })
    }).collect()).unwrap_or_default()
}

fn search_archive(q: &str, limit: u32, offset: u32) -> Result<Vec<VideoHit>, String> {
    let mut url = reqwest::Url::parse("https://archive.org/advancedsearch.php").map_err(|e| e.to_string())?;
    {
        let mut p = url.query_pairs_mut();
        p.append_pair("q", &format!("({q}) AND mediatype:(movies)"))
            .append_pair("fl[]", "identifier").append_pair("fl[]", "title").append_pair("fl[]", "creator")
            .append_pair("sort[]", "downloads desc")
            .append_pair("rows", &limit.to_string())
            .append_pair("page", &(offset / limit.max(1) + 1).to_string())
            .append_pair("output", "json");
    }
    Ok(parse_archive_search(&get_json(url)?))
}

pub fn parse_sepia(v: &Value) -> Vec<VideoHit> {
    v["data"].as_array().map(|rows| rows.iter().filter_map(|d| {
        let id = text(&d["uuid"]);
        let page = text(&d["url"]);
        if id.is_empty() || !is_http(&page) { return None; }
        let embed = text(&d["embedUrl"]);
        let who = text(&d["channel"]["displayName"]);
        Some(VideoHit {
            title: text(&d["name"]),
            channel: if who.is_empty() { text(&d["account"]["displayName"]) } else { who },
            duration_secs: d["duration"].as_u64(),
            thumbnail: Some(text(&d["thumbnailUrl"])).filter(|t| is_http(t)),
            embed_url: Some(embed).filter(|e| is_http(e)),
            page_url: page,
            source: "peertube".into(),
            id,
        })
    }).collect()).unwrap_or_default()
}

fn search_peertube(q: &str, limit: u32, offset: u32) -> Result<Vec<VideoHit>, String> {
    let mut url = reqwest::Url::parse("https://sepiasearch.org/api/v1/search/videos").map_err(|e| e.to_string())?;
    url.query_pairs_mut().append_pair("search", q).append_pair("start", &offset.to_string()).append_pair("count", &limit.to_string());
    Ok(parse_sepia(&get_json(url)?))
}

// ---------------------------------------------------------------- picking a file

/// Choose the file to download from an Internet Archive item: the h.264 mp4 derivative if
/// there is one, else any mp4, else webm. (The player cannot open mkv or avi.)
pub fn pick_archive_file(meta: &Value) -> Option<String> {
    let files = meta["files"].as_array()?;
    let name = |f: &Value| f["name"].as_str().unwrap_or("").to_string();
    let lower = |f: &Value| name(f).to_lowercase();
    let rank = |f: &Value| -> Option<u8> {
        let (n, fmt) = (lower(f), f["format"].as_str().unwrap_or("").to_lowercase());
        if n.contains('/') || n.contains("_thumb") { return None; }
        if n.ends_with(".mp4") && (fmt.contains("h.264") || fmt.contains("mpeg4")) { Some(0) }
        else if n.ends_with(".mp4") { Some(1) }
        else if n.ends_with(".webm") { Some(2) }
        else { None }
    };
    files.iter().filter_map(|f| rank(f).map(|r| (r, name(f)))).min_by_key(|(r, _)| *r).map(|(_, n)| n)
}

/// Choose a PeerTube file: the best one up to 720p, else the smallest above it.
pub fn pick_peertube_file(v: &Value) -> Option<String> {
    let mut all: Vec<&Value> = v["files"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
    if let Some(pl) = v["streamingPlaylists"].as_array() {
        for p in pl { if let Some(f) = p["files"].as_array() { all.extend(f.iter()); } }
    }
    let cand: Vec<(u64, String)> = all.iter().filter_map(|f| {
        let url = ["fileDownloadUrl", "fileUrl"].iter().map(|k| text(&f[*k])).find(|u| is_http(u))?;
        Some((f["resolution"]["id"].as_u64().unwrap_or(0), url))
    }).collect();
    cand.iter().filter(|(r, _)| *r <= 720).max_by_key(|(r, _)| *r)
        .or_else(|| cand.iter().min_by_key(|(r, _)| *r)).map(|(_, u)| u.clone())
}

// ---------------------------------------------------------------- download

fn extension_of(url: &str) -> &'static str {
    let path = url.split(['?', '#']).next().unwrap_or("").to_lowercase();
    if path.ends_with(".webm") { "webm" } else if path.ends_with(".ogv") { "ogv" } else if path.ends_with(".m4v") { "m4v" } else { "mp4" }
}

fn channel_dir(root: &Path, channel: &str) -> PathBuf {
    let c = crate::podcasts::sanitize_name(channel);
    root.join(if channel.trim().is_empty() { "Other" } else { c.as_str() })
}

fn save_stream(url: &str, dir: &Path, title: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let name = format!("{}.{}", crate::podcasts::sanitize_name(title).chars().take(120).collect::<String>(), extension_of(url));
    let dest = dir.join(&name);
    if dest.exists() { return Ok(dest); }
    let part = dir.join(format!("{name}.part"));
    let mut resp = client(None)?.get(url).send().and_then(|r| r.error_for_status()).map_err(|e| format!("Download failed: {e}"))?;
    let expected = resp.content_length();
    let mut f = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let written = resp.copy_to(&mut f).map_err(|e| { let _ = std::fs::remove_file(&part); format!("Download interrupted: {e}") })?;
    f.flush().map_err(|e| e.to_string())?;
    if expected.map_or(false, |n| n != written) { let _ = std::fs::remove_file(&part); return Err("Download was cut short; try again".into()); }
    std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    let _ = crate::meta::append_entry(dir, &name, title, None, None);
    Ok(dest)
}

/// Download one search result into <videos root>/<channel>/. Returns the saved path.
pub fn download(data_dir: &Path, hit: &VideoHit, videos_root: &Path) -> Result<PathBuf, String> {
    let dir = channel_dir(videos_root, &hit.channel);
    match hit.source.as_str() {
        "archive" => {
            if !safe_id(&hit.id) { return Err("Bad item id".into()); }
            let meta = get_json(reqwest::Url::parse(&format!("https://archive.org/metadata/{}", hit.id)).map_err(|e| e.to_string())?)?;
            let file = pick_archive_file(&meta).ok_or("This item has no video file the player can open")?;
            let mut url = reqwest::Url::parse(&format!("https://archive.org/download/{}/", hit.id)).map_err(|e| e.to_string())?;
            url.path_segments_mut().map_err(|_| "Bad address")?.pop().push(&file);
            save_stream(url.as_str(), &dir, &hit.title)
        }
        "peertube" => {
            let page = reqwest::Url::parse(&hit.page_url).map_err(|e| e.to_string())?;
            let host = page.host_str().ok_or("Bad video address")?;
            if !safe_id(&hit.id) { return Err("Bad video id".into()); }
            let api = reqwest::Url::parse(&format!("{}://{}/api/v1/videos/{}", page.scheme(), host, hit.id)).map_err(|e| e.to_string())?;
            let file = pick_peertube_file(&get_json(api)?).ok_or("This video has no downloadable file")?;
            save_stream(&file, &dir, &hit.title)
        }
        "youtube" => youtube_download(data_dir, hit, &dir),
        _ => Err("Unknown video source".into()),
    }
}

// ---------------------------------------------------------------- YouTube (private build only)

#[cfg(feature = "ytdl")]
pub use ytdl::{install as ytdl_install, version as ytdl_version};

#[cfg(not(feature = "ytdl"))]
pub fn ytdl_install(_: &Path) -> Result<String, String> { Err("The YouTube downloader is not part of this build.".into()) }
#[cfg(not(feature = "ytdl"))]
pub fn ytdl_version(_: &Path) -> Option<String> { None }

#[cfg(feature = "ytdl")]
fn youtube_search(data_dir: &Path, q: &str, limit: u32, offset: u32) -> Result<Vec<VideoHit>, String> {
    ytdl::search(data_dir, q, limit, offset)
}
#[cfg(not(feature = "ytdl"))]
fn youtube_search(_: &Path, _: &str, _: u32, _: u32) -> Result<Vec<VideoHit>, String> {
    Err("YouTube search is not part of this build.".into())
}
#[cfg(feature = "ytdl")]
fn youtube_download(data_dir: &Path, hit: &VideoHit, dir: &Path) -> Result<PathBuf, String> { ytdl::download(data_dir, hit, dir) }
#[cfg(not(feature = "ytdl"))]
fn youtube_download(_: &Path, _: &VideoHit, _: &Path) -> Result<PathBuf, String> {
    Err("Downloading from YouTube is not part of this build. You can still watch it online.".into())
}

#[cfg(feature = "ytdl")]
pub mod ytdl {
    use super::*;
    use std::process::Command;

    fn exe_name() -> &'static str { if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" } }

    fn run(bin: &Path, args: &[&str]) -> Result<String, String> {
        let mut c = Command::new(bin);
        c.args(args);
        #[cfg(windows)]
        { use std::os::windows::process::CommandExt; c.creation_flags(0x0800_0000); } // no console window
        let out = c.output().map_err(|e| format!("Could not run yt-dlp: {e}"))?;
        if out.status.success() { Ok(String::from_utf8_lossy(&out.stdout).into_owned()) }
        else {
            let err = String::from_utf8_lossy(&out.stderr);
            Err(err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("yt-dlp failed").trim().to_string())
        }
    }

    /// The yt-dlp we manage in the app data folder, else one on the PATH.
    pub fn binary(data_dir: &Path) -> Option<PathBuf> {
        let own = data_dir.join("bin").join(exe_name());
        if own.exists() { return Some(own); }
        let on_path = PathBuf::from(exe_name());
        run(&on_path, &["--version"]).ok().map(|_| on_path)
    }

    pub fn version(data_dir: &Path) -> Option<String> {
        let b = binary(data_dir)?;
        run(&b, &["--version"]).ok().map(|v| v.trim().to_string())
    }

    /// Download (or update to) the latest yt-dlp release into the app data folder.
    pub fn install(data_dir: &Path) -> Result<String, String> {
        let asset = if cfg!(windows) { "yt-dlp.exe" } else if cfg!(target_os = "macos") { "yt-dlp_macos" } else { "yt-dlp_linux" };
        let dir = data_dir.join("bin");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let url = format!("https://github.com/yt-dlp/yt-dlp/releases/latest/download/{asset}");
        let bytes = client(Some(Duration::from_secs(300)))?.get(url).send().and_then(|r| r.error_for_status()).and_then(|r| r.bytes())
            .map_err(|e| format!("Could not download yt-dlp: {e}"))?;
        if bytes.len() < 1_000_000 { return Err("The yt-dlp download looks incomplete; try again".into()); }
        let part = dir.join(format!("{}.part", exe_name()));
        std::fs::write(&part, &bytes).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        { use std::os::unix::fs::PermissionsExt; let _ = std::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o755)); }
        let dest = dir.join(exe_name());
        let _ = std::fs::remove_file(&dest);
        std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
        run(&dest, &["--version"]).map(|v| v.trim().to_string())
    }

    pub fn parse_search(v: &Value) -> Vec<VideoHit> {
        let one = |d: &Value| -> Option<VideoHit> {
            let id = text(&d["id"]);
            if !safe_id(&id) { return None; }
            let who = text(&d["channel"]);
            Some(VideoHit {
                title: text(&d["title"]),
                channel: if who.is_empty() { text(&d["uploader"]) } else { who },
                duration_secs: d["duration"].as_f64().map(|s| s as u64),
                thumbnail: Some(format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg")),
                page_url: format!("https://www.youtube.com/watch?v={id}"),
                embed_url: Some(format!("https://www.youtube.com/embed/{id}")),
                source: "youtube".into(),
                id,
            })
        };
        match v["entries"].as_array() {
            Some(list) => list.iter().filter_map(one).collect(),
            None => one(v).into_iter().collect(),
        }
    }

    /// A search phrase, or a pasted video link.
    pub fn search(data_dir: &Path, q: &str, limit: u32, offset: u32) -> Result<Vec<VideoHit>, String> {
        let bin = binary(data_dir).ok_or("The YouTube helper (yt-dlp) is not installed yet. Install it from the Videos tab.")?;
        let out = if q.starts_with("http://") || q.starts_with("https://") {
            run(&bin, &["--dump-single-json", "--flat-playlist", "--no-playlist", "--no-warnings", "--", q])?
        } else {
            let n = (limit + offset).to_string();
            let term = format!("ytsearch{n}:{q}");
            run(&bin, &["--dump-single-json", "--flat-playlist", "--no-warnings", "--", &term])?
        };
        let v: Value = serde_json::from_str(&out).map_err(|e| format!("Unexpected answer from yt-dlp: {e}"))?;
        Ok(parse_search(&v).into_iter().skip(offset as usize).collect())
    }

    pub fn download(data_dir: &Path, hit: &VideoHit, dir: &Path) -> Result<PathBuf, String> {
        if !safe_id(&hit.id) { return Err("Bad video id".into()); }
        let bin = binary(data_dir).ok_or("The YouTube helper (yt-dlp) is not installed yet. Install it from the Videos tab.")?;
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let url = format!("https://www.youtube.com/watch?v={}", hit.id);
        let out = run(&bin, &[
            "-f", "b[ext=mp4]/b", "--no-playlist", "--no-warnings", "--windows-filenames", "--no-mtime", "-q",
            "--no-simulate", "--print", "after_move:filepath",
            "-P", &dir.to_string_lossy(), "-o", "%(title).150B.%(ext)s", "--", &url,
        ])?;
        let path = PathBuf::from(out.lines().rev().find(|l| !l.trim().is_empty()).ok_or("yt-dlp did not report a file")?.trim());
        if !path.exists() { return Err("yt-dlp finished but the file was not found".into()); }
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let _ = crate::meta::append_entry(dir, name, &hit.title, None, None);
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn archive_results_are_read_and_unsafe_ids_dropped() {
        let v = json!({"response":{"docs":[
            {"identifier":"abc_123","title":["Old Film"],"creator":"Someone"},
            {"identifier":"../evil","title":"x"},
            {"identifier":"nocreator"}]}});
        let hits = parse_archive_search(&v);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].title, "Old Film");
        assert_eq!(hits[0].channel, "Someone");
        assert_eq!(hits[0].page_url, "https://archive.org/details/abc_123");
        assert_eq!(hits[1].title, "nocreator", "falls back to the id");
    }

    #[test]
    fn archive_file_prefers_h264_mp4() {
        let v = json!({"files":[
            {"name":"movie.ogv","format":"Ogg Video"},
            {"name":"movie_thumb.mp4","format":"Thumbnail"},
            {"name":"movie.mkv","format":"Matroska"},
            {"name":"orig.mp4","format":"Original"},
            {"name":"movie.mp4","format":"h.264"}]});
        assert_eq!(pick_archive_file(&v).as_deref(), Some("movie.mp4"));
        assert_eq!(pick_archive_file(&json!({"files":[{"name":"a.mkv","format":"x"}]})), None);
        assert_eq!(pick_archive_file(&json!({"files":[{"name":"a.webm","format":"x"}]})).as_deref(), Some("a.webm"));
    }

    #[test]
    fn peertube_picks_best_up_to_720() {
        let v = json!({"files":[
            {"resolution":{"id":1080},"fileDownloadUrl":"https://h/1080.mp4"},
            {"resolution":{"id":720},"fileDownloadUrl":"https://h/720.mp4"},
            {"resolution":{"id":360},"fileUrl":"https://h/360.mp4"}]});
        assert_eq!(pick_peertube_file(&v).as_deref(), Some("https://h/720.mp4"));
        let big = json!({"files":[{"resolution":{"id":2160},"fileUrl":"https://h/4k.mp4"},{"resolution":{"id":1080},"fileUrl":"https://h/hd.mp4"}]});
        assert_eq!(pick_peertube_file(&big).as_deref(), Some("https://h/hd.mp4"), "smallest when all are above 720p");
        assert_eq!(pick_peertube_file(&json!({"files":[]})), None);
    }

    #[test]
    fn sepia_results_are_read() {
        let v = json!({"data":[{"uuid":"u1","name":"Vid","duration":61,"thumbnailUrl":"https://h/t.jpg","url":"https://h/w/u1","embedUrl":"https://h/e/u1","channel":{"displayName":"Chan"},"account":{"displayName":"Acc"}},
            {"uuid":"u2","name":"No url"}]});
        let hits = parse_sepia(&v);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].channel, "Chan");
        assert_eq!(hits[0].duration_secs, Some(61));
    }

    #[test]
    fn file_extension_and_folder_names_are_safe() {
        assert_eq!(extension_of("https://h/a.WEBM?x=1"), "webm");
        assert_eq!(extension_of("https://h/a"), "mp4");
        let d = channel_dir(Path::new("/v"), "A/B: C");
        assert_eq!(d, Path::new("/v").join("A_B_ C"));
        assert_eq!(channel_dir(Path::new("/v"), "  "), Path::new("/v").join("Other"));
    }

    #[test]
    fn source_list_always_has_the_open_sources() {
        let ids: Vec<_> = sources().iter().map(|s| s.id).collect();
        assert!(ids.contains(&"archive") && ids.contains(&"peertube") && ids.contains(&"youtube"));
    }

    #[cfg(feature = "ytdl")]
    #[test]
    fn youtube_results_and_single_links_are_read() {
        let list = json!({"entries":[{"id":"dQw4w9WgXcQ","title":"T","channel":"C","duration":212.0},{"id":"bad id!"}]});
        let hits = ytdl::parse_search(&list);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].duration_secs, Some(212));
        assert_eq!(hits[0].embed_url.as_deref(), Some("https://www.youtube.com/embed/dQw4w9WgXcQ"));
        let single = json!({"id":"abc","title":"One","uploader":"U"});
        assert_eq!(ytdl::parse_search(&single)[0].channel, "U");
    }
}
