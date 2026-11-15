use crate::config::{Config, NOW_PLAYING_ENDPOINT, TOKEN_ENDPOINT, TOP_ENDPOINT};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TrackInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    pub is_playing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

fn basic(cfg: &Config) -> String {
    STANDARD.encode(format!("{}:{}", cfg.client_id, cfg.client_secret))
}

pub fn authorize_url(cfg: &Config, state: &str) -> String {
    let q = form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", &cfg.client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &cfg.redirect_uri)
        .append_pair("state", state)
        .append_pair("scope", "user-read-currently-playing user-top-read")
        .finish();
    format!("https://accounts.spotify.com/authorize?{q}")
}

fn token_request(cfg: &Config, form: &[(&str, &str)]) -> Option<TokenResponse> {
    match ureq::post(TOKEN_ENDPOINT)
        .set("Authorization", &format!("Basic {}", basic(cfg)))
        .send_form(form)
    {
        Ok(res) => res.into_json().ok(),
        Err(ureq::Error::Status(code, res)) => {
            eprintln!(
                "Spotify token request failed ({code}): {:?}",
                res.into_string()
            );
            None
        }
        Err(e) => {
            eprintln!("Spotify token request failed: {e}");
            None
        }
    }
}

pub fn exchange_code(cfg: &Config, code: &str) -> Option<String> {
    token_request(
        cfg,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &cfg.redirect_uri),
        ],
    )?
    .refresh_token
    .filter(|t| !t.is_empty())
    .inspect(|t| cfg.tokens.set_refresh_token(t))
}

fn access_token(cfg: &Config) -> Option<String> {
    if let Some(t) = cfg.tokens.cached_access_token() {
        return Some(t);
    }
    let refresh = cfg.tokens.refresh_token();
    let res = token_request(
        cfg,
        &[("grant_type", "refresh_token"), ("refresh_token", &refresh)],
    )?;
    let access = res.access_token?;
    if let Some(new) = res.refresh_token.filter(|t| !t.is_empty() && *t != refresh) {
        cfg.tokens.set_refresh_token(&new);
    }
    cfg.tokens
        .cache_access_token(&access, res.expires_in.unwrap_or(3600));
    Some(access)
}

fn format_track(data: &Value) -> Option<TrackInfo> {
    let item = data.get("item").filter(|i| !i.is_null())?;
    if data.get("currently_playing_type")?.as_str()? != "track" {
        return None;
    }
    let artist = item
        .get("artists")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|x| x["name"].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let cover = item["album"]["images"]
        .as_array()
        .and_then(|i| i.last())
        .and_then(|i| i["url"].as_str())
        .map(String::from);
    Some(TrackInfo {
        progress: Some(data["progress_ms"].as_u64().unwrap_or(0)),
        duration: item["duration_ms"].as_u64(),
        track: item["name"].as_str().map(String::from),
        artist: Some(artist),
        is_playing: data["is_playing"].as_bool().unwrap_or(false),
        cover_url: cover,
        url: item["external_urls"]["spotify"].as_str().map(String::from),
    })
}

fn current_track(cfg: &Config) -> Option<TrackInfo> {
    let token = access_token(cfg)?;
    let res = match ureq::get(NOW_PLAYING_ENDPOINT)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
    {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            if code == 401 {
                cfg.tokens.invalidate_access_token();
            }
            eprintln!(
                "Spotify now-playing request failed ({code}): {:?}",
                res.into_string()
            );
            return None;
        }
        Err(e) => {
            eprintln!("Spotify now-playing request failed: {e}");
            return None;
        }
    };
    if res.status() != 200 {
        return None;
    }
    format_track(&res.into_json().ok()?)
}

fn cover_base64(url: &str) -> Option<String> {
    let res = ureq::get(url)
        .call()
        .map_err(|e| eprintln!("Error fetching cover image: {e}"))
        .ok()?;
    let mut bytes = Vec::new();
    res.into_reader().read_to_end(&mut bytes).ok()?;
    Some(format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)))
}

pub struct TopItem {
    pub name: String,
    pub subtitle: String,
}

pub fn valid_time_range(r: &str) -> Option<&'static str> {
    ["short_term", "medium_term", "long_term"]
        .into_iter()
        .find(|v| *v == r)
}

fn top_items(cfg: &Config, kind: &str, time_range: &str) -> Vec<TopItem> {
    let Some(token) = access_token(cfg) else {
        return Vec::new();
    };
    let url = format!("{TOP_ENDPOINT}/{kind}?limit=5&time_range={time_range}");
    let res = match ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
    {
        Ok(res) => res,
        Err(ureq::Error::Status(code, res)) => {
            if code == 401 {
                cfg.tokens.invalidate_access_token();
            }
            eprintln!(
                "Spotify top {kind} request failed ({code}): {:?}",
                res.into_string()
            );
            return Vec::new();
        }
        Err(e) => {
            eprintln!("Spotify top {kind} request failed: {e}");
            return Vec::new();
        }
    };
    let Ok(data) = res.into_json::<Value>() else {
        return Vec::new();
    };
    data["items"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|i| {
                    let name = i["name"].as_str()?.to_string();
                    let subtitle = i["artists"]
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x["name"].as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    Some(TopItem { name, subtitle })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_top_artists(cfg: &Config, time_range: &str) -> Vec<TopItem> {
    top_items(cfg, "artists", time_range)
}

pub fn get_top_tracks(cfg: &Config, time_range: &str) -> Vec<TopItem> {
    top_items(cfg, "tracks", time_range)
}

pub fn get_now_playing(cfg: &Config, embed_cover: bool) -> TrackInfo {
    let Some(mut track) = current_track(cfg) else {
        return TrackInfo::default();
    };
    if embed_cover {
        track.cover_url = track.cover_url.as_deref().and_then(cover_base64);
    }
    track
}
