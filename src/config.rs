use crate::auth::TokenStore;
use std::env;
use std::path::PathBuf;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub tokens: TokenStore,
    pub auth_secret: String,
    pub cors_host: String,
    pub redirect_uri: String,
    pub basepath: String,
    pub listen_addr: String,
    pub listen_port: u16,
}

pub const TOKEN_ENDPOINT: &str = "https://accounts.spotify.com/api/token";
pub const NOW_PLAYING_ENDPOINT: &str = "https://api.spotify.com/v1/me/player/currently-playing";
pub const TOP_ENDPOINT: &str = "https://api.spotify.com/v1/me/top";

impl Config {
    pub fn from_env() -> Self {
        let get = |k: &str| {
            env::var(k).unwrap_or_else(|_| {
                eprintln!("Missing environment variable: {k}");
                String::new()
            })
        };
        let addr = env::var("LISTEN_ADDR")
            .ok()
            .unwrap_or("127.0.0.1".to_string());
        let port = env::var("LISTEN_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        let base = env::var("BASE_PATH").unwrap_or("".to_string());
        // PUBLIC_URL (e.g. https://badge.example.com/spotfy-badges) overrides the default redirect base.
        let public_url =
            env::var("PUBLIC_URL").unwrap_or_else(|_| format!("https://{addr}:{port}{base}"));

        if !public_url.ends_with(&base) {
            eprintln!(
                "Warning: PUBLIC_URL '{public_url}' does not end with BASE_PATH '{base}', make sure that this is intended."
            )
        }

        if env::var("AUTH_SECRET").unwrap_or_default().is_empty() {
            eprintln!("AUTH_SECRET not set: /api/auth is disabled.");
        }
        let token_file = env::var("TOKEN_FILE")
            .ok()
            .or_else(|| {
                env::var("STATE_DIRECTORY")
                    .ok()
                    .map(|d| format!("{}/refresh_token", d.trim_end_matches('/')))
            })
            .map(PathBuf::from);

        Self {
            client_id: get("SPOTIFY_CLIENT_ID"),
            client_secret: get("SPOTIFY_CLIENT_SECRET"),
            tokens: TokenStore::new(
                env::var("SPOTIFY_REFRESH_TOKEN").unwrap_or_default(),
                token_file,
            ),
            auth_secret: env::var("AUTH_SECRET").unwrap_or_default(),
            cors_host: env::var("API_CORS_HOST").unwrap_or_else(|_| "*".into()),
            redirect_uri: format!("{}/api/auth", public_url.trim_end_matches('/')),
            listen_addr: addr,
            listen_port: port,
            basepath: base.trim_end_matches('/').to_string(),
        }
    }
}
