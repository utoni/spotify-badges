use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const STATE_TTL: Duration = Duration::from_secs(600);
const MAX_PENDING_STATES: usize = 32;

pub struct TokenStore {
    refresh: Mutex<String>,
    access: Mutex<Option<(String, Instant)>>,
    states: Mutex<Vec<(String, Instant)>>,
    file: Option<PathBuf>,
}

impl TokenStore {
    pub fn new(env_token: String, file: Option<PathBuf>) -> Self {
        let stored = file
            .as_ref()
            .and_then(|f| fs::read_to_string(f).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(_) = stored {
            println!("Using refresh token from file {:?}", file);
        }
        Self {
            refresh: Mutex::new(stored.unwrap_or(env_token)),
            access: Mutex::new(None),
            states: Mutex::new(Vec::new()),
            file,
        }
    }

    pub fn refresh_token(&self) -> String {
        self.refresh.lock().unwrap().clone()
    }

    pub fn set_refresh_token(&self, token: &str) {
        *self.refresh.lock().unwrap() = token.to_string();
        *self.access.lock().unwrap() = None;
        let Some(path) = &self.file else { return };
        if let Err(e) = write_private(path, token) {
            eprintln!("Cannot persist refresh token to {}: {e}", path.display());
        } else {
            println!("Sucessfully wrote refresh token to {}", path.display());
        }
    }

    pub fn cached_access_token(&self) -> Option<String> {
        self.access
            .lock()
            .unwrap()
            .as_ref()
            .filter(|(_, exp)| *exp > Instant::now())
            .map(|(t, _)| t.clone())
    }

    pub fn cache_access_token(&self, token: &str, expires_in: u64) {
        let ttl = Duration::from_secs(expires_in.saturating_sub(60));
        *self.access.lock().unwrap() = Some((token.to_string(), Instant::now() + ttl));
    }

    pub fn invalidate_access_token(&self) {
        *self.access.lock().unwrap() = None;
    }

    pub fn new_state(&self) -> Option<String> {
        let state = random_hex()?;
        let now = Instant::now();
        let mut states = self.states.lock().unwrap();
        states.retain(|(_, t)| now.duration_since(*t) < STATE_TTL);
        if states.len() >= MAX_PENDING_STATES {
            states.remove(0);
        }
        states.push((state.clone(), now));
        Some(state)
    }

    pub fn consume_state(&self, state: &str) -> bool {
        let now = Instant::now();
        let mut states = self.states.lock().unwrap();
        states.retain(|(_, t)| now.duration_since(*t) < STATE_TTL);
        match states.iter().position(|(s, _)| ct_eq(s, state)) {
            Some(i) => {
                states.remove(i);
                true
            }
            None => false,
        }
    }
}

pub fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() != b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

fn random_hex() -> Option<String> {
    let mut buf = [0u8; 24];
    fs::File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut buf)
        .ok()?;
    Some(buf.iter().map(|b| format!("{b:02x}")).collect())
}

fn write_private(path: &PathBuf, content: &str) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let tmp = path.with_extension("tmp");
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp)?;
    f.write_all(content.as_bytes())?;
    f.sync_all()?;
    fs::rename(tmp, path)
}
