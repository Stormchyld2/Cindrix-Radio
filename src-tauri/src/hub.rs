//! Connect to a Cindrix Ops Hub or Vault on the same network (usually its Wi-Fi hotspot) and use the
//! music on its drive (the 4 TB drive or a thumb drive). Uses only the Hub's existing share server:
//! sign in with its PIN, ask it for the music list, copy songs over. Nothing leaves your network.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
struct Conn { addr: String, cookie: String }

static CONN: Mutex<Option<Conn>> = Mutex::new(None);
static FILE: Mutex<Option<PathBuf>> = Mutex::new(None);
static PROGRESS: Mutex<(usize, usize)> = Mutex::new((0, 0));

/// Remember where the connection is kept, and load a saved one.
pub fn init(path: PathBuf) {
    if let Some(c) = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str::<Conn>(&t).ok()) {
        *CONN.lock().unwrap() = Some(c);
    }
    *FILE.lock().unwrap() = Some(path);
}

fn save() {
    let (Some(f), c) = (FILE.lock().unwrap().clone(), CONN.lock().unwrap().clone()) else { return };
    match c {
        Some(c) => { let _ = std::fs::write(&f, serde_json::to_string(&c).unwrap_or_default()); }
        None => { let _ = std::fs::remove_file(&f); }
    }
}

pub fn status() -> Option<String> { CONN.lock().unwrap().as_ref().map(|c| c.addr.clone()) }
pub fn progress() -> (usize, usize) { *PROGRESS.lock().unwrap() }
pub fn disconnect() { *CONN.lock().unwrap() = None; save(); }

/// "192.168.43.1" -> "http://192.168.43.1:8765"
pub fn normalize(addr: &str) -> String {
    let mut a = addr.trim().trim_end_matches('/').to_string();
    if !a.to_ascii_lowercase().starts_with("http://") && !a.to_ascii_lowercase().starts_with("https://") { a = format!("http://{a}"); }
    let host = a.splitn(2, "://").nth(1).unwrap_or("");
    if !host.contains(':') { a.push_str(":8765"); }
    a
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CindrixRadio/0.1")
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(std::time::Duration::from_secs(8))
        .timeout(std::time::Duration::from_secs(120))
        .build().map_err(|e| e.to_string())
}

fn conn() -> Result<Conn, String> {
    CONN.lock().unwrap().clone().ok_or_else(|| "Not connected to a Cindrix Hub yet.".to_string())
}

/// Sign in with the Hub's PIN (shown in Cindrix Ops under Settings, Sharing).
pub fn connect(addr: &str, pin: &str) -> Result<String, String> {
    let addr = normalize(addr);
    let resp = client()?.post(format!("{addr}/pin")).header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("pin={}", pin.trim())).send()
        .map_err(|e| format!("Could not reach the Hub at {addr}. Is this phone on the Hub's Wi-Fi hotspot, and is sharing switched on? ({e})"))?;
    let cookie = resp.headers().get_all("set-cookie").iter().filter_map(|v| v.to_str().ok())
        .find_map(|v| v.split(';').next().filter(|kv| kv.starts_with("cx=")).map(|s| s.to_string()));
    match (resp.status().as_u16(), cookie) {
        (_, Some(c)) => {
            *CONN.lock().unwrap() = Some(Conn { addr: addr.clone(), cookie: c });
            save();
            Ok(addr)
        }
        (429, _) => Err("Too many wrong PIN tries. Wait a minute and try again.".into()),
        (200, _) => Err("That PIN is not right.".into()),
        (s, _) => Err(format!("The Hub answered with an unexpected reply ({s}). Is that address a Cindrix Ops Hub?")),
    }
}

fn post(cmd: &str, body: Value) -> Result<Value, String> {
    let c = conn()?;
    let resp = client()?.post(format!("{}/api/{cmd}", c.addr)).header("Cookie", &c.cookie).header("Content-Type", "application/json")
        .body(body.to_string()).send().map_err(|e| format!("Lost the Hub ({e}). Check the Wi-Fi connection."))?;
    let status = resp.status().as_u16();
    if (300..400).contains(&status) || status == 401 { disconnect(); return Err("The Hub signed this phone out. Connect again with the PIN.".into()); }
    let text = resp.text().map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_str(&text).map_err(|_| "The Hub sent an answer this app does not understand.".to_string())?;
    if status >= 400 { return Err(v["error"].as_str().unwrap_or("The Hub refused that.").to_string()); }
    Ok(v)
}

/// Songs matching `text`, inside `folder` (only directly in it when `direct`).
pub fn query(text: &str, folder: Option<&str>, direct: bool, offset: usize, limit: usize) -> Result<Value, String> {
    post("music_query", json!({ "args": { "text": text, "folder": folder, "direct": direct }, "offset": offset, "limit": limit }))
}

