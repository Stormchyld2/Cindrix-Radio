//! Radio stations: the user's saved list (a plain, portable radio_stations.json in the
//! Radio folder) and the Radio Browser community directory.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub url: String,
    pub favicon: Option<String>,
    pub tags: Option<String>,
    pub country: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    /// Radio Browser id, when the station came from the directory.
    pub rb_uuid: Option<String>,
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() { h ^= b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}

pub fn id_for(url: &str, rb_uuid: Option<&str>) -> String {
    rb_uuid.map(|u| u.to_string()).unwrap_or_else(|| format!("u{:016x}", fnv(url.trim())))
}

#[derive(Serialize, Deserialize, Default)]
struct File { stations: Vec<Station> }

#[derive(Clone)]
pub struct StationStore { stations: Arc<Mutex<Vec<Station>>>, path: PathBuf }

impl StationStore {
    pub fn load(path: PathBuf) -> StationStore {
        let stations = std::fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<File>(&s).ok()).map(|f| f.stations).unwrap_or_default();
        StationStore { stations: Arc::new(Mutex::new(stations)), path }
    }

    fn save(&self, s: &[Station]) {
        if let Some(d) = self.path.parent() { let _ = std::fs::create_dir_all(d); }
        let tmp = self.path.with_extension("json.tmp");
        let body = serde_json::to_string_pretty(&File { stations: s.to_vec() }).unwrap();
        if std::fs::write(&tmp, body).is_ok() { let _ = std::fs::rename(tmp, &self.path); }
    }

    pub fn list(&self) -> Vec<Station> { self.stations.lock().unwrap().clone() }

    pub fn add(&self, mut st: Station) -> Result<Vec<Station>, String> {
        if !crate::podcasts::is_http(&st.url) { return Err("A stream address must start with http:// or https://".into()); }
        st.url = st.url.trim().to_string();
        st.name = st.name.trim().to_string();
        if st.name.is_empty() { st.name = st.url.clone(); }
        st.id = id_for(&st.url, st.rb_uuid.as_deref());
        let mut g = self.stations.lock().unwrap();
        if let Some(existing) = g.iter_mut().find(|s| s.id == st.id || s.url == st.url) { *existing = st; }
        else { g.push(st); }
        self.save(&g);
        Ok(g.clone())
    }

    pub fn remove(&self, id: &str) -> Vec<Station> {
        let mut g = self.stations.lock().unwrap();
        g.retain(|s| s.id != id);
        self.save(&g);
        g.clone()
    }

    /// Add every station from an .m3u/.pls list. Returns how many were new.
    pub fn import(&self, text: &str) -> (Vec<Station>, usize) {
        let before = self.list().len();
        for (name, url) in crate::stream::parse_station_list(text) {
            let host = url.split('/').nth(2).unwrap_or(&url).to_string();
            let _ = self.add(Station { name: name.unwrap_or(host), url, ..Default::default() });
        }
        let all = self.list();
        let added = all.len().saturating_sub(before);
        (all, added)
    }
}

// ---- Radio Browser directory -------------------------------------------------------

const FALLBACK_SERVER: &str = "https://de1.api.radio-browser.info";
static SERVERS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Server discovery as the directory asks: look up all.api.radio-browser.info and
/// reverse-resolve the addresses to server names.
fn servers() -> Vec<String> {
    {
        let g = SERVERS.lock().unwrap();
        if !g.is_empty() { return g.clone(); }
    }
    use std::net::ToSocketAddrs;
    let mut names: Vec<String> = Vec::new();
    if let Ok(addrs) = ("all.api.radio-browser.info", 443).to_socket_addrs() {
        for a in addrs {
            if let Ok(host) = dns_lookup::lookup_addr(&a.ip()) {
                let url = format!("https://{host}");
                if host.contains("radio-browser") && !names.contains(&url) { names.push(url); }
            }
        }
    }
    if names.is_empty() { names.push(FALLBACK_SERVER.to_string()); }
    // spread load: rotate the starting server by the clock
    let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos() as usize).unwrap_or(0) % names.len();
    names.rotate_left(n);
    *SERVERS.lock().unwrap() = names.clone();
    names
}

