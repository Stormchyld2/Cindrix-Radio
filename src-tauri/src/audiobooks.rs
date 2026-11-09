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

static PROGRESS: std::sync::Mutex<(usize, usize)> = std::sync::Mutex::new((0, 0));

/// (chapters finished, chapters in total) for the book being downloaded right now.
pub fn progress() -> (usize, usize) { *PROGRESS.lock().unwrap() }

fn fetch_bytes(c: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>, String> {
    let mut resp = c.get(url).send().and_then(|r| r.error_for_status()).map_err(|e| format!("Download failed: {e}"))?;
    let expected = resp.content_length();
    let mut buf = Vec::with_capacity(expected.unwrap_or(0) as usize);
    std::io::copy(&mut resp, &mut buf).map_err(|e| format!("Download interrupted: {e}"))?;
    if expected.map_or(false, |x| x != buf.len() as u64) { return Err("A chapter was cut short; try again".into()); }
    Ok(buf)
}

/// Raw mp3 audio only: no ID3 tags at either end and no Xing/Info header frame, so chapters can be
/// joined end to end into one plain constant-bitrate file.
pub fn mp3_audio(data: &[u8]) -> &[u8] {
    let mut d = data;
    if d.len() > 10 && &d[..3] == b"ID3" {
        let size = ((d[6] as usize & 0x7f) << 21) | ((d[7] as usize & 0x7f) << 14) | ((d[8] as usize & 0x7f) << 7) | (d[9] as usize & 0x7f);
        let footer = if d[5] & 0x10 != 0 { 10 } else { 0 };
        d = &d[(10 + size + footer).min(d.len())..];
    }
    if d.len() > 128 && &d[d.len() - 128..d.len() - 125] == b"TAG" { d = &d[..d.len() - 128]; }
    // skip leading junk up to the first frame sync
    if let Some(p) = d.windows(2).position(|w| w[0] == 0xFF && (w[1] & 0xE0) == 0xE0) { d = &d[p..]; }
    if let Some(len) = frame_len(d) {
        let probe = &d[..len.min(d.len()).min(200)];
        if probe.windows(4).any(|w| w == b"Xing" || w == b"Info") { d = &d[len.min(d.len())..]; }
    }
    d
}

fn frame_len(d: &[u8]) -> Option<usize> {
    if d.len() < 4 || d[0] != 0xFF || (d[1] & 0xE0) != 0xE0 { return None; }
    let version = (d[1] >> 3) & 3; // 3 = MPEG1, 2 = MPEG2, 0 = MPEG2.5
    let layer = (d[1] >> 1) & 3; // 1 = Layer III
    if version == 1 || layer != 1 { return None; }
    let br_idx = (d[2] >> 4) as usize;
    let sr_idx = ((d[2] >> 2) & 3) as usize;
    if br_idx == 0 || br_idx == 15 || sr_idx == 3 { return None; }
    let pad = ((d[2] >> 1) & 1) as usize;
    let (br, sr, coef) = if version == 3 {
        ([0u32, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320][br_idx], [44100u32, 48000, 32000][sr_idx], 144_000u32)
    } else {
        ([0u32, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160][br_idx], if version == 2 { [22050u32, 24000, 16000][sr_idx] } else { [11025u32, 12000, 8000][sr_idx] }, 72_000u32)
    };
    Some((coef * br / sr) as usize + pad)
}

/// Download every chapter and join them into ONE file: <root>/<Title>/<Title>.mp3.
/// Returns the file and how many chapters it holds.
pub fn download(root: &Path, id: &str, title: &str) -> Result<(PathBuf, usize), String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c)) { return Err("That book has no valid address".into()); }
    let c = client()?;
    let meta: serde_json::Value = c.get(format!("https://archive.org/metadata/{id}"))
        .send().and_then(|r| r.error_for_status()).map_err(|e| format!("Could not open the book: {e}"))?
        .text().map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))?;
    let chapters = chapter_files(&meta);
    if chapters.is_empty() { return Err("This book has no mp3 chapters to download".into()); }
    let safe = crate::podcasts::sanitize_name(title);
    let dir = root.join(&safe);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(format!("{safe}.mp3"));
    *PROGRESS.lock().unwrap() = (0, chapters.len());
    if !dest.exists() {
        let part = dir.join(format!("{safe}.mp3.part"));
        let mut out = std::fs::File::create(&part).map_err(|e| e.to_string())?;
        let result = (|| -> Result<(), String> {
            for (n, name) in chapters.iter().enumerate() {
                let bytes = fetch_bytes(&c, &format!("https://archive.org/download/{id}/{}", encode(name)))?;
                out.write_all(mp3_audio(&bytes)).map_err(|e| e.to_string())?;
                PROGRESS.lock().unwrap().0 = n + 1;
            }
            out.flush().map_err(|e| e.to_string())
        })();
        drop(out);
        if let Err(e) = result { let _ = std::fs::remove_file(&part); *PROGRESS.lock().unwrap() = (0, 0); return Err(e); }
        std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    }
    // tidy up the separate chapter files an earlier version of the app saved for this same book
    for name in &chapters {
        let leaf = crate::podcasts::sanitize_name(name.rsplit('/').next().unwrap_or(name));
        let _ = std::fs::remove_file(dir.join(leaf));
    }
    *PROGRESS.lock().unwrap() = (0, 0);
    Ok((dest, chapters.len()))
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
    fn mp3_tags_and_xing_frame_are_stripped() {
        // ID3v2 (size 4) + junk-free MPEG1 L3 128 kbps 44.1k frame holding "Xing" + one real frame + ID3v1
        let mut d = b"ID3\x03\x00\x00\x00\x00\x00\x04abcd".to_vec();
        let mut frame = vec![0xFF, 0xFB, 0x90, 0x00];
        frame.resize(417, 0); // 144000*128/44100 = 417
        frame[36..40].copy_from_slice(b"Xing");
        d.extend(&frame);
        let mut real = vec![0xFF, 0xFB, 0x90, 0x00];
        real.resize(417, 7);
        d.extend(&real);
        let mut v1 = b"TAG".to_vec();
        v1.resize(128, 0);
        d.extend(&v1);
        let out = mp3_audio(&d);
        assert_eq!(out.len(), 417);
        assert_eq!(out[10], 7);
    }

    #[test]
    fn prefers_64kbps_chapters_in_order() {
        let m: serde_json::Value = serde_json::json!({"files":[
            {"name":"b_64kb.mp3","format":"64Kbps MP3"},{"name":"a_64kb.mp3","format":"64Kbps MP3"},
            {"name":"x.zip","format":"ZIP"},{"name":"a.mp3","format":"VBR MP3"}]});
        assert_eq!(chapter_files(&m), vec!["a_64kb.mp3", "b_64kb.mp3"]);
    }
}
