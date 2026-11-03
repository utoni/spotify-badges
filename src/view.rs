use crate::spotify::TrackInfo;

const WIDTH: u32 = 540;
const HEIGHT: u32 = 52;

const EQ_FRAMES: [u8; 26] = [
    90, 77, 73, 52, 38, 52, 73, 13, 73, 90, 38, 73, 90, 52, 20, 52, 90, 52, 90, 38, 90, 69, 90, 38, 90, 90,
];

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn equalizer() -> String {
    let frames: String = EQ_FRAMES
        .iter()
        .enumerate()
        .map(|(i, y)| format!("{}%{{transform:translateY({y}%);}}", i * 4))
        .collect();
    format!(
        r#"<style>
.equalizer {{ overflow: hidden; position: relative; width: 20px; height: 20px; display: flex; margin-left: auto; }}
.equalizer-line {{ display: block; content: ''; height: 100%; width: 3px; transition: .8s; animation: equalize 4s linear infinite; background: #EAEAEA; }}
.equalizer-line + .equalizer-line {{ margin-left: 2px; }}
.equalizer-line:nth-child(1) {{ animation-delay: -1.9s; }}
.equalizer-line:nth-child(2) {{ animation-delay: -2.9s; }}
.equalizer-line:nth-child(3) {{ animation-delay: -3.9s; }}
@keyframes equalize {{ {frames}100%{{transform:translateY(90%);}} }}
</style>
<div class="equalizer"><div class="equalizer-line"></div><div class="equalizer-line"></div><div class="equalizer-line"></div></div>"#
    )
}

const PAUSE: &str = r#"<style>
.pause { display: flex; justify-content: center; align-items: center; box-sizing: border-box; border: 1px solid #ccc; width: 22px; height: 22px; margin: -1px; border-radius: 50%; }
.pause:before, .pause:after { display: block; height: 9px; width: 3px; margin: 1px; content: ''; background: #ccc; }
</style>
<div class="pause"></div>"#;

const REFRESH_ICON: &str = r##"<svg width="20" height="20" viewBox="0 0 50 50" xmlns="http://www.w3.org/2000/svg"><path fill="none" stroke="#000" stroke-linecap="round" stroke-miterlimit="10" stroke-width="2" d="M40 7v9l-9-.001"/><path d="M41.999 25c0 9.39-7.61 17-17 17s-17-7.61-17-17 7.61-17 17-17c5.011 0 9.516 2.167 12.627 5.616a17.103 17.103 0 011.683 2.203" fill="none" stroke="#000" stroke-linecap="round" stroke-miterlimit="10" stroke-width="2"/></svg>"##;

const SPOTIFY_LOGO: &str = r##"<svg class="spotify-logo" width="32" height="32" viewBox="0 0 168 168" xmlns="http://www.w3.org/2000/svg"><path fill="#1ED760" d="M83.996.277C37.747.277.253 37.77.253 84.019c0 46.251 37.494 83.741 83.743 83.741 46.254 0 83.744-37.49 83.744-83.741 0-46.246-37.49-83.738-83.745-83.738l.001-.004zm38.404 120.78a5.217 5.217 0 01-7.18 1.73c-19.662-12.01-44.414-14.73-73.564-8.07a5.222 5.222 0 01-6.249-3.93 5.213 5.213 0 013.926-6.25c31.9-7.291 59.263-4.15 81.337 9.34 2.46 1.51 3.24 4.72 1.73 7.18zm10.25-22.805c-1.89 3.075-5.91 4.045-8.98 2.155-22.51-13.839-56.823-17.846-83.448-9.764-3.453 1.043-7.1-.903-8.148-4.35a6.538 6.538 0 014.354-8.143c30.413-9.228 68.222-4.758 94.072 11.127 3.07 1.89 4.04 5.91 2.15 8.976v-.001zm.88-23.744c-26.99-16.031-71.52-17.505-97.289-9.684-4.138 1.255-8.514-1.081-9.768-5.219a7.835 7.835 0 015.221-9.771c29.581-8.98 78.756-7.245 109.83 11.202a7.823 7.823 0 012.74 10.733c-2.2 3.722-7.02 4.949-10.73 2.739z"/></svg>"##;

fn cover(src: &str, playing: bool) -> String {
    let shadow = if playing { "0px 1px 4px rgb(0 0 0 / 20%)" } else { "none" };
    format!(
        r#"<style>.cover {{ height: 100%; width: 100%; overflow: hidden; border-radius: 2px; box-shadow: {shadow}; }}</style>
<img class="cover" src="{}"/>"#,
        escape(src)
    )
}

fn progress(playing: bool, progress: u64, duration: u64) -> String {
    let color = if playing { "#10a242" } else { "rgba(0, 0, 0, 0.1)" };
    let state = if playing { "running" } else { "paused" };
    format!(
        r#"<style>
.progress {{ position: relative; width: 100%; height: 3px; background: #EAEAEA; overflow: hidden; border-radius: 3px; }}
.progress:before {{ position: absolute; top: 0; left: 0; content: ''; width: 100%; height: 100%; background-color: {color}; transform: translateX(-100%); animation: progress {duration}ms linear; animation-delay: -{progress}ms; animation-play-state: {state}; border-radius: 3px; }}
@keyframes progress {{ 0% {{ transform: translateX(-100%); }} 100% {{ transform: translateX(0); }} }}
</style>
<div class="progress"></div>"#
    )
}

fn track_info(track: &str, artist: &str) -> String {
    let artist = if artist.is_empty() {
        String::new()
    } else {
        format!(r#"<span class="track-info-artist">{}</span>"#, escape(&format!(" – {artist}")))
    };
    format!(
        r#"<style>
.track-info {{ position: relative; width: 100%; white-space: nowrap; }}
.track-info:after {{ position: absolute; top: 0; right: 0; display: block; content: ''; height: 100%; width: 40px; background: linear-gradient(90deg, rgba(255, 255, 255, 0), rgba(255, 255, 255, 1)); }}
.track-info-artist {{ color: #707070; }}
</style>
<div class="track-info"><span class="track-info-composition">{}</span>{artist}</div>"#,
        escape(track)
    )
}

pub fn player(p: &TrackInfo) -> String {
    let has_track = p.artist.is_some() && p.track.is_some();
    let duration = p.duration.unwrap_or(0);
    let prog = p.progress.unwrap_or(0);
    let delay = if p.is_playing { duration.saturating_sub(prog) } else { 0 };
    let state = if p.is_playing { "running" } else { "paused" };

    let image = match p.cover_url.as_deref().filter(|c| !c.is_empty()) {
        Some(c) => cover(c, p.is_playing),
        None => SPOTIFY_LOGO.to_string(),
    };
    let info = track_info(
        p.track.as_deref().unwrap_or("Not playing"),
        p.artist.as_deref().unwrap_or("Spotify"),
    );
    let status = match (has_track, p.is_playing) {
        (false, _) => String::new(),
        (true, true) => format!(
            r#"<div class="frame-body-status frame-body-status_playing"><div class="frame-body-status-playing">{REFRESH_ICON}{}</div></div>"#,
            equalizer()
        ),
        (true, false) => format!(r#"<div class="frame-body-status">{PAUSE}</div>"#),
    };
    let bar = if p.is_playing {
        format!(r#"<div class="frame-progress">{}</div>"#, progress(true, prog, duration))
    } else {
        String::new()
    };

    format!(
        r#"<svg fill="none" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" xmlns="http://www.w3.org/2000/svg">
<foreignObject width="{WIDTH}" height="{HEIGHT}">
<style>
.frame {{ --delay: {delay}ms; display: flex; box-sizing: border-box; width: 100%; padding: 6px 12px; align-items: center; border-radius: 6px; background-color: #fff; font-family: -apple-system,BlinkMacSystemFont,Segoe UI,Helvetica,Arial,sans-serif,Apple Color Emoji,Segoe UI Emoji; font-size: 16px; }}
.frame-image {{ display: flex; justify-content: center; align-items: center; flex-shrink: 0; width: 40px; height: 40px; margin-right: 12px; }}
.frame-body {{ display: flex; width: 100%; flex-wrap: wrap; align-items: center; }}
.frame-body-content {{ display: flex; max-width: calc(100% - 32px); height: 20px; flex-grow: 1; overflow: hidden; align-items: center; animation: shift-5 0.2s; animation-fill-mode: forwards; animation-delay: var(--delay); animation-play-state: {state}; }}
.frame-body-status {{ margin-left: 12px; width: 20px; height: 20px; flex-shrink: 0; }}
.frame-body-status_playing {{ overflow: hidden; animation: shift-5 0.2s; animation-fill-mode: forwards; animation-delay: var(--delay); }}
.frame-body-status-playing {{ transform: translateY(-50%); animation: shift-half 0.2s; animation-fill-mode: forwards; animation-delay: var(--delay); }}
.frame-progress {{ width: 100%; margin-top: 8px; animation: disappear 0.2s; animation-delay: var(--delay); animation-fill-mode: forwards; }}
@keyframes disappear {{ 0% {{ opacity: 1; transform: translateY(0); }} 100% {{ opacity: 0; transform: translateY(100%); }} }}
@keyframes shift-5 {{ 0% {{ transform: translateY(0); }} 100% {{ transform: translateY(5px); }} }}
@keyframes shift-half {{ 0% {{ transform: translateY(-50%); }} 100% {{ transform: translateY(0); }} }}
</style>
<div class="frame" xmlns="http://www.w3.org/1999/xhtml">
<div class="frame-image">{image}</div>
<div class="frame-body"><div class="frame-body-content">{info}</div>{status}{bar}</div>
</div>
</foreignObject>
</svg>"#
    )
}

pub fn refresh_token_page(token: Option<&str>) -> String {
    let body = match token {
        Some(t) => format!("<div><h3>Refresh token:</h3><pre>{}</pre></div>", escape(t)),
        None => "<div>Invalid authorization code</div>".to_string(),
    };
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>
body {{ font-size: 16px; font-weight: 400; line-height: 1.5; font-family: -apple-system,BlinkMacSystemFont,Segoe UI,Helvetica,Arial,sans-serif,Apple Color Emoji,Segoe UI Emoji; color: #333; }}
pre {{ max-width: 600px; padding: 16px; border-radius: 4px; background-color: #f5f5f5; overflow-wrap: break-word; white-space: pre-wrap; }}
</style></head><body>{body}</body></html>"#
    )
}
