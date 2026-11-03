use crate::config::{Config, NOW_PLAYING_ENDPOINT, TOKEN_ENDPOINT};
use base64::{engine::general_purpose::STANDARD, Engine};
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
}

fn basic(cfg: &Config) -> String {
    STANDARD.encode(format!("{}:{}", cfg.client_id, cfg.client_secret))
}

pub fn authorize_url(cfg: &Config) -> String {
    let q = form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", &cfg.client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &cfg.redirect_uri)
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
            eprintln!("Spotify token request failed ({code}): {:?}", res.into_string());
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
}

fn access_token(cfg: &Config) -> Option<String> {
    token_request(
        cfg,
        &[("grant_type", "refresh_token"), ("refresh_token", &cfg.refresh_token)],
    )?
    .access_token
}

fn format_track(data: &Value) -> Option<TrackInfo> {
    let item = data.get("item").filter(|i| !i.is_null())?;
    if data.get("currently_playing_type")?.as_str()? != "track" {
        return None;
    }
    let artist = item
        .get("artists")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|x| x["name"].as_str()).collect::<Vec<_>>().join(", "))
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
            eprintln!("Spotify now-playing request failed ({code}): {:?}", res.into_string());
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
    let res = ureq::get(url).call().map_err(|e| eprintln!("Error fetching cover image: {e}")).ok()?;
    let mut bytes = Vec::new();
    res.into_reader().read_to_end(&mut bytes).ok()?;
    Some(format!("data:image/jpeg;base64,{}", STANDARD.encode(bytes)))
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
