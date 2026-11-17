//! GitHub data: profile contribution calendar (GraphQL) or one repository's
//! commit history (REST), aggregated into a 53x7 day grid, plus an offline cache.
//!
//! Synchronous `ureq` only. No async runtime, no `reqwest`.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::config::{cache_path, write_atomic, Config, CACHE_VERSION, SourceMode};
use crate::security;

pub const SLOTS: usize = 53 * 7;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DayStat {
    pub date: String,
    pub level: u8,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    fetched_at: String,
    tz: String,
    days: Vec<DayStat>,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .user_agent("GitWallpaper/0.1 (+Windows tray wallpaper)")
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(20))
        .timeout_write(Duration::from_secs(20))
        .build()
}

/// `https://github.com/owner/repo(.git)(/)` -> `(owner, repo)`.
pub fn parse_repo_url(input: &str) -> Option<(String, String)> {
    let t = input.trim().trim_end_matches('/');
    let t = t.strip_suffix(".git").unwrap_or(t);
    let rest = if let Some(r) = t.split("github.com/").nth(1) {
        r
    } else if let Some(r) = t.strip_prefix("git@github.com:") {
        r
    } else if !t.contains('/') {
        return None;
    } else {
        t
    };
    let mut parts = rest.split('/').filter(|p| !p.is_empty());
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.split(['?', '#']).next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

/// A bare username is the only accepted non-URL shape for profile mode.
pub fn parse_username(input: &str) -> Option<String> {
    let t = input.trim().trim_matches('/');
    if t.is_empty() || t.contains(' ') || t.contains('/') {
        return None;
    }
    Some(t.to_string())
}

fn level_for(count: u32, max: u32) -> u8 {
    if count == 0 || max == 0 {
        return 0;
    }
    let scaled = (count as f32 / max as f32 * 4.0).ceil() as u32;
    scaled.clamp(1, 4) as u8
}

/// Project per-day counts onto a Sunday-aligned 53x7 grid ending this week.
pub fn build_grid(events: &HashMap<NaiveDate, u32>) -> Vec<DayStat> {
    let today = Local::now().date_naive();
    let sunday_this_week = today - ChronoDuration::days(today.weekday().num_days_from_sunday() as i64);
    let start = sunday_this_week - ChronoDuration::days(52 * 7);
    let max = events.values().copied().max().unwrap_or(0);

    (0..SLOTS)
        .map(|i| {
            let day = start + ChronoDuration::days(i as i64);
            let count = if day > today {
                0
            } else {
                events.get(&day).copied().unwrap_or(0)
            };
            DayStat {
                date: day.format("%Y-%m-%d").to_string(),
                level: level_for(count, max),
                count,
            }
        })
        .collect()
}

fn fetch_profile(user: &str, token: &str) -> Result<Vec<DayStat>, String> {
    if token.trim().is_empty() {
        return Err(
            "profile mode butuh Personal Access Token (GraphQL GitHub mewajibkannya)".into(),
        );
    }
    const QUERY: &str = "query($login:String!){user(login:$login){contributionsCollection{contributionCalendar{weeks{contributionDays{date contributionCount}}}}}}";
    let body = serde_json::json!({ "query": QUERY, "variables": { "login": user } });
    let resp = agent()
        .post("https://api.github.com/graphql")
        .set("Authorization", &format!("Bearer {}", token.trim()))
        .set("Accept", "application/json")
        .send_json(body);

    let value: serde_json::Value = match resp {
        Ok(r) => r
            .into_json()
            .map_err(|e| format!("baca respons GraphQL: {e}"))?,
        Err(ureq::Error::Status(code, r)) => {
            return Err(format!("GitHub GraphQL HTTP {code}: {}", r.status_text()))
        }
        Err(e) => return Err(format!("GraphQL transport: {e}")),
    };

    if let Some(errors) = value.get("errors").and_then(|v| v.as_array()) {
        if let Some(first) = errors.first() {
            let msg = first
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown GraphQL error");
            return Err(format!("GitHub GraphQL: {msg}"));
        }
    }

    let weeks = value
        .pointer("/data/user/contributionsCollection/contributionCalendar/weeks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("respons GraphQL tanpa data user '{user}' (username salah?)"))?;

    let mut events: HashMap<NaiveDate, u32> = HashMap::new();
    for week in weeks {
        let Some(days) = week.get("contributionDays").and_then(|v| v.as_array()) else {
            continue;
        };
        for d in days {
            let (Some(date), count) = (
                d.get("date").and_then(|v| v.as_str()),
                d.get("contributionCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
            ) else {
                continue;
            };
            if let Ok(day) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                *events.entry(day).or_insert(0) += count;
            }
        }
    }
    Ok(build_grid(&events))
}

fn fetch_repository(owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<DayStat>, String> {
    let since = (Local::now() - ChronoDuration::days(364)).format("%Y-%m-%dT00:00:00Z");
    let mut events: HashMap<NaiveDate, u32> = HashMap::new();

    for page in 1..=10u32 {
        let url = format!(
            "https://api.github.com/repos/{owner}/{repo}/commits?since={since}&per_page=100&page={page}"
        );
        let mut req = agent()
            .get(&url)
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(t) = token {
            if !t.trim().is_empty() {
                req = req.set("Authorization", &format!("Bearer {}", t.trim()));
            }
        }
        let commits: Vec<serde_json::Value> = match req.call() {
            Ok(r) => r
                .into_json()
                .map_err(|e| format!("baca respons commits: {e}"))?,
            Err(ureq::Error::Status(409, _)) => break, // empty repository
            Err(ureq::Error::Status(code, r)) => {
                let hint = if code == 404 {
                    " (repo tidak ada / privat tanpa token)"
                } else if code == 403 {
                    " (rate limit / butuh token)"
                } else {
                    ""
                };
                return Err(format!(
                    "GitHub REST HTTP {code}{hint}: {}",
                    r.status_text()
                ));
            }
            Err(e) => return Err(format!("REST transport: {e}")),
        };
        if commits.is_empty() {
            break;
        }
        let len = commits.len();
        for c in commits {
            let Some(iso) = c
                .pointer("/commit/author/date")
                .and_then(|v| v.as_str())
                .or_else(|| c.pointer("/commit/committer/date").and_then(|v| v.as_str()))
            else {
                continue;
            };
            let Ok(ts) = DateTime::parse_from_rfc3339(iso) else {
                continue;
            };
            let day = ts.with_timezone(&Local).date_naive();
            *events.entry(day).or_insert(0) += 1;
        }
        if len < 100 {
            break;
        }
    }
    Ok(build_grid(&events))
}

pub fn token_of(cfg: &Config) -> Result<String, String> {
    security::decrypt(&cfg.source.encrypted_api_key)
}

/// Fetch exactly what the config asks for. Never falls back to the cache here;
/// see [`resolve`] for that.
pub fn fetch(cfg: &Config) -> Result<Vec<DayStat>, String> {
    if cfg.source.input_target.trim().is_empty() {
        return Err("target GitHub belum diisi (buka Settings)".into());
    }
    let token = token_of(cfg)?;
    match cfg.source.mode {
        SourceMode::Profile => {
            let user = parse_username(&cfg.source.input_target)
                .ok_or_else(|| "username GitHub tidak valid".to_string())?;
            fetch_profile(&user, &token)
        }
        SourceMode::Repository => {
            let (owner, repo) = parse_repo_url(&cfg.source.input_target)
                .ok_or_else(|| "URL repository tidak valid".to_string())?;
            fetch_repository(&owner, &repo, Some(token.as_str()))
        }
    }
}

pub fn load_cache() -> Option<Vec<DayStat>> {
    let text = std::fs::read_to_string(cache_path()).ok()?;
    let cache: CacheFile = serde_json::from_str(&text).ok()?;
    if cache.version != CACHE_VERSION || cache.days.is_empty() {
        return None;
    }
    Some(cache.days)
}

pub fn save_cache(days: &[DayStat]) -> Result<(), String> {
    let cache = CacheFile {
        version: CACHE_VERSION,
        fetched_at: Local::now().to_rfc3339(),
        tz: Local::now().offset().to_string(),
        days: days.to_vec(),
    };
    let text = serde_json::to_string(&cache).map_err(|e| format!("serialize cache: {e}"))?;
    write_atomic(&cache_path(), text.as_bytes())
}

/// Fetch, persist on success, fall back to the cache on failure. The second
/// value carries the reason the network fetch failed (for the tray tooltip).
pub fn resolve(cfg: &Config) -> (Vec<DayStat>, Option<String>) {
    match fetch(cfg) {
        Ok(days) => {
            let _ = save_cache(&days);
            (days, None)
        }
        Err(err) => match load_cache() {
            Some(days) => (days, Some(err)),
            None => (build_grid(&HashMap::new()), Some(err)),
        },
    }
}

/// Deterministic synthetic year, used by `--render-once` before the first fetch.
pub fn demo_days() -> Vec<DayStat> {
    let mut events: HashMap<NaiveDate, u32> = HashMap::new();
    let today = Local::now().date_naive();
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    for back in 0..364 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let day = today - ChronoDuration::days(back);
        let weekday = day.weekday().num_days_from_sunday();
        let weekend_penalty = if weekday >= 5 { 3 } else { 1 };
        let count = ((state >> 33) % 9) as u32 / weekend_penalty;
        if count > 0 {
            events.insert(day, count);
        }
    }
    build_grid(&events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_url_shapes() {
        assert_eq!(
            parse_repo_url("https://github.com/rust-lang/rust"),
            Some(("rust-lang".into(), "rust".into()))
        );
        assert_eq!(
            parse_repo_url("https://github.com/rust-lang/rust.git/"),
            Some(("rust-lang".into(), "rust".into()))
        );
        assert_eq!(
            parse_repo_url("git@github.com:owner/repo.git"),
            Some(("owner".into(), "repo".into()))
        );
        assert_eq!(parse_repo_url("not a url"), None);
    }

    #[test]
    fn grid_is_53x7_and_ends_with_today() {
        let days = build_grid(&HashMap::new());
        assert_eq!(days.len(), SLOTS);
        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
        assert!(days.iter().any(|d| d.date == today));
        assert!(days.iter().all(|d| d.level <= 4));
    }

    #[test]
    fn levels_are_relative() {
        assert_eq!(level_for(0, 10), 0);
        assert_eq!(level_for(1, 10), 1);
        assert_eq!(level_for(10, 10), 4);
    }
}
