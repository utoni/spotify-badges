mod config;
mod spotify;
mod view;

use smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use std::sync::Arc;

struct Response {
    status: &'static str,
    headers: Vec<(&'static str, String)>,
    body: String,
}

impl Response {
    fn new(status: &'static str, content_type: &str, body: String) -> Self {
        let headers = vec![("Content-Type", content_type.to_string())];
        Self { status, headers, body }
    }

    fn redirect(location: &str) -> Self {
        let mut r = Self::new("307 Temporary Redirect", "text/plain", String::new());
        r.headers.push(("Location", location.to_string()));
        r
    }
}

fn main() {
    let cfg = Arc::new(config::Config::from_env());
    smol::block_on(async {
        let addr = format!("0.0.0.0:{}", cfg.port);
        let listener = TcpListener::bind(&addr).await.expect("bind failed");
        println!("Listening on http://{addr}");
        loop {
            let Ok((stream, _)) = listener.accept().await else { continue };
            let cfg = cfg.clone();
            smol::spawn(async move {
                if let Err(e) = handle(stream, &cfg).await {
                    eprintln!("connection error: {e}");
                }
            })
            .detach();
        }
    });
}

async fn read_request_line(stream: &mut TcpStream) -> std::io::Result<String> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") && buf.len() < 16 * 1024 {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&buf);
    Ok(text.lines().next().unwrap_or_default().to_string())
}

async fn handle(mut stream: TcpStream, cfg: &Arc<config::Config>) -> std::io::Result<()> {
    let line = read_request_line(&mut stream).await?;
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/"));

    let res = if method != "GET" {
        Response::new("405 Method Not Allowed", "text/plain", "Method Not Allowed".into())
    } else {
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        route(cfg, path, query).await
    };

    let mut out = format!("HTTP/1.1 {}\r\n", res.status);
    for (k, v) in &res.headers {
        out.push_str(&format!("{k}: {v}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\nConnection: close\r\n\r\n", res.body.len()));
    stream.write_all(out.as_bytes()).await?;
    stream.write_all(res.body.as_bytes()).await?;
    stream.flush().await
}

async fn route(cfg: &Arc<config::Config>, path: &str, query: &str) -> Response {
    match path {
        "/" => Response::redirect("https://github.com/akellbl4/spotify-badge"),
        "/api/auth" => {
            let code = form_urlencoded::parse(query.as_bytes())
                .find(|(k, v)| k == "code" && !v.is_empty())
                .map(|(_, v)| v.into_owned());
            match code {
                None => Response::redirect(&spotify::authorize_url(cfg)),
                Some(code) => {
                    let c = cfg.clone();
                    let token = smol::unblock(move || spotify::exchange_code(&c, &code)).await;
                    Response::new("200 OK", "text/html; charset=utf-8", view::refresh_token_page(token.as_deref()))
                }
            }
        }
        "/api/now-playing" | "/now-playing" => {
            let c = cfg.clone();
            let track = smol::unblock(move || spotify::get_now_playing(&c, false)).await;
            let mut r = Response::new("200 OK", "application/json", serde_json::to_string(&track).unwrap_or_default());
            if !cfg.cors_host.is_empty() {
                r.headers.push(("Access-Control-Allow-Origin", cfg.cors_host.clone()));
                r.headers.push(("Access-Control-Allow-Methods", "GET".into()));
            }
            r.headers.push(("Cache-Control", "s-maxage=1".into()));
            r
        }
        "/api/now-playing.svg" => {
            let c = cfg.clone();
            let track = smol::unblock(move || spotify::get_now_playing(&c, true)).await;
            let mut r = Response::new("200 OK", "image/svg+xml", view::player(&track));
            r.headers.push(("Cache-Control", "s-maxage=1".into()));
            r
        }
        _ => Response::new("404 Not Found", "text/plain", "Not Found".into()),
    }
}