pub fn parse_directory(json: &str) -> Vec<Station> {
    let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(json) else { return vec![] };
    let text = |v: &Value, k: &str| v.get(k).and_then(|x| x.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let num = |v: &Value, k: &str| v.get(k).and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.trim().parse().ok())));
    rows.iter().filter_map(|v| {
        let url = text(v, "url_resolved").or_else(|| text(v, "url"))?;
        let hls = v.get("hls").map_or(false, |h| h.as_u64() == Some(1) || h.as_bool() == Some(true));
        if hls || url.to_ascii_lowercase().split('?').next().unwrap_or("").ends_with(".m3u8") { return None; }
        let uuid = text(v, "stationuuid");
        Some(Station {
            id: id_for(&url, uuid.as_deref()),
            name: text(v, "name")?,
            url,
            favicon: text(v, "favicon"),
            tags: text(v, "tags"),
            country: text(v, "countrycode").or_else(|| text(v, "country")),
            codec: text(v, "codec"),
            bitrate: num(v, "bitrate").map(|b| b as u32).filter(|b| *b > 0),
            rb_uuid: uuid,
        })
    }).collect()
}

fn http() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent("CindrixRadio/0.1")
        .timeout(std::time::Duration::from_secs(15))
        .build().map_err(|e| e.to_string())
}

const ORDERS: [&str; 6] = ["votes", "clickcount", "clicktrend", "bitrate", "name", "random"];

fn clean_order(o: Option<&str>) -> &'static str {
    ORDERS.iter().find(|x| Some(**x) == o).copied().unwrap_or("votes")
}

fn clean_country(c: Option<&str>) -> Option<String> {
    c.map(|c| c.trim().to_ascii_uppercase()).filter(|c| c.len() == 2 && c.chars().all(|ch| ch.is_ascii_alphabetic()))
}

/// What to ask the directory: a typed search looks in station names AND genre tags, a
/// category looks in each of its tags, and nothing typed returns the top list.
pub fn plan(query: &str, tags: &[String]) -> Vec<(Option<String>, Option<String>)> {
    let q = query.trim();
    let q = if q.is_empty() { None } else { Some(q.to_string()) };
    let tags: Vec<String> = tags.iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
    if !tags.is_empty() { return tags.into_iter().map(|t| (q.clone(), Some(t))).collect(); }
    match q {
        None => vec![(None, None)],
        Some(q) => vec![(Some(q.clone()), None), (None, Some(q))],
    }
}

fn search_one(client: &reqwest::blocking::Client, name: Option<&str>, tag: Option<&str>, order: &str, cc: Option<&str>, limit: u32, offset: u32) -> Result<Vec<Station>, String> {
    let mut last = String::from("No directory server answered");
    for server in servers() {
        let mut url = reqwest::Url::parse(&format!("{server}/json/stations/search")).map_err(|e| e.to_string())?;
        {
            let mut q = url.query_pairs_mut();
            if let Some(n) = name { q.append_pair("name", n); }
            if let Some(t) = tag { q.append_pair("tag", t); }
            if let Some(c) = cc { q.append_pair("countrycode", c); }
            q.append_pair("order", order).append_pair("reverse", "true").append_pair("hidebroken", "true")
                .append_pair("limit", &limit.to_string()).append_pair("offset", &offset.to_string());
        }
        match client.get(url).send().and_then(|r| r.error_for_status()).and_then(|r| r.text()) {
            Ok(body) => return Ok(parse_directory(&body)),
            Err(e) => last = format!("Directory unavailable: {e}"),
        }
    }
    Err(last)
}

/// Search the directory by station name, genre tag, category and country, ordered by
/// votes, plays or trend. Broken stations are hidden.
pub fn search(query: &str, tags: &[String], order: Option<&str>, country: Option<&str>, limit: u32, offset: u32) -> Result<Vec<Station>, String> {
    let client = http()?;
    let (order, cc) = (clean_order(order), clean_country(country));
    let jobs = plan(query, tags);
    let mut out: Vec<Station> = Vec::new();
    let mut errors = Vec::new();
    for (name, tag) in &jobs {
        match search_one(&client, name.as_deref(), tag.as_deref(), order, cc.as_deref(), limit, offset) {
            Ok(rows) => for r in rows { if !out.iter().any(|o| o.id == r.id || o.url == r.url) { out.push(r); } },
            Err(e) => errors.push(e),
        }
    }
    if out.is_empty() && errors.len() == jobs.len() { return Err(errors.remove(0)); }
    out.truncate(limit as usize);
    Ok(out)
}

