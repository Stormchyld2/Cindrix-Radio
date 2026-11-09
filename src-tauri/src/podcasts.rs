//! Podcast feeds: RSS parsing, OPML import, local feed list, and episode downloads.
//! There is no server. The app fetches feeds itself and saves episodes to
//! <Podcasts folder>/<Show Name>/. Episodes are matched by feed GUID, not filename.
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Episode {
    pub guid: String,
    pub title: String,
    pub enclosure_url: String,
    pub published: Option<String>,
    pub duration_secs: Option<u32>,
    /// Local file once downloaded.
    pub file: Option<String>,
    pub played: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Feed {
    pub id: String,
    pub url: String,
    pub title: String,
    pub image: Option<String>,
    pub episodes: Vec<Episode>,
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

pub fn feed_id(url: &str) -> String { format!("{:016x}", fnv(url.trim())) }

pub fn sanitize_name(s: &str) -> String {
    let cleaned: String = s.chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) || c.is_control() { '_' } else { c })
        .collect();
    let t = cleaned.trim().trim_matches('.').trim();
    let t: String = t.chars().take(80).collect();
    if t.is_empty() { "Untitled".into() } else { t }
}

fn parse_duration(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.trim().split(':').collect();
    let mut total: u32 = 0;
    if parts.is_empty() || parts.len() > 3 { return None; }
    for p in parts {
        let n: f64 = p.trim().parse().ok()?;
        total = total.checked_mul(60)?.checked_add(n as u32)?;
    }
    Some(total)
}

