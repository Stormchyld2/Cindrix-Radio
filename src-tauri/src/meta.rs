//! Optional human-editable `metadata.txt` in a media folder:
//!   filename | display title | date (YYYY-MM-DD) | description
//! Blank lines and lines starting with # are ignored. A literal pipe is written \|.
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta { pub title: Option<String>, pub date: Option<String>, pub description: Option<String> }

fn split_fields(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek() == Some(&'|') { cur.push('|'); chars.next(); }
        else if c == '|' { out.push(cur.trim().to_string()); cur.clear(); }
        else { cur.push(c); }
    }
    out.push(cur.trim().to_string());
    out
}

fn some(s: Option<&String>) -> Option<String> { s.filter(|v| !v.is_empty()).cloned() }

/// Keys are lower-cased file names.
pub fn parse(text: &str) -> HashMap<String, Meta> {
    let mut map = HashMap::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') { continue; }
        let f = split_fields(l);
        if f[0].is_empty() { continue; }
        map.insert(f[0].to_lowercase(), Meta { title: some(f.get(1)), date: some(f.get(2)), description: some(f.get(3)) });
    }
    map
}

pub fn load_dir(dir: &Path) -> HashMap<String, Meta> {
    std::fs::read_to_string(dir.join("metadata.txt")).map(|t| parse(&t)).unwrap_or_default()
}

fn esc(s: &str) -> String { s.replace('|', "\\|").replace(['\n', '\r'], " ") }

/// Add a line for `file` if the folder's metadata.txt has none. Existing lines are kept as is.
pub fn append_entry(dir: &Path, file: &str, title: &str, date: Option<&str>, description: Option<&str>) -> std::io::Result<()> {
    let path = dir.join("metadata.txt");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if parse(&existing).contains_key(&file.to_lowercase()) { return Ok(()); }
    let mut body = existing;
    if body.is_empty() {
        body.push_str("# filename | display title | date (YYYY-MM-DD) | description\n");
    } else if !body.ends_with('\n') { body.push('\n'); }
    body.push_str(&format!("{} | {} | {} | {}\n", esc(file), esc(title), date.unwrap_or(""), esc(description.unwrap_or(""))));
    let tmp = dir.join("metadata.txt.tmp");
    std::fs::write(&tmp, body)?;
    std::fs::rename(tmp, path)
}

/// Set the line for `file`: replace it if there is one, otherwise add it. Other lines,
/// comments and their order are kept. With every field empty the line is removed.
pub fn set_entry(dir: &Path, file: &str, title: &str, date: Option<&str>, description: Option<&str>) -> std::io::Result<()> {
    let path = dir.join("metadata.txt");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let key = file.to_lowercase();
    let clear = title.trim().is_empty() && date.map_or(true, |d| d.trim().is_empty()) && description.map_or(true, |d| d.trim().is_empty());
    let new_line = format!("{} | {} | {} | {}\n", esc(file), esc(title.trim()), date.unwrap_or("").trim(), esc(description.unwrap_or("").trim()));
    let (mut out, mut placed) = (String::new(), false);
    for line in existing.lines() {
        let l = line.trim().trim_start_matches('\u{feff}');
        let hit = !l.is_empty() && !l.starts_with('#') && split_fields(l)[0].to_lowercase() == key;
        if !hit { out.push_str(line); out.push('\n'); }
        else if !placed && !clear { out.push_str(&new_line); placed = true; }
    }
    if !placed && !clear {
        if out.is_empty() { out.push_str("# filename | display title | date (YYYY-MM-DD) | description\n"); }
        out.push_str(&new_line);
    }
    let tmp = dir.join("metadata.txt.tmp");
    std::fs::write(&tmp, out)?;
    std::fs::rename(tmp, path)
}

