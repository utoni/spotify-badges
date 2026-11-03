use std::env;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub cors_host: String,
    pub redirect_uri: String,
    pub port: u16,
}

pub const TOKEN_ENDPOINT: &str = "https://accounts.spotify.com/api/token";
pub const NOW_PLAYING_ENDPOINT: &str = "https://api.spotify.com/v1/me/player/currently-playing";

impl Config {
    pub fn from_env() -> Self {
        let get = |k: &str| {
            env::var(k).unwrap_or_else(|_| {
                eprintln!("Missing environment variable: {k}");
                String::new()
            })
        };
        let port = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
        // PUBLIC_URL (e.g. https://badge.example.com) overrides the default redirect base.
        let base = env::var("PUBLIC_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));
        Self {
            client_id: get("SPOTIFY_CLIENT_ID"),
            client_secret: get("SPOTIFY_CLIENT_SECRET"),
            refresh_token: get("SPOTIFY_REFRESH_TOKEN"),
            cors_host: env::var("API_CORS_HOST").unwrap_or_else(|_| "*".into()),
            redirect_uri: format!("{}/api/auth", base.trim_end_matches('/')),
            port,
        }
    }
}