fn attr(e: &BytesStart, name: &str) -> Option<String> {
    e.attributes().flatten()
        .find(|a| a.key.as_ref() == name.as_bytes())
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

pub fn parse_feed(xml: &str, url: &str) -> Result<Feed, String> {
    let mut r = Reader::from_str(xml);
    r.config_mut().trim_text(true);
    let mut feed = Feed { id: feed_id(url), url: url.trim().to_string(), ..Default::default() };
    let mut stack: Vec<String> = Vec::new();
    let mut ep: Option<Episode> = None;
    let mut saw_channel = false;

    loop {
        match r.read_event() {
            Err(e) => return Err(format!("Not a valid feed: {e}")),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = open_tag(&e, &mut feed, &mut ep, &mut saw_channel);
                stack.push(name);
            }
            // self-closing tags (like <enclosure .../>) have no End event, so they are not pushed
            Ok(Event::Empty(e)) => {
                open_tag(&e, &mut feed, &mut ep, &mut saw_channel);
            }
            Ok(Event::End(_)) => {
                if let Some(name) = stack.pop() {
                    if name == "item" {
                        if let Some(mut done) = ep.take() {
                            if !done.enclosure_url.is_empty() {
                                if done.guid.is_empty() { done.guid = done.enclosure_url.clone(); }
                                feed.episodes.push(done);
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().map(|c| c.into_owned()).unwrap_or_default();
                set_text(&mut feed, &mut ep, &stack, text);
            }
            Ok(Event::CData(c)) => {
                let text = String::from_utf8_lossy(&c.into_inner()).to_string();
                set_text(&mut feed, &mut ep, &stack, text);
            }
            _ => {}
        }
    }
    if !saw_channel { return Err("This does not look like a podcast RSS feed".into()); }
    if feed.title.is_empty() { feed.title = feed.url.clone(); }
    Ok(feed)
}

fn open_tag(e: &BytesStart, feed: &mut Feed, ep: &mut Option<Episode>, saw_channel: &mut bool) -> String {
    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
    match name.as_str() {
        "channel" => *saw_channel = true,
        "item" => *ep = Some(Episode::default()),
        "enclosure" => {
            if let (Some(ep), Some(u)) = (ep.as_mut(), attr(e, "url")) { ep.enclosure_url = u; }
        }
        "itunes:image" => {
            if ep.is_none() && feed.image.is_none() { feed.image = attr(e, "href"); }
        }
        _ => {}
    }
    name
}

fn set_text(feed: &mut Feed, ep: &mut Option<Episode>, stack: &[String], text: String) {
    let top = match stack.last() { Some(t) => t.as_str(), None => return };
    let parent = if stack.len() >= 2 { stack[stack.len() - 2].as_str() } else { "" };
    match (ep.as_mut(), top) {
        (Some(e), "title") => e.title = text,
        (Some(e), "guid") => e.guid = text,
        (Some(e), "pubDate") => e.published = Some(text),
        (Some(e), "itunes:duration") => e.duration_secs = parse_duration(&text),
        (None, "title") if parent == "channel" => feed.title = text,
        (None, "url") if parent == "image" && feed.image.is_none() => feed.image = Some(text),
        _ => {}
    }
}

/// Keep what the user already has (downloads, played flags) when a feed is refreshed.
pub fn merge(old: &Feed, mut new: Feed) -> Feed {
    for e in new.episodes.iter_mut() {
        if let Some(o) = old.episodes.iter().find(|o| o.guid == e.guid) {
            e.file = o.file.clone();
            e.played = o.played;
        }
    }
    // downloaded episodes that dropped out of the feed stay available
    for o in old.episodes.iter().filter(|o| o.file.is_some()) {
        if !new.episodes.iter().any(|e| e.guid == o.guid) { new.episodes.push(o.clone()); }
    }
    new
}

pub fn parse_opml(xml: &str) -> Vec<String> {
    let mut r = Reader::from_str(xml);
    let mut urls = Vec::new();
    loop {
        match r.read_event() {
            Ok(Event::Eof) | Err(_) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) if e.name().as_ref() == b"outline" => {
                if let Some(u) = attr(&e, "xmlUrl") { urls.push(u); }
            }
            _ => {}
        }
    }
    urls
}

pub fn is_http(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("http://") || u.starts_with("https://")
}

fn client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CindrixRadio/0.1")
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())
}

#[derive(Serialize, Clone, Debug)]
pub struct PodHit { pub name: String, pub author: String, pub feed_url: String, pub image: Option<String>, pub genre: String }

/// Search the public podcast directory (Apple's free search, no account) by name, topic or idea.
pub fn search(q: &str) -> Result<Vec<PodHit>, String> {
    let q = q.trim();
    if q.is_empty() { return Err("Type a topic or a podcast name first".into()); }
    let body: serde_json::Value = client()?.get("https://itunes.apple.com/search")
        .query(&[("media", "podcast"), ("entity", "podcast"), ("limit", "25"), ("term", q)])
        .send().and_then(|r| r.error_for_status()).map_err(|e| format!("Could not search: {e}"))?
        .text().map_err(|e| e.to_string())
        .and_then(|t| serde_json::from_str(&t).map_err(|e| format!("Unexpected answer from the podcast directory: {e}")))?;
    let s = |v: &serde_json::Value| v.as_str().unwrap_or("").to_string();
    Ok(body["results"].as_array().cloned().unwrap_or_default().iter().filter_map(|r| {
        let feed_url = s(&r["feedUrl"]);
        if !is_http(&feed_url) { return None; }
        Some(PodHit { name: s(&r["collectionName"]), author: s(&r["artistName"]), feed_url,
            image: Some(s(&r["artworkUrl100"])).filter(|u| !u.is_empty()), genre: s(&r["primaryGenreName"]) })
    }).collect())
}

pub fn fetch_feed(url: &str) -> Result<Feed, String> {
    if !is_http(url) { return Err("Enter a web address starting with http:// or https://".into()); }
    let body = client()?.get(url.trim()).send().and_then(|r| r.error_for_status())
        .map_err(|e| format!("Could not load the feed: {e}"))?
        .text().map_err(|e| e.to_string())?;
    parse_feed(&body, url)
}

fn extension_of(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or("");
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some(e) if e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()) => e.to_ascii_lowercase(),
        _ => "mp3".into(),
    }
}

/// Download one episode into <root>/<Show>/. Returns the saved path.
pub fn download_episode(root: &Path, feed: &Feed, ep: &Episode) -> Result<PathBuf, String> {
    if !is_http(&ep.enclosure_url) { return Err("Episode has no valid download link".into()); }
    let dir = root.join(sanitize_name(&feed.title));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = format!("{}-{:08x}.{}", sanitize_name(&ep.title), fnv(&ep.guid) as u32, extension_of(&ep.enclosure_url));
    let dest = dir.join(&name);
    if dest.exists() { return Ok(dest); }
    let part = dir.join(format!("{name}.part"));
    let mut resp = client()?.get(&ep.enclosure_url).send().and_then(|r| r.error_for_status())
        .map_err(|e| format!("Download failed: {e}"))?;
    let expected = resp.content_length();
    let mut f = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let written = resp.copy_to(&mut f).map_err(|e| { let _ = std::fs::remove_file(&part); format!("Download interrupted: {e}") })?;
    f.flush().map_err(|e| e.to_string())?;
    if let Some(n) = expected {
        if n != written { let _ = std::fs::remove_file(&part); return Err("Download was cut short; try again".into()); }
    }
    std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    // keep the folder self-describing for other devices; failure here must not fail the download
    let _ = crate::meta::append_entry(&dir, &name, &ep.title, ep.published.as_deref().and_then(crate::meta::iso_date).as_deref(), None);
    Ok(dest)
}

