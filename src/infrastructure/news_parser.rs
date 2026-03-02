use feed_rs::{model::Feed, parser};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const PHORONIX_RSS_URL: &str = "https://www.phoronix.com/rss.php";
pub const FOSSLINUX_RSS_URL: &str = "https://fosslinux.com/feed";
pub const ITSFOSS_RSS_URL: &str = "https://itsfoss.com/rss/";
pub const CACHE_DIR: &str = "/tmp";
pub const CACHE_FILE_PREFIX: &str = "tealinux-rss-cache-";
pub const CACHE_TTL_SECS: u64 = 3600;

pub type DynError = Box<dyn Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct NewsParser {
    client: Client,
}

impl NewsParser {
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("tealinux-modularitea-libs2/news-parser")
            .build()?;

        Ok(Self { client })
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    pub fn parse_from_str(xml: &str) -> Result<Feed, parser::ParseFeedError> {
        parser::parse(xml.as_bytes())
    }

    pub fn fetch_xml(&self, url: &str) -> Result<String, DynError> {
        let response = self.client.get(url).send()?.error_for_status()?;
        Ok(response.text()?)
    }

    pub fn fetch_and_parse(&self, url: &str) -> Result<Feed, DynError> {
        let xml = self.fetch_xml(url)?;
        Ok(parser::parse(xml.as_bytes())?)
    }

    pub fn phoronix_fetcher(&self) -> Result<Vec<ParsedNewsItem>, DynError> {
        let feed = self.fetch_and_parse(PHORONIX_RSS_URL)?;
        self.feed_to_items(feed)
    }

    pub fn fosslinux_fetcher(&self) -> Result<Vec<ParsedNewsItem>, DynError> {
        let feed = self.fetch_and_parse(FOSSLINUX_RSS_URL)?;
        self.feed_to_items(feed)
    }

    pub fn itsfoss_fetcher(&self) -> Result<Vec<ParsedNewsItem>, DynError> {
        let feed = self.fetch_and_parse(ITSFOSS_RSS_URL)?;
        self.feed_to_items(feed)
    }

    /// Blackbox combiner for all configured sources (cache-aware).
    pub fn blackbox_fetcher(&self) -> Result<Vec<ParsedNewsItem>, DynError> {
        self.get_cached_or_fetch(false)
    }

    /// Force refresh cache and return latest combined content.
    pub fn force_refresh_cache(&self) -> Result<Vec<ParsedNewsItem>, DynError> {
        self.get_cached_or_fetch(true)
    }

    fn get_cached_or_fetch(&self, force_refresh: bool) -> Result<Vec<ParsedNewsItem>, DynError> {
        let now = Self::now_unix_secs()?;

        if !force_refresh {
            if let Some((cache_path, ts)) = Self::find_latest_cache_file()? {
                if Self::is_cache_valid(ts, now) {
                    return Self::load_cache(&cache_path);
                }
            }
        }

        let fresh = self.fetch_all_sources()?;
        Self::write_cache(&fresh, now)?;
        Self::cleanup_old_cache_files(now)?;
        Ok(fresh)
    }

    fn fetch_all_sources(&self) -> Result<Vec<ParsedNewsItem>, DynError> {
        let mut out = Vec::new();
        out.extend(self.phoronix_fetcher()?);
        out.extend(self.fosslinux_fetcher()?);
        out.extend(self.itsfoss_fetcher()?);
        Ok(out)
    }

    fn now_unix_secs() -> Result<u64, DynError> {
        Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
    }

    fn cache_file_path(ts: u64) -> PathBuf {
        Path::new(CACHE_DIR).join(format!("{CACHE_FILE_PREFIX}{ts}.json"))
    }

    fn parse_cache_timestamp(name: &str) -> Option<u64> {
        if !name.starts_with(CACHE_FILE_PREFIX) || !name.ends_with(".json") {
            return None;
        }
        let ts = &name[CACHE_FILE_PREFIX.len()..name.len() - 5];
        ts.parse::<u64>().ok()
    }