/// Tell the directory a station was played (it asks apps to do this so popularity stays
/// meaningful). Best effort only: playback never waits on it.
pub fn count_click(uuid: &str) {
    let uuid = uuid.to_string();
    std::thread::spawn(move || {
        if let Ok(c) = http() {
            for server in servers().into_iter().take(2) {
                if c.get(format!("{server}/json/url/{uuid}")).send().map(|r| r.status().is_success()).unwrap_or(false) { break; }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cindrix-radio-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }
    fn st(name: &str, url: &str) -> Station { Station { name: name.into(), url: url.into(), ..Default::default() } }

    #[test]
    fn add_remove_and_persist_as_a_plain_file() {
        let dir = temp("a");
        let path = dir.join("radio_stations.json");
        let s = StationStore::load(path.clone());
        s.add(st("Alpha", "http://a.example/live")).unwrap();
        s.add(st("Beta", "https://b.example/stream")).unwrap();
        assert_eq!(StationStore::load(path.clone()).list().len(), 2, "survives a restart");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"Alpha\"") && text.contains("http://a.example/live"), "human-readable");
        let id = s.list()[0].id.clone();
        assert_eq!(s.remove(&id).len(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn same_url_is_not_added_twice_and_bad_urls_are_refused() {
        let dir = temp("b");
        let s = StationStore::load(dir.join("radio_stations.json"));
        s.add(st("Alpha", "http://a.example/live")).unwrap();
        s.add(st("Alpha renamed", "http://a.example/live")).unwrap();
        assert_eq!(s.list().len(), 1);
        assert_eq!(s.list()[0].name, "Alpha renamed");
        assert!(s.add(st("Bad", "file:///etc/passwd")).is_err());
        assert!(s.add(st("Bad", "not a url")).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn import_reads_m3u_and_counts_new_stations() {
        let dir = temp("c");
        let s = StationStore::load(dir.join("radio_stations.json"));
        s.add(st("Existing", "http://a.example/1")).unwrap();
        let (all, added) = s.import("#EXTM3U\n#EXTINF:-1,Alpha FM\nhttp://a.example/1\n#EXTINF:-1,Beta FM\nhttp://b.example/2\n");
        assert_eq!((all.len(), added), (2, 1));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn corrupt_file_gives_an_empty_list() {
        let dir = temp("d");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("radio_stations.json"), "{ nope").unwrap();
        assert!(StationStore::load(dir.join("radio_stations.json")).list().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn typed_search_looks_in_names_and_genres() {
        assert_eq!(plan("country", &[]), vec![(Some("country".into()), None), (None, Some("country".into()))]);
        assert_eq!(plan("  ", &[]), vec![(None, None)]);
        let t = vec!["prepper".to_string(), "survival".to_string()];
        assert_eq!(plan("", &t), vec![(None, Some("prepper".into())), (None, Some("survival".into()))]);
        assert_eq!(plan("fm", &t[..1]), vec![(Some("fm".into()), Some("prepper".into()))]);
    }

    #[test]
    fn sort_and_country_are_checked() {
        assert_eq!(clean_order(Some("clickcount")), "clickcount");
        assert_eq!(clean_order(Some("drop table")), "votes");
        assert_eq!(clean_order(None), "votes");
        assert_eq!(clean_country(Some("us")).as_deref(), Some("US"));
        assert_eq!(clean_country(Some("usa")), None);
        assert_eq!(clean_country(Some("1;")), None);
    }

    #[test]
    fn directory_json_is_read_leniently() {
        let json = r#"[
          {"stationuuid":"u-1","name":"Test FM","url":"http://x/old","url_resolved":"http://x/live","favicon":"https://x/i.png","tags":"news,talk","countrycode":"US","codec":"MP3","bitrate":128,"hls":0},
          {"stationuuid":"u-2","name":"Str Bitrate","url_resolved":"http://y/s","bitrate":"64","hls":false},
          {"stationuuid":"u-3","name":"Segmented","url_resolved":"http://z/a.m3u8","hls":1},
          {"stationuuid":"u-4","name":"No address","url_resolved":""}
        ]"#;
        let list = parse_directory(json);
        assert_eq!(list.len(), 2, "HLS and address-less stations are left out");
        assert_eq!(list[0].url, "http://x/live");
        assert_eq!(list[0].bitrate, Some(128));
        assert_eq!(list[0].rb_uuid.as_deref(), Some("u-1"));
        assert_eq!(list[1].bitrate, Some(64));
        assert!(parse_directory("not json").is_empty());
    }
}