/// Saved feed list (JSON in the app data folder).
#[derive(Clone)]
pub struct PodStore { pub feeds: Arc<Mutex<Vec<Feed>>>, pub path: PathBuf }

impl PodStore {
    pub fn load(path: PathBuf) -> PodStore {
        let feeds = std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<Vec<Feed>>(&s).ok()).unwrap_or_default();
        PodStore { feeds: Arc::new(Mutex::new(feeds)), path }
    }
    pub fn save(&self, feeds: &[Feed]) {
        if let Some(d) = self.path.parent() { let _ = std::fs::create_dir_all(d); }
        let tmp = self.path.with_extension("json.tmp");
        if std::fs::write(&tmp, serde_json::to_string_pretty(feeds).unwrap()).is_ok() {
            let _ = std::fs::rename(tmp, &self.path);
        }
    }
    pub fn upsert(&self, fresh: Feed) -> Vec<Feed> {
        let mut g = self.feeds.lock().unwrap();
        match g.iter().position(|f| f.id == fresh.id) {
            Some(i) => { let m = merge(&g[i], fresh); g[i] = m; }
            None => g.push(fresh),
        }
        self.save(&g);
        g.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS: &str = r#"<?xml version="1.0"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
 <channel>
  <title>Test Prepper Show</title>
  <itunes:image href="https://example.com/cover.jpg"/>
  <item>
    <title>Episode One &amp; Friends</title>
    <guid>guid-1</guid>
    <pubDate>Mon, 03 Nov 2026 10:00:00 GMT</pubDate>
    <itunes:duration>1:02:03</itunes:duration>
    <enclosure url="https://example.com/ep1.mp3?x=1" length="123" type="audio/mpeg"/>
  </item>
  <item>
    <title><![CDATA[Episode Two]]></title>
    <enclosure url="https://example.com/ep2.m4a" length="1" type="audio/mp4"/>
  </item>
  <item><title>No audio here</title></item>
 </channel>
</rss>"#;

    #[test]
    fn parses_title_image_and_episodes() {
        let f = parse_feed(RSS, "https://example.com/feed.xml").unwrap();
        assert_eq!(f.title, "Test Prepper Show");
        assert_eq!(f.image.as_deref(), Some("https://example.com/cover.jpg"));
        assert_eq!(f.episodes.len(), 2, "episode without audio is skipped");
        assert_eq!(f.episodes[0].title, "Episode One & Friends");
        assert_eq!(f.episodes[0].guid, "guid-1");
        assert_eq!(f.episodes[0].duration_secs, Some(3723));
        assert_eq!(f.episodes[0].enclosure_url, "https://example.com/ep1.mp3?x=1");
        assert_eq!(f.episodes[1].title, "Episode Two");
        assert_eq!(f.episodes[1].guid, "https://example.com/ep2.m4a", "falls back to link when no guid");
    }

    #[test]
    fn rejects_non_feed() {
        assert!(parse_feed("<html><body>hi</body></html>", "x").is_err());
    }

    #[test]
    fn refresh_keeps_downloads_and_played() {
        let mut old = parse_feed(RSS, "https://example.com/feed.xml").unwrap();
        old.episodes[0].file = Some("C:/a.mp3".into());
        old.episodes[0].played = true;
        let new = parse_feed(RSS, "https://example.com/feed.xml").unwrap();
        let m = merge(&old, new);
        assert_eq!(m.episodes[0].file.as_deref(), Some("C:/a.mp3"));
        assert!(m.episodes[0].played);
    }

    #[test]
    fn opml_lists_feed_urls() {
        let o = r#"<opml><body><outline text="A" xmlUrl="https://a.example/rss"/><outline text="Folder"><outline xmlUrl="https://b.example/rss"/></outline></body></opml>"#;
        assert_eq!(parse_opml(o), vec!["https://a.example/rss", "https://b.example/rss"]);
    }

    #[test]
    fn folder_names_are_safe() {
        assert_eq!(sanitize_name("A/B: C?"), "A_B_ C_");
        assert_eq!(sanitize_name("..."), "Untitled");
    }

    #[test]
    fn only_http_links_allowed() {
        assert!(is_http("https://x.y/z"));
        assert!(!is_http("file:///etc/passwd"));
    }

    #[test]
    fn duration_formats() {
        assert_eq!(parse_duration("90"), Some(90));
        assert_eq!(parse_duration("2:30"), Some(150));
        assert_eq!(parse_duration("bad"), None);
    }
}