    fn find_latest_cache_file() -> Result<Option<(PathBuf, u64)>, DynError> {
        let mut latest: Option<(PathBuf, u64)> = None;

        for entry in fs::read_dir(CACHE_DIR)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let Some(ts) = Self::parse_cache_timestamp(name) else {
                continue;
            };

            match latest {
                Some((_, latest_ts)) if latest_ts >= ts => {}
                _ => latest = Some((path, ts)),
            }
        }

        Ok(latest)
    }

    fn is_cache_valid(ts: u64, now: u64) -> bool {
        now.saturating_sub(ts) < CACHE_TTL_SECS
    }

    fn load_cache(path: &Path) -> Result<Vec<ParsedNewsItem>, DynError> {
        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    fn write_cache(items: &[ParsedNewsItem], ts: u64) -> Result<(), DynError> {
        let raw = serde_json::to_string_pretty(items)?;
        fs::write(Self::cache_file_path(ts), raw)?;
        Ok(())
    }

    fn cleanup_old_cache_files(keep_ts: u64) -> Result<(), DynError> {
        for entry in fs::read_dir(CACHE_DIR)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let Some(ts) = Self::parse_cache_timestamp(name) else {
                continue;
            };

            if ts != keep_ts {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }

    fn feed_to_items(&self, feed: Feed) -> Result<Vec<ParsedNewsItem>, DynError> {
        let mut out = Vec::with_capacity(feed.entries.len());

        for entry in feed.entries {
            let url = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default();

            let title = entry
                .title
                .as_ref()
                .map(|t| t.content.clone())
                .unwrap_or_default();

            let descriptive = entry
                .summary
                .as_ref()
                .map(|s| s.content.clone())
                .or_else(|| entry.content.as_ref().and_then(|c| c.body.clone()))
                .unwrap_or_default();

            let mut thumbnail = entry
                .links
                .iter()
                .find(|l| {
                    l.media_type
                        .as_deref()
                        .map(|m| m.starts_with("image/"))
                        .unwrap_or(false)
                        || l.rel.as_deref() == Some("enclosure")
                })
                .map(|l| l.href.clone());

            if thumbnail.is_none() && !url.is_empty() {
                if let Ok(html) = self.fetch_xml(&url) {
                    thumbnail = Self::extract_og_image(&html);
                }
            }

            out.push(ParsedNewsItem {
                url,
                title,
                descriptive,
                thumbnail,
            });
        }

        Ok(out)
    }

    fn extract_og_image(html: &str) -> Option<String> {
        for needle in [
            "property=\"og:image\"",
            "property='og:image'",
            "name=\"og:image\"",
            "name='og:image'",
        ] {
            if let Some(meta_pos) = html.find(needle) {
                let tail = &html[meta_pos..];
                if let Some(content_pos) = tail.find("content=") {
                    let value = &tail[content_pos + "content=".len()..];
                    let mut chars = value.chars();
                    let quote = chars.next()?;
                    if quote == '"' || quote == '\'' {
                        let rest = &value[1..];
                        if let Some(end) = rest.find(quote) {
                            let candidate = rest[..end].trim();
                            if !candidate.is_empty() {
                                return Some(candidate.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedNewsItem {
    pub url: String,
    pub title: String,
    pub descriptive: String,
    pub thumbnail: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn parse_inline_xml_works() {
    //     let xml = r#"
    //     <feed xmlns="http://www.w3.org/2005/Atom">
    //        <title type="text">sample feed</title>
    //        <updated>2005-07-31T12:29:29Z</updated>
    //        <id>feed1</id>
    //        <entry>
    //            <title>sample entry</title>
    //            <id>entry1</id>
    //            <updated>2005-07-31T12:29:29Z</updated>
    //        </entry>
    //     </feed>
    //     "#;

    //     let feed = NewsParser::parse_from_str(xml).unwrap();
    //     assert_eq!(feed.title.map(|t| t.content), Some("sample feed".to_string()));
    //     assert_eq!(feed.entries.len(), 1);
    // }
}
