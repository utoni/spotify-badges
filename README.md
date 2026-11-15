# spotify-badges (Rust)

Inspired by [akellbl4/spotify-badge](https://github.com/akellbl4/spotify-badge) but written in Rust, usable within self-hosted environments.

## Endpoints
- `GET /api/now-playing.svg`: SVG badge (cover embedded as base64)
- `GET /api/now-playing` (also `/now-playing`): JSON track info
- `GET /api/top-artists.svg`: SVG badge with your top 5 artists
- `GET /api/top-tracks.svg`: SVG badge with your top 5 most played tracks
  (both accept `?time_range=short_term|medium_term|long_term`, default `medium_term`)
- `GET /api/auth?secret=$AUTH_SECRET`: Spotify OAuth flow; the new refresh token is activated and persisted, no restart needed
- `GET /`: redirects to the project page

## Config (env vars, see `.env.example`)
- `SPOTIFY_CLIENT_ID` (default: empty)
- `SPOTIFY_CLIENT_SECRET` (default: empty)
- `SPOTIFY_REFRESH_TOKEN` (default: empty)
- `AUTH_SECRET` (default: empty)
- `TOKEN_FILE` (default: empty)
- `API_CORS_HOST` (default: `*`)
- `LISTEN_ADDR` (default: `127.0.0.1`)
- `LISTEN_PORT` (default: `3000`)
- `BASE_PATH` (default: empty)
- `PUBLIC_URL` (base URL for the OAuth redirect, default `https://{LISTEN_ADDR}:{LISTEN_PORT}{BASE_PATH}`; register `{PUBLIC_URL}/api/auth` in your Spotify app)

## Run
```
cargo run --release
```