/// The folders (categories) under `prefix`.
pub fn folders(prefix: Option<&str>) -> Result<Value, String> {
    post("music_facets", json!({ "kind": "folders", "prefix": prefix }))
}

fn enc_path(p: &str) -> String {
    p.split('/').map(|seg| seg.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect::<String>()).collect::<Vec<_>>().join("/")
}

/// Copy one song from the Hub to `dest` (skipped when it is already there).
pub fn fetch(rel: &str, dest: &Path) -> Result<PathBuf, String> {
    if dest.exists() { return Ok(dest.to_path_buf()); }
    let c = conn()?;
    let prepared = post("music_prepare", json!({ "rel": rel }))?;
    let prepared = prepared.as_str().ok_or("The Hub could not prepare that song.")?.to_string();
    if let Some(d) = dest.parent() { std::fs::create_dir_all(d).map_err(|e| e.to_string())?; }
    let part = dest.with_extension("part");
    let mut out = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let url = format!("{}/music/{}", c.addr, enc_path(&prepared));
    let cl = client()?;
    let mut start: u64 = 0;
    let result = (|| -> Result<(), String> {
        loop {
            let resp = cl.get(&url).header("Cookie", &c.cookie).header("Range", format!("bytes={start}-{}", start + 8 * 1024 * 1024 - 1)).send()
                .map_err(|e| format!("Lost the Hub while copying ({e})"))?;
            let status = resp.status().as_u16();
            let range = resp.headers().get("content-range").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
            if status == 416 { return Ok(()); }
            if status != 200 && status != 206 { return Err(format!("The Hub could not send that song ({status}).")); }
            let bytes = resp.bytes().map_err(|e| format!("Copy interrupted: {e}"))?;
            out.write_all(&bytes).map_err(|e| e.to_string())?;
            if status == 200 { return Ok(()); }
            // "bytes 0-8388607/12345678"
            let total = range.as_deref().and_then(|r| r.rsplit('/').next()).and_then(|t| t.parse::<u64>().ok());
            start += bytes.len() as u64;
            match total { Some(t) if start >= t => return Ok(()), None if bytes.is_empty() => return Ok(()), _ => {} }
        }
    })();
    drop(out);
    if let Err(e) = result { let _ = std::fs::remove_file(&part); return Err(e); }
    std::fs::rename(&part, dest).map_err(|e| e.to_string())?;
    Ok(dest.to_path_buf())
}

/// A safe local file name for a path on the Hub (no "..", no odd characters).
pub fn local_name(rel: &str) -> PathBuf {
    let mut p = PathBuf::new();
    for seg in rel.split(['/', '\\']).filter(|s| !s.is_empty() && *s != "." && *s != "..") {
        p.push(crate::podcasts::sanitize_name(seg));
    }
    p
}

/// Copy every song under `folder` into `root` (keeping the folder layout). Returns how many songs were copied.
pub fn save_folder(folder: &str, root: &Path) -> Result<usize, String> {
    let mut rels: Vec<String> = Vec::new();
    let mut offset = 0;
    loop {
        let page = query("", Some(folder), false, offset, 500)?;
        let tracks = page["tracks"].as_array().cloned().unwrap_or_default();
        if tracks.is_empty() { break; }
        offset += tracks.len();
        rels.extend(tracks.iter().filter_map(|t| t["rel"].as_str().map(|s| s.to_string())));
        if offset >= page["total"].as_u64().unwrap_or(0) as usize || offset >= 5000 { break; }
    }
    if rels.is_empty() { return Err("There are no songs in that folder.".into()); }
    *PROGRESS.lock().unwrap() = (0, rels.len());
    for (i, rel) in rels.iter().enumerate() {
        if let Err(e) = fetch(rel, &root.join(local_name(rel))) { *PROGRESS.lock().unwrap() = (0, 0); return Err(e); }
        PROGRESS.lock().unwrap().0 = i + 1;
    }
    *PROGRESS.lock().unwrap() = (0, 0);
    Ok(rels.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addresses_are_completed() {
        assert_eq!(normalize("192.168.43.1"), "http://192.168.43.1:8765");
        assert_eq!(normalize(" http://hub.local:9000/ "), "http://hub.local:9000");
    }
    #[test]
    fn paths_are_encoded_and_local_names_are_safe() {
        assert_eq!(enc_path("t/Rock/a b#.mp3"), "t/Rock/a%20b%23.mp3");
        assert_eq!(local_name("../Rock/../x/y.mp3"), PathBuf::from("Rock").join("x").join("y.mp3"));
    }
}
