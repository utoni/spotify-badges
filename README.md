# spotify-badge (Rust)

Pure-Rust port of [akellbl4/spotify-badge](https://github.com/akellbl4/spotify-badge): a "now playing" SVG badge and JSON API. No npm/Node.

## Endpoints
- `GET /api/now-playing.svg` – SVG badge (cover embedded as base64)
- `GET /api/now-playing` (also `/now-playing`) – JSON track info
- `GET /api/auth` – Spotify OAuth flow; shows your refresh token
- `GET /` – redirects to the project page

## Config (env vars, see `.env.example`)
`SPOTIFY_CLIENT_ID`, `SPOTIFY_CLIENT_SECRET`, `SPOTIFY_REFRESH_TOKEN`, `API_CORS_HOST` (default `*`),
`PORT` (default 3000), `PUBLIC_URL` (base URL for the OAuth redirect, default `http://localhost:$PORT`;
register `$PUBLIC_URL/api/auth` in your Spotify app).

## Run
```
cargo run --release
```