/// "Mon, 03 Nov 2026 10:00:00 GMT" -> "2026-11-03". Returns None for anything else.
pub fn iso_date(pub_date: &str) -> Option<String> {
    let s = pub_date.trim();
    if s.len() >= 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' { return Some(s[..10].to_string()); }
    let parts: Vec<&str> = s.split_whitespace().collect();
    let (d, m, y) = match parts.as_slice() {
        [_, d, m, y, ..] => (*d, *m, *y),
        [d, m, y, ..] => (*d, *m, *y),
        _ => return None,
    };
    let months = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
    let mi = months.iter().position(|x| m.to_lowercase().starts_with(x))? + 1;
    let (d, y): (u32, u32) = (d.parse().ok()?, y.parse().ok()?);
    Some(format!("{y:04}-{mi:02}-{d:02}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines_comments_and_escaped_pipes() {
        let t = "# comment\n\nep3740.mp3|Episode 3740: Off-Grid Solar Setup Basics|2026-11-08|Building a compact 300W battery backup bank.\nEP2.MP3 | A \\| B | |\n";
        let m = parse(t);
        let a = &m["ep3740.mp3"];
        assert_eq!(a.title.as_deref(), Some("Episode 3740: Off-Grid Solar Setup Basics"));
        assert_eq!(a.date.as_deref(), Some("2026-11-08"));
        assert_eq!(a.description.as_deref(), Some("Building a compact 300W battery backup bank."));
        let b = &m["ep2.mp3"];
        assert_eq!(b.title.as_deref(), Some("A | B"));
        assert_eq!(b.date, None);
    }

    #[test]
    fn missing_entry_is_simply_absent() {
        assert!(parse("a.mp3|T||").get("b.mp3").is_none());
        assert!(parse("").is_empty());
    }

    #[test]
    fn append_adds_once_and_keeps_existing_lines() {
        let dir = std::env::temp_dir().join(format!("cindrix-meta-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("metadata.txt"), "keep.mp3|Mine|2026-01-01|hand written").unwrap();
        append_entry(&dir, "new.mp3", "New | Show", Some("2026-11-08"), None).unwrap();
        append_entry(&dir, "new.mp3", "Other title", None, None).unwrap(); // already there: no change
        let m = load_dir(&dir);
        assert_eq!(m["keep.mp3"].title.as_deref(), Some("Mine"));
        assert_eq!(m["new.mp3"].title.as_deref(), Some("New | Show"));
        assert_eq!(std::fs::read_to_string(dir.join("metadata.txt")).unwrap().matches("new.mp3").count(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rss_dates_become_iso() {
        assert_eq!(iso_date("Mon, 03 Nov 2026 10:00:00 GMT").as_deref(), Some("2026-11-03"));
        assert_eq!(iso_date("2026-11-08T10:00:00Z").as_deref(), Some("2026-11-08"));
        assert_eq!(iso_date("garbage"), None);
    }

    #[test]
    fn set_entry_adds_replaces_keeps_others_and_clears() {
        let dir = std::env::temp_dir().join(format!("cindrix-radio-meta-set-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        set_entry(&dir, "Ch1.mp3", "Chapter One", Some("2026-01-02"), Some("Intro | part")).unwrap();
        set_entry(&dir, "ch2.mp3", "Chapter Two", None, None).unwrap();
        let m = load_dir(&dir);
        assert_eq!(m["ch1.mp3"].title.as_deref(), Some("Chapter One"));
        assert_eq!(m["ch1.mp3"].description.as_deref(), Some("Intro | part"), "pipes survive");
        set_entry(&dir, "CH1.MP3", "First", Some("2026-02-03"), None).unwrap();
        let m = load_dir(&dir);
        assert_eq!(m.len(), 2, "replaced, not duplicated");
        assert_eq!(m["ch1.mp3"].title.as_deref(), Some("First"));
        assert_eq!(m["ch1.mp3"].date.as_deref(), Some("2026-02-03"));
        assert_eq!(m["ch1.mp3"].description, None);
        assert_eq!(m["ch2.mp3"].title.as_deref(), Some("Chapter Two"), "other lines kept");
        set_entry(&dir, "ch1.mp3", "", None, Some("  ")).unwrap();
        let m = load_dir(&dir);
        assert!(!m.contains_key("ch1.mp3") && m.contains_key("ch2.mp3"), "empty fields remove the line");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
