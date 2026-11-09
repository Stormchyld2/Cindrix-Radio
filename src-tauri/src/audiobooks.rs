//! Find free public-domain audiobooks (LibriVox, hosted by the Internet Archive) and save them
//! to Downloads/Cindrix/Audiobooks/<Title>/. Nothing here ships with the app; it only runs when the
//! user searches, and never in Grid-Down mode.
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Clone, Debug)]
pub struct BookHit { pub id: String, pub title: String, pub author: String, pub runtime: String }

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CindrixRadio/0.1")
        .connect_timeout(std::time::Duration::from_secs(15))
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())
}

fn text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(a) => a.iter().map(text).collect::<Vec<_>>().join(", "),
        serde_json::Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

/// Keep only letters, digits and spaces so the text is safe inside a search query.
fn clean_query(q: &str) -> String {
    q.chars().map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' }).collect::<String>()
        .split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn search(q: &str) -> Result<Vec<BookHit>, String> {
    let q = clean_query(q);
    if q.is_empty() { return Err("Type a book title or an author first".into()); }
    let lucene = format!("collection:librivoxaudio AND (title:({q}) OR creator:({q}))");
    let body: serde_json::Value = client()?.get("https://archive.org/advancedsearch.php")
        .query(&[("q", lucene.as_str()), ("fl[]", "identifier"), ("fl[]", "title"), ("fl[]", "creator"), ("fl[]", "runtime"),
                 ("rows", "30"), ("output", "json"), ("sort[]", "downloads desc")])
        .send().and_then(|r| r.error_for_status()).map_err(|e| format!("Could not search: {e}"))?
        .text().map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| format!("Unexpected answer from the book catalog: {e}")))?;
    let docs = body["response"]["docs"].as_array().cloned().unwrap_or_default();
    Ok(docs.iter().filter_map(|d| {
        let id = text(&d["identifier"]);
        if id.is_empty() { return None; }
        Some(BookHit { id, title: text(&d["title"]), author: text(&d["creator"]), runtime: text(&d["runtime"]) })
    }).collect())
}

fn encode(name: &str) -> String {
    name.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => (b as char).to_string(),
        _ => format!("%{b:02X}"),
    }).collect()
}

/// The chapter files of a book, in order: 64 kbps mp3 when present, otherwise any mp3 set.
fn chapter_files(meta: &serde_json::Value) -> Vec<String> {
    let files = meta["files"].as_array().cloned().unwrap_or_default();
    let pick = |fmt: &str| -> Vec<String> {
        let mut v: Vec<String> = files.iter()
            .filter(|f| f["format"].as_str().map_or(false, |x| x.eq_ignore_ascii_case(fmt)))
            .filter_map(|f| f["name"].as_str().map(|s| s.to_string())).collect();
        v.sort();
        v
    };
    for fmt in ["64Kbps MP3", "VBR MP3", "128Kbps MP3"] {
        let v = pick(fmt);
        if !v.is_empty() { return v; }
    }
    Vec::new()
}

fn save_one(c: &reqwest::blocking::Client, url: &str, dest: &Path) -> Result<(), String> {
    if dest.exists() { return Ok(()); }
    let part = dest.with_extension("part");
    let mut resp = c.get(url).send().and_then(|r| r.error_for_status()).map_err(|e| format!("Download failed: {e}"))?;
    let expected = resp.content_length();
    let mut f = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let n = resp.copy_to(&mut f).map_err(|e| { let _ = std::fs::remove_file(&part); format!("Download interrupted: {e}") })?;
    f.flush().map_err(|e| e.to_string())?;
    if expected.map_or(false, |x| x != n) { let _ = std::fs::remove_file(&part); return Err("A chapter was cut short; try again".into()); }
    std::fs::rename(&part, dest).map_err(|e| e.to_string())
}

/// Download every chapter into <root>/<Title>/. Returns the folder and how many files were saved.
pub fn download(root: &Path, id: &str, title: &str) -> Result<(PathBuf, usize), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) { return Err("That book has no valid address".into()); }
    let c = client()?;
    let meta: serde_json::Value = c.get(format!("https://archive.org/metadata/{id}"))
        .send().and_then(|r| r.error_for_status()).map_err(|e| format!("Could not open the book: {e}"))?
        .text().map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))?;
    let chapters = chapter_files(&meta);
    if chapters.is_empty() { return Err("This book has no mp3 chapters to download".into()); }
    let dir = root.join(crate::podcasts::sanitize_name(title));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for name in &chapters {
        let leaf = name.rsplit('/').next().unwrap_or(name);
        save_one(&c, &format!("https://archive.org/download/{id}/{}", encode(name)), &dir.join(crate::podcasts::sanitize_name(leaf)))?;
    }
    Ok((dir, chapters.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_is_cleaned_and_names_are_encoded() {
        assert_eq!(clean_query("  Pride & \"Prejudice\" (Austen) "), "Pride Prejudice Austen");
        assert_eq!(encode("a b/c#1.mp3"), "a%20b/c%231.mp3");
    }
    #[test]
    fn prefers_64kbps_chapters_in_order() {
        let m: serde_json::Value = serde_json::json!({"files":[
            {"name":"b_64kb.mp3","format":"64Kbps MP3"},{"name":"a_64kb.mp3","format":"64Kbps MP3"},
            {"name":"x.zip","format":"ZIP"},{"name":"a.mp3","format":"VBR MP3"}]});
        assert_eq!(chapter_files(&m), vec!["a_64kb.mp3", "b_64kb.mp3"]);
    }
}
