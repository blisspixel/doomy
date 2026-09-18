//! Developer-only audio generation for fragr through the ElevenLabs HTTP API.
//!
//! Everything that can be checked without a network lives here: request shapes,
//! response handling, WAV wrapping, batch specs, the provenance manifest, and the
//! command runner. The binary in `main.rs` only parses arguments and supplies the
//! real HTTP transport. Players never run this; it exists so developers can
//! produce sound effects and music beds that are committed as ordinary assets.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default API host. Regional hosts exist but are not needed for asset generation.
pub const DEFAULT_BASE_URL: &str = "https://api.elevenlabs.io";
/// Environment variable both official SDKs read for the key.
pub const API_KEY_ENV: &str = "ELEVENLABS_API_KEY";
/// The only sound effects model the API accepts at the time of writing.
pub const SFX_MODEL: &str = "eleven_text_to_sound_v2";
/// Newest music model accepted everywhere on the music endpoints.
pub const DEFAULT_MUSIC_MODEL: &str = "music_v2_5";
/// 24 kHz PCM is available on every paid tier; 44.1 kHz PCM needs a higher tier.
pub const DEFAULT_SFX_FORMAT: &str = "pcm_24000";
/// MP3 at 44.1 kHz and 128 kbps is available on every tier and streams in Godot.
pub const DEFAULT_MUSIC_FORMAT: &str = "mp3_44100_128";
/// Default output directory relative to the repository root.
pub const DEFAULT_OUT_DIR: &str = "client/assets/audio";
/// Provenance manifest written next to generated files.
pub const MANIFEST_FILE: &str = "audiogen-manifest.json";

const MUSIC_MODELS: [&str; 3] = ["music_v1", "music_v2", "music_v2_5"];
const SFX_SECONDS_MIN: f64 = 0.5;
const SFX_SECONDS_MAX: f64 = 30.0;
const MUSIC_LENGTH_MIN_MS: u32 = 3_000;
const MUSIC_LENGTH_MAX_MS: u32 = 600_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// One HTTP call, independent of the client library.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    MissingApiKey,
    InvalidArgument(String),
    Transport(String),
    Api { status: u16, message: String },
    Io(std::io::Error),
    Spec(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingApiKey => write!(
                f,
                "no API key: set {API_KEY_ENV} or pass --api-key-file (see tools/audiogen/README.md)"
            ),
            Error::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
            Error::Transport(msg) => write!(f, "transport error: {msg}"),
            Error::Api { status, message } => write!(f, "API error {status}: {message}"),
            Error::Io(err) => write!(f, "io error: {err}"),
            Error::Spec(msg) => write!(f, "spec error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

/// The HTTP seam. The binary supplies a real client; tests supply fakes.
pub trait Transport {
    fn send(&self, api_key: &str, request: &Request) -> Result<Response, Error>;
}

/// How a requested `output_format` lands on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encoding {
    /// Raw signed 16-bit little-endian samples; wrapped in a WAV header on disk.
    Pcm {
        sample_rate: u32,
    },
    Mp3,
    Opus,
}

impl Encoding {
    pub fn extension(&self) -> &'static str {
        match self {
            Encoding::Pcm { .. } => "wav",
            Encoding::Mp3 => "mp3",
            Encoding::Opus => "opus",
        }
    }
}

/// Parse an ElevenLabs `output_format` value such as `pcm_24000` or `mp3_44100_128`.
pub fn parse_format(format: &str) -> Result<Encoding, Error> {
    let mut parts = format.split('_');
    let codec = parts.next().unwrap_or_default();
    let rate = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| {
            Error::InvalidArgument(format!("output format '{format}' has no sample rate"))
        })?;
    match codec {
        "pcm" => Ok(Encoding::Pcm { sample_rate: rate }),
        "mp3" => Ok(Encoding::Mp3),
        "opus" => Ok(Encoding::Opus),
        _ => Err(Error::InvalidArgument(format!(
            "output format '{format}' is not supported; use pcm_*, mp3_* or opus_*"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SfxParams {
    pub prompt: String,
    pub seconds: Option<f64>,
    pub influence: Option<f64>,
    pub looping: bool,
    pub format: String,
}

impl SfxParams {
    pub fn validate(&self) -> Result<(), Error> {
        if self.prompt.trim().is_empty() {
            return Err(Error::InvalidArgument("sfx prompt is empty".to_string()));
        }
        if let Some(seconds) = self.seconds {
            if !(SFX_SECONDS_MIN..=SFX_SECONDS_MAX).contains(&seconds) {
                return Err(Error::InvalidArgument(format!(
                    "sfx seconds {seconds} is outside {SFX_SECONDS_MIN}..={SFX_SECONDS_MAX}"
                )));
            }
        }
        if let Some(influence) = self.influence {
            if !(0.0..=1.0).contains(&influence) {
                return Err(Error::InvalidArgument(format!(
                    "sfx influence {influence} is outside 0..=1"
                )));
            }
        }
        parse_format(&self.format).map(|_| ())
    }
}

/// `POST /v1/sound-generation` body and query.
pub fn sfx_request(params: &SfxParams) -> Result<Request, Error> {
    params.validate()?;
    let mut body = serde_json::Map::new();
    body.insert("text".into(), params.prompt.clone().into());
    if let Some(seconds) = params.seconds {
        body.insert("duration_seconds".into(), seconds.into());
    }
    if let Some(influence) = params.influence {
        body.insert("prompt_influence".into(), influence.into());
    }
    body.insert("loop".into(), params.looping.into());
    body.insert("model_id".into(), SFX_MODEL.into());
    Ok(Request {
        method: Method::Post,
        path: "/v1/sound-generation".to_string(),
        query: vec![("output_format".to_string(), params.format.clone())],
        body: Some(serde_json::Value::Object(body)),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicParams {
    pub prompt: String,
    pub length_ms: Option<u32>,
    pub model: String,
    pub instrumental: bool,
    pub format: String,
}

impl MusicParams {
    pub fn validate(&self) -> Result<(), Error> {
        if self.prompt.trim().is_empty() {
            return Err(Error::InvalidArgument("music prompt is empty".to_string()));
        }
        if let Some(length_ms) = self.length_ms {
            if !(MUSIC_LENGTH_MIN_MS..=MUSIC_LENGTH_MAX_MS).contains(&length_ms) {
                return Err(Error::InvalidArgument(format!(
                    "music length {length_ms} ms is outside {MUSIC_LENGTH_MIN_MS}..={MUSIC_LENGTH_MAX_MS}"
                )));
            }
        }
        if !MUSIC_MODELS.contains(&self.model.as_str()) {
            return Err(Error::InvalidArgument(format!(
                "music model '{}' is not one of {}",
                self.model,
                MUSIC_MODELS.join(", ")
            )));
        }
        parse_format(&self.format).map(|_| ())
    }
}

/// `POST /v1/music` body and query. The plain endpoint returns raw audio bytes.
pub fn music_request(params: &MusicParams) -> Result<Request, Error> {
    params.validate()?;
    let mut body = serde_json::Map::new();
    body.insert("prompt".into(), params.prompt.clone().into());
    if let Some(length_ms) = params.length_ms {
        body.insert("music_length_ms".into(), length_ms.into());
    }
    body.insert("model_id".into(), params.model.clone().into());
    body.insert("force_instrumental".into(), params.instrumental.into());
    Ok(Request {
        method: Method::Post,
        path: "/v1/music".to_string(),
        query: vec![("output_format".to_string(), params.format.clone())],
        body: Some(serde_json::Value::Object(body)),
    })
}

/// `GET /v1/user/subscription`, the quota check.
pub fn quota_request() -> Request {
    Request {
        method: Method::Get,
        path: "/v1/user/subscription".to_string(),
        query: Vec::new(),
        body: None,
    }
}

/// Subset of the subscription response we report. Credits are exposed under
/// the historical "character" field names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Subscription {
    pub tier: Option<String>,
    pub status: Option<String>,
    pub character_count: Option<u64>,
    pub character_limit: Option<u64>,
    pub next_character_count_reset_unix: Option<u64>,
}

pub fn parse_subscription(body: &[u8]) -> Result<Subscription, Error> {
    serde_json::from_slice(body).map_err(|err| Error::Api {
        status: 200,
        message: format!("unreadable subscription body: {err}"),
    })
}

pub fn describe_subscription(sub: &Subscription) -> String {
    let tier = sub.tier.as_deref().unwrap_or("unknown");
    let status = sub.status.as_deref().unwrap_or("unknown");
    let used = sub.character_count.unwrap_or(0);
    let limit = sub.character_limit.unwrap_or(0);
    let remaining = limit.saturating_sub(used);
    let reset = sub
        .next_character_count_reset_unix
        .map(|unix| format!("{unix} (unix)"))
        .unwrap_or_else(|| "unknown".to_string());
    format!("tier: {tier}\nstatus: {status}\ncredits used: {used} of {limit} (remaining {remaining})\nresets at: {reset}")
}

/// Turn an error body into one readable line. Handles the structured
/// `{"detail": {...}}` shape, the validation `{"detail": [...]}` shape, and plain text.
pub fn api_error_message(status: u16, body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(detail) = value.get("detail") {
            if let Some(object) = detail.as_object() {
                let code = object
                    .get("code")
                    .or_else(|| object.get("status"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("error");
                let message = object.get("message").and_then(|v| v.as_str()).unwrap_or("");
                return format!("{code}: {message}")
                    .trim_end_matches(": ")
                    .to_string();
            }
            if let Some(items) = detail.as_array() {
                let messages: Vec<String> = items
                    .iter()
                    .map(|item| {
                        let field = item
                            .get("loc")
                            .and_then(|loc| loc.as_array())
                            .map(|loc| {
                                loc.iter()
                                    .filter_map(|part| part.as_str().map(str::to_string))
                                    .collect::<Vec<_>>()
                                    .join(".")
                            })
                            .unwrap_or_default();
                        let msg = item
                            .get("msg")
                            .and_then(|v| v.as_str())
                            .unwrap_or("invalid");
                        if field.is_empty() {
                            msg.to_string()
                        } else {
                            format!("{field}: {msg}")
                        }
                    })
                    .collect();
                return format!("validation failed: {}", messages.join("; "));
            }
            if let Some(message) = detail.as_str() {
                return message.to_string();
            }
        }
    }
    let snippet: String = text.chars().take(200).collect();
    if snippet.trim().is_empty() {
        format!("HTTP {status} with empty body")
    } else {
        snippet
    }
}

/// Rate limiting and server-side failures are worth another attempt.
pub fn should_retry(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub attempts: u32,
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            attempts: 4,
            base_delay: Duration::from_secs(2),
        }
    }
}

/// Send with exponential backoff. `sleep` is injected so tests do not wait.
pub fn send_with_retry(
    transport: &dyn Transport,
    api_key: &str,
    request: &Request,
    policy: &RetryPolicy,
    sleep: &mut dyn FnMut(Duration),
) -> Result<Response, Error> {
    let attempts = policy.attempts.max(1);
    let mut attempt = 0;
    loop {
        let response = transport.send(api_key, request)?;
        if (200..300).contains(&response.status) {
            return Ok(response);
        }
        attempt += 1;
        if should_retry(response.status) && attempt < attempts {
            let delay = policy.base_delay * 2u32.saturating_pow(attempt - 1);
            tracing::warn!(
                status = response.status,
                attempt,
                "retrying after {:?}",
                delay
            );
            sleep(delay);
            continue;
        }
        return Err(Error::Api {
            status: response.status,
            message: api_error_message(response.status, &response.body),
        });
    }
}

/// Wrap raw S16LE samples in a canonical 44-byte WAV header.
pub fn wav_from_pcm_s16le(pcm: &[u8], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_len = u32::try_from(pcm.len()).unwrap_or(u32::MAX);
    let block_align = channels * 2;
    let byte_rate = sample_rate * u32::from(block_align);
    let mut out = Vec::with_capacity(44 + pcm.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(pcm);
    out
}

/// The API does not document PCM channel count. When the caller asked for a
/// duration, pick the channel count whose implied duration is closest to it.
/// Without a duration, assume mono.
pub fn infer_channels(byte_len: usize, sample_rate: u32, expected_seconds: Option<f64>) -> u16 {
    let Some(expected) = expected_seconds else {
        return 1;
    };
    if sample_rate == 0 || byte_len == 0 {
        return 1;
    }
    let mono_seconds = byte_len as f64 / (f64::from(sample_rate) * 2.0);
    let stereo_seconds = mono_seconds / 2.0;
    if (stereo_seconds - expected).abs() < (mono_seconds - expected).abs() {
        2
    } else {
        1
    }
}

/// Convert a response body into the bytes that go on disk.
pub fn encode_output(
    body: Vec<u8>,
    encoding: &Encoding,
    expected_seconds: Option<f64>,
) -> (Vec<u8>, u16) {
    match encoding {
        Encoding::Pcm { sample_rate } => {
            let channels = infer_channels(body.len(), *sample_rate, expected_seconds);
            (wav_from_pcm_s16le(&body, *sample_rate, channels), channels)
        }
        Encoding::Mp3 | Encoding::Opus => (body, 0),
    }
}

/// One generation job, the unit the runner works on.
#[derive(Debug, Clone, PartialEq)]
pub enum Job {
    Sfx(SfxParams),
    Music(MusicParams),
}

impl Job {
    pub fn request(&self) -> Result<Request, Error> {
        match self {
            Job::Sfx(params) => sfx_request(params),
            Job::Music(params) => music_request(params),
        }
    }

    pub fn encoding(&self) -> Result<Encoding, Error> {
        parse_format(self.format())
    }

    pub fn format(&self) -> &str {
        match self {
            Job::Sfx(params) => &params.format,
            Job::Music(params) => &params.format,
        }
    }

    pub fn prompt(&self) -> &str {
        match self {
            Job::Sfx(params) => &params.prompt,
            Job::Music(params) => &params.prompt,
        }
    }

    pub fn model(&self) -> &str {
        match self {
            Job::Sfx(_) => SFX_MODEL,
            Job::Music(params) => &params.model,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Job::Sfx(_) => "sfx",
            Job::Music(_) => "music",
        }
    }

    pub fn expected_seconds(&self) -> Option<f64> {
        match self {
            Job::Sfx(params) => params.seconds,
            Job::Music(params) => params.length_ms.map(|ms| f64::from(ms) / 1000.0),
        }
    }
}

/// Batch spec file: a list of named jobs plus an optional output directory.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub out_dir: Option<String>,
    pub items: Vec<SpecItem>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpecItem {
    Sfx {
        name: String,
        prompt: String,
        seconds: Option<f64>,
        influence: Option<f64>,
        #[serde(default, rename = "loop")]
        looping: bool,
        format: Option<String>,
    },
    Music {
        name: String,
        prompt: String,
        length_ms: Option<u32>,
        model: Option<String>,
        #[serde(default)]
        instrumental: bool,
        format: Option<String>,
    },
}

impl SpecItem {
    pub fn name(&self) -> &str {
        match self {
            SpecItem::Sfx { name, .. } | SpecItem::Music { name, .. } => name,
        }
    }

    pub fn into_job(self) -> Job {
        match self {
            SpecItem::Sfx {
                prompt,
                seconds,
                influence,
                looping,
                format,
                ..
            } => Job::Sfx(SfxParams {
                prompt,
                seconds,
                influence,
                looping,
                format: format.unwrap_or_else(|| DEFAULT_SFX_FORMAT.to_string()),
            }),
            SpecItem::Music {
                prompt,
                length_ms,
                model,
                instrumental,
                format,
                ..
            } => Job::Music(MusicParams {
                prompt,
                length_ms,
                model: model.unwrap_or_else(|| DEFAULT_MUSIC_MODEL.to_string()),
                instrumental,
                format: format.unwrap_or_else(|| DEFAULT_MUSIC_FORMAT.to_string()),
            }),
        }
    }
}

pub fn parse_spec(json: &str) -> Result<Spec, Error> {
    let spec: Spec = serde_json::from_str(json).map_err(|err| Error::Spec(err.to_string()))?;
    if spec.items.is_empty() {
        return Err(Error::Spec("spec has no items".to_string()));
    }
    let mut seen = std::collections::BTreeSet::new();
    for item in &spec.items {
        validate_name(item.name())?;
        if !seen.insert(item.name().to_string()) {
            return Err(Error::Spec(format!(
                "duplicate item name '{}'",
                item.name()
            )));
        }
    }
    Ok(spec)
}

/// Names are relative asset paths without an extension: `fire_rail` or `music/match_01`.
pub fn validate_name(name: &str) -> Result<(), Error> {
    let ok_chars = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '/'));
    if name.is_empty()
        || !ok_chars
        || name.starts_with('/')
        || name.ends_with('/')
        || name
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(Error::InvalidArgument(format!(
            "name '{name}' must be lowercase letters, digits, '_', '-', with '/' for folders, and no extension"
        )));
    }
    Ok(())
}

/// Provenance for every generated file, so anyone can regenerate or audit it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: BTreeMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub kind: String,
    pub file: String,
    pub prompt: String,
    pub model: String,
    pub format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_ms: Option<u32>,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub channels: u16,
    pub bytes: u64,
    pub generated_unix: u64,
}

pub fn load_manifest(dir: &Path) -> Result<Manifest, Error> {
    let path = dir.join(MANIFEST_FILE);
    if !path.exists() {
        return Ok(Manifest::default());
    }
    let text = fs::read_to_string(&path)?;
    serde_json::from_str(&text)
        .map_err(|err| Error::Spec(format!("unreadable manifest {}: {err}", path.display())))
}

pub fn save_manifest(dir: &Path, manifest: &Manifest) -> Result<(), Error> {
    fs::create_dir_all(dir)?;
    let text =
        serde_json::to_string_pretty(manifest).map_err(|err| Error::Spec(err.to_string()))?;
    fs::write(dir.join(MANIFEST_FILE), format!("{text}\n"))?;
    Ok(())
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Key names accepted in a `.env` file, compared case-insensitively.
const DOTENV_KEYS: [&str; 4] = [
    "ELEVENLABS_API_KEY",
    "ELEVENLABS",
    "ELEVENLABS_KEY",
    "XI_API_KEY",
];

/// Read an API key from a `.env` style file: `KEY=VALUE` lines, `#` comments,
/// optional `export` prefix, optional quotes. `Ok(None)` when the file or key is absent.
pub fn read_dotenv_key(path: &Path) -> Result<Option<String>, Error> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if !DOTENV_KEYS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(key))
        {
            continue;
        }
        let value = value.trim().trim_matches(|c| c == '"' || c == '\'').trim();
        if !value.is_empty() {
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

/// Resolve the API key: an explicit key file wins, then the environment. Whitespace is trimmed.
pub fn resolve_api_key(
    env_value: Option<String>,
    key_file: Option<&Path>,
) -> Result<String, Error> {
    let raw = match key_file {
        Some(path) => Some(fs::read_to_string(path)?),
        None => env_value,
    };
    match raw.map(|value| value.trim().to_string()) {
        Some(key) if !key.is_empty() => Ok(key),
        _ => Err(Error::MissingApiKey),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Written {
        path: PathBuf,
        bytes: usize,
        channels: u16,
    },
    Skipped(PathBuf),
    DryRun {
        path: PathBuf,
        request: Request,
    },
}

pub struct Generator<'a> {
    pub transport: &'a dyn Transport,
    pub api_key: String,
    pub out_dir: PathBuf,
    pub overwrite: bool,
    pub dry_run: bool,
    pub retry: RetryPolicy,
}

impl Generator<'_> {
    pub fn run(
        &self,
        name: &str,
        job: &Job,
        sleep: &mut dyn FnMut(Duration),
    ) -> Result<Outcome, Error> {
        validate_name(name)?;
        let encoding = job.encoding()?;
        let request = job.request()?;
        let path = self
            .out_dir
            .join(format!("{name}.{}", encoding.extension()));
        if self.dry_run {
            return Ok(Outcome::DryRun { path, request });
        }
        if path.exists() && !self.overwrite {
            return Ok(Outcome::Skipped(path));
        }
        let response =
            send_with_retry(self.transport, &self.api_key, &request, &self.retry, sleep)?;
        if response.body.is_empty() {
            return Err(Error::Api {
                status: response.status,
                message: "empty audio body".to_string(),
            });
        }
        let (bytes, channels) = encode_output(response.body, &encoding, job.expected_seconds());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &bytes)?;
        let mut manifest = load_manifest(&self.out_dir)?;
        manifest.entries.insert(
            name.to_string(),
            ManifestEntry {
                kind: job.kind().to_string(),
                file: format!("{name}.{}", encoding.extension()),
                prompt: job.prompt().to_string(),
                model: job.model().to_string(),
                format: job.format().to_string(),
                seconds: match job {
                    Job::Sfx(params) => params.seconds,
                    Job::Music(_) => None,
                },
                length_ms: match job {
                    Job::Music(params) => params.length_ms,
                    Job::Sfx(_) => None,
                },
                looping: matches!(job, Job::Sfx(params) if params.looping),
                channels,
                bytes: bytes.len() as u64,
                generated_unix: now_unix(),
            },
        );
        save_manifest(&self.out_dir, &manifest)?;
        Ok(Outcome::Written {
            path,
            bytes: bytes.len(),
            channels,
        })
    }
}

/// What the binary asks the library to do, after argument parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Generate { name: String, job: Job },
    Batch { spec: Spec, only: Option<String> },
    Quota,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RunOptions {
    pub out_dir: Option<PathBuf>,
    pub overwrite: bool,
    pub dry_run: bool,
}

fn report(out: &mut dyn Write, name: &str, outcome: &Outcome) -> Result<(), Error> {
    match outcome {
        Outcome::Written {
            path,
            bytes,
            channels,
        } => {
            let channels_note = if *channels > 0 {
                format!(", {channels} ch")
            } else {
                String::new()
            };
            writeln!(
                out,
                "wrote {name} -> {} ({bytes} bytes{channels_note})",
                path.display()
            )?;
        }
        Outcome::Skipped(path) => {
            writeln!(
                out,
                "skip {name}: {} exists (pass --overwrite to replace)",
                path.display()
            )?;
        }
        Outcome::DryRun { path, request } => {
            let body = request
                .body
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            writeln!(
                out,
                "dry-run {name} -> {}: POST {}?{} {body}",
                path.display(),
                request.path,
                request
                    .query
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("&")
            )?;
        }
    }
    Ok(())
}

/// Execute one command. Returns an error on the first failed job so partial
/// batches are visible instead of silently incomplete.
pub fn run_command(
    command: &Command,
    options: &RunOptions,
    transport: &dyn Transport,
    api_key: &str,
    out: &mut dyn Write,
    sleep: &mut dyn FnMut(Duration),
) -> Result<(), Error> {
    match command {
        Command::Quota => {
            let response = send_with_retry(
                transport,
                api_key,
                &quota_request(),
                &RetryPolicy::default(),
                sleep,
            )?;
            let subscription = parse_subscription(&response.body)?;
            writeln!(out, "{}", describe_subscription(&subscription))?;
            Ok(())
        }
        Command::Generate { name, job } => {
            let generator = Generator {
                transport,
                api_key: api_key.to_string(),
                out_dir: options
                    .out_dir
                    .clone()
                    .unwrap_or_else(|| PathBuf::from(DEFAULT_OUT_DIR)),
                overwrite: options.overwrite,
                dry_run: options.dry_run,
                retry: RetryPolicy::default(),
            };
            let outcome = generator.run(name, job, sleep)?;
            report(out, name, &outcome)
        }
        Command::Batch { spec, only } => {
            let out_dir = options
                .out_dir
                .clone()
                .or_else(|| spec.out_dir.as_ref().map(PathBuf::from))
                .unwrap_or_else(|| PathBuf::from(DEFAULT_OUT_DIR));
            let generator = Generator {
                transport,
                api_key: api_key.to_string(),
                out_dir,
                overwrite: options.overwrite,
                dry_run: options.dry_run,
                retry: RetryPolicy::default(),
            };
            let mut matched = 0;
            for item in &spec.items {
                if only.as_deref().is_some_and(|wanted| wanted != item.name()) {
                    continue;
                }
                matched += 1;
                let name = item.name().to_string();
                let job = item.clone().into_job();
                let outcome = generator.run(&name, &job, sleep)?;
                report(out, &name, &outcome)?;
            }
            if matched == 0 {
                return Err(Error::Spec(format!(
                    "no spec item named '{}'",
                    only.as_deref().unwrap_or_default()
                )));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeTransport {
        responses: RefCell<Vec<Response>>,
        seen: RefCell<Vec<(String, Request)>>,
    }

    impl FakeTransport {
        fn new(responses: Vec<Response>) -> Self {
            FakeTransport {
                responses: RefCell::new(responses),
                seen: RefCell::new(Vec::new()),
            }
        }

        fn ok(body: Vec<u8>) -> Self {
            FakeTransport::new(vec![Response { status: 200, body }])
        }
    }

    impl Transport for FakeTransport {
        fn send(&self, api_key: &str, request: &Request) -> Result<Response, Error> {
            self.seen
                .borrow_mut()
                .push((api_key.to_string(), request.clone()));
            let mut responses = self.responses.borrow_mut();
            if responses.is_empty() {
                return Err(Error::Transport("no scripted response".to_string()));
            }
            Ok(responses.remove(0))
        }
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fragr-audiogen-{tag}-{}", now_unix()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn no_sleep() -> impl FnMut(Duration) {
        |_| {}
    }

    fn sfx(prompt: &str) -> SfxParams {
        SfxParams {
            prompt: prompt.to_string(),
            seconds: Some(1.0),
            influence: Some(0.4),
            looping: false,
            format: DEFAULT_SFX_FORMAT.to_string(),
        }
    }

    fn music(prompt: &str) -> MusicParams {
        MusicParams {
            prompt: prompt.to_string(),
            length_ms: Some(10_000),
            model: DEFAULT_MUSIC_MODEL.to_string(),
            instrumental: true,
            format: DEFAULT_MUSIC_FORMAT.to_string(),
        }
    }

    #[test]
    fn parse_format_recognises_codecs() {
        assert_eq!(
            parse_format("pcm_24000").unwrap(),
            Encoding::Pcm { sample_rate: 24000 }
        );
        assert_eq!(parse_format("mp3_44100_128").unwrap(), Encoding::Mp3);
        assert_eq!(parse_format("opus_48000_64").unwrap(), Encoding::Opus);
        assert!(parse_format("ulaw_8000").is_err());
        assert!(parse_format("pcm").is_err());
        assert!(parse_format("mp3_fast").is_err());
        assert_eq!(Encoding::Pcm { sample_rate: 1 }.extension(), "wav");
        assert_eq!(Encoding::Mp3.extension(), "mp3");
        assert_eq!(Encoding::Opus.extension(), "opus");
    }

    #[test]
    fn sfx_request_matches_api_shape() {
        let mut params = sfx("rail shot");
        params.looping = true;
        let request = sfx_request(&params).unwrap();
        assert_eq!(request.method, Method::Post);
        assert_eq!(request.path, "/v1/sound-generation");
        assert_eq!(
            request.query,
            vec![("output_format".to_string(), "pcm_24000".to_string())]
        );
        let body = request.body.unwrap();
        assert_eq!(body["text"], "rail shot");
        assert_eq!(body["duration_seconds"], 1.0);
        assert_eq!(body["prompt_influence"], 0.4);
        assert_eq!(body["loop"], true);
        assert_eq!(body["model_id"], SFX_MODEL);
    }

    #[test]
    fn sfx_request_omits_optional_fields() {
        let params = SfxParams {
            seconds: None,
            influence: None,
            ..sfx("x")
        };
        let body = sfx_request(&params).unwrap().body.unwrap();
        assert!(body.get("duration_seconds").is_none());
        assert!(body.get("prompt_influence").is_none());
        assert_eq!(body["loop"], false);
    }

    #[test]
    fn sfx_validation_rejects_out_of_range() {
        assert!(sfx("").validate().is_err());
        assert!(SfxParams {
            seconds: Some(0.1),
            ..sfx("x")
        }
        .validate()
        .is_err());
        assert!(SfxParams {
            seconds: Some(31.0),
            ..sfx("x")
        }
        .validate()
        .is_err());
        assert!(SfxParams {
            influence: Some(1.5),
            ..sfx("x")
        }
        .validate()
        .is_err());
        assert!(SfxParams {
            format: "wav_44100".to_string(),
            ..sfx("x")
        }
        .validate()
        .is_err());
        assert!(sfx("ok").validate().is_ok());
    }

    #[test]
    fn music_request_matches_api_shape() {
        let request = music_request(&music("arena loop")).unwrap();
        assert_eq!(request.path, "/v1/music");
        assert_eq!(request.query[0].1, "mp3_44100_128");
        let body = request.body.unwrap();
        assert_eq!(body["prompt"], "arena loop");
        assert_eq!(body["music_length_ms"], 10_000);
        assert_eq!(body["model_id"], "music_v2_5");
        assert_eq!(body["force_instrumental"], true);
    }

    #[test]
    fn music_validation_rejects_bad_values() {
        assert!(music(" ").validate().is_err());
        assert!(MusicParams {
            length_ms: Some(100),
            ..music("x")
        }
        .validate()
        .is_err());
        assert!(MusicParams {
            length_ms: Some(700_000),
            ..music("x")
        }
        .validate()
        .is_err());
        assert!(MusicParams {
            model: "music_v9".to_string(),
            ..music("x")
        }
        .validate()
        .is_err());
        assert!(MusicParams {
            length_ms: None,
            ..music("x")
        }
        .validate()
        .is_ok());
        let body = music_request(&MusicParams {
            length_ms: None,
            ..music("x")
        })
        .unwrap()
        .body
        .unwrap();
        assert!(body.get("music_length_ms").is_none());
    }

    #[test]
    fn quota_request_is_a_get() {
        let request = quota_request();
        assert_eq!(request.method, Method::Get);
        assert_eq!(request.path, "/v1/user/subscription");
        assert!(request.body.is_none());
    }

    #[test]
    fn subscription_parses_and_describes() {
        let body = br#"{"tier":"creator","status":"active","character_count":1200,"character_limit":100000,"next_character_count_reset_unix":1760000000,"extra":true}"#;
        let sub = parse_subscription(body).unwrap();
        assert_eq!(sub.tier.as_deref(), Some("creator"));
        let text = describe_subscription(&sub);
        assert!(text.contains("tier: creator"));
        assert!(text.contains("remaining 98800"));
        assert!(text.contains("1760000000 (unix)"));
        let empty = describe_subscription(&Subscription::default());
        assert!(empty.contains("tier: unknown"));
        assert!(empty.contains("resets at: unknown"));
        assert!(parse_subscription(b"nope").is_err());
    }

    #[test]
    fn api_error_message_handles_all_shapes() {
        let structured =
            br#"{"detail":{"code":"insufficient_credits","message":"Not enough credits"}}"#;
        assert_eq!(
            api_error_message(402, structured),
            "insufficient_credits: Not enough credits"
        );
        let legacy = br#"{"detail":{"status":"quota_exceeded","message":"Quota"}}"#;
        assert_eq!(api_error_message(401, legacy), "quota_exceeded: Quota");
        let code_only = br#"{"detail":{"code":"bad_prompt"}}"#;
        assert_eq!(api_error_message(400, code_only), "bad_prompt");
        let validation = br#"{"detail":[{"loc":["body","text"],"msg":"field required","type":"value_error"},{"msg":"other"}]}"#;
        assert_eq!(
            api_error_message(422, validation),
            "validation failed: body.text: field required; other"
        );
        assert_eq!(api_error_message(400, br#"{"detail":"plain"}"#), "plain");
        assert_eq!(api_error_message(500, b"gateway down"), "gateway down");
        assert_eq!(api_error_message(503, b"   "), "HTTP 503 with empty body");
        let long = "x".repeat(400);
        assert_eq!(api_error_message(500, long.as_bytes()).len(), 200);
    }

    #[test]
    fn retry_only_on_rate_limit_and_server_errors() {
        assert!(should_retry(429));
        assert!(should_retry(500));
        assert!(should_retry(503));
        assert!(!should_retry(400));
        assert!(!should_retry(401));
        assert!(!should_retry(422));
    }

    #[test]
    fn send_with_retry_backs_off_then_succeeds() {
        let transport = FakeTransport::new(vec![
            Response {
                status: 429,
                body: b"slow down".to_vec(),
            },
            Response {
                status: 503,
                body: Vec::new(),
            },
            Response {
                status: 200,
                body: b"ok".to_vec(),
            },
        ]);
        let mut delays = Vec::new();
        let policy = RetryPolicy {
            attempts: 4,
            base_delay: Duration::from_millis(10),
        };
        let response = send_with_retry(&transport, "key", &quota_request(), &policy, &mut |d| {
            delays.push(d)
        })
        .unwrap();
        assert_eq!(response.body, b"ok");
        assert_eq!(
            delays,
            vec![Duration::from_millis(10), Duration::from_millis(20)]
        );
        assert_eq!(transport.seen.borrow().len(), 3);
        assert_eq!(transport.seen.borrow()[0].0, "key");
    }

    #[test]
    fn send_with_retry_gives_up_after_attempts() {
        let transport = FakeTransport::new(vec![
            Response {
                status: 500,
                body: Vec::new(),
            },
            Response {
                status: 500,
                body: Vec::new(),
            },
        ]);
        let policy = RetryPolicy {
            attempts: 2,
            base_delay: Duration::from_millis(1),
        };
        let err = send_with_retry(&transport, "k", &quota_request(), &policy, &mut no_sleep())
            .unwrap_err();
        assert!(matches!(err, Error::Api { status: 500, .. }));
        assert_eq!(transport.seen.borrow().len(), 2);
    }

    #[test]
    fn send_with_retry_does_not_retry_client_errors() {
        let transport = FakeTransport::new(vec![Response {
            status: 401,
            body: br#"{"detail":{"code":"invalid_api_key","message":"bad key"}}"#.to_vec(),
        }]);
        let err = send_with_retry(
            &transport,
            "k",
            &quota_request(),
            &RetryPolicy::default(),
            &mut no_sleep(),
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "API error 401: invalid_api_key: bad key");
        assert_eq!(transport.seen.borrow().len(), 1);
    }

    #[test]
    fn send_with_retry_propagates_transport_failures() {
        let transport = FakeTransport::new(Vec::new());
        let err = send_with_retry(
            &transport,
            "k",
            &quota_request(),
            &RetryPolicy::default(),
            &mut no_sleep(),
        )
        .unwrap_err();
        assert!(matches!(err, Error::Transport(_)));
        assert!(err.to_string().contains("no scripted response"));
    }

    #[test]
    fn wav_header_is_canonical() {
        let pcm = vec![1u8, 0, 2, 0, 3, 0, 4, 0];
        let wav = wav_from_pcm_s16le(&pcm, 24000, 1);
        assert_eq!(wav.len(), 44 + pcm.len());
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 36 + 8);
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(u32::from_le_bytes(wav[16..20].try_into().unwrap()), 16);
        assert_eq!(u16::from_le_bytes(wav[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24000);
        assert_eq!(u32::from_le_bytes(wav[28..32].try_into().unwrap()), 48000);
        assert_eq!(u16::from_le_bytes(wav[32..34].try_into().unwrap()), 2);
        assert_eq!(u16::from_le_bytes(wav[34..36].try_into().unwrap()), 16);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 8);
        assert_eq!(&wav[44..], &pcm[..]);
        let stereo = wav_from_pcm_s16le(&pcm, 44100, 2);
        assert_eq!(
            u32::from_le_bytes(stereo[28..32].try_into().unwrap()),
            176_400
        );
        assert_eq!(u16::from_le_bytes(stereo[32..34].try_into().unwrap()), 4);
    }

    #[test]
    fn channel_inference_uses_expected_duration() {
        let rate = 24000;
        let one_second_mono = rate as usize * 2;
        assert_eq!(infer_channels(one_second_mono, rate, Some(1.0)), 1);
        assert_eq!(infer_channels(one_second_mono * 2, rate, Some(1.0)), 2);
        assert_eq!(infer_channels(one_second_mono * 2, rate, None), 1);
        assert_eq!(infer_channels(0, rate, Some(1.0)), 1);
        assert_eq!(infer_channels(one_second_mono, 0, Some(1.0)), 1);
    }

    #[test]
    fn encode_output_wraps_pcm_and_passes_mp3() {
        let (wav, channels) = encode_output(
            vec![0u8; 48000],
            &Encoding::Pcm { sample_rate: 24000 },
            Some(1.0),
        );
        assert_eq!(channels, 1);
        assert_eq!(&wav[0..4], b"RIFF");
        let (mp3, channels) = encode_output(vec![9u8; 3], &Encoding::Mp3, None);
        assert_eq!(channels, 0);
        assert_eq!(mp3, vec![9u8; 3]);
        let (opus, _) = encode_output(vec![1u8], &Encoding::Opus, None);
        assert_eq!(opus, vec![1u8]);
    }

    #[test]
    fn job_accessors_cover_both_kinds() {
        let s = Job::Sfx(sfx("bang"));
        assert_eq!(s.kind(), "sfx");
        assert_eq!(s.model(), SFX_MODEL);
        assert_eq!(s.prompt(), "bang");
        assert_eq!(s.format(), DEFAULT_SFX_FORMAT);
        assert_eq!(s.expected_seconds(), Some(1.0));
        assert_eq!(s.encoding().unwrap(), Encoding::Pcm { sample_rate: 24000 });
        let m = Job::Music(music("bed"));
        assert_eq!(m.kind(), "music");
        assert_eq!(m.model(), "music_v2_5");
        assert_eq!(m.prompt(), "bed");
        assert_eq!(m.expected_seconds(), Some(10.0));
        assert_eq!(m.request().unwrap().path, "/v1/music");
    }

    #[test]
    fn spec_parses_and_applies_defaults() {
        let json = r#"{
            "out_dir": "client/assets/audio",
            "items": [
                {"kind": "sfx", "name": "fire_rail", "prompt": "rail shot", "seconds": 0.9, "loop": false},
                {"kind": "music", "name": "music/match_01", "prompt": "bed", "length_ms": 60000, "instrumental": true}
            ]
        }"#;
        let spec = parse_spec(json).unwrap();
        assert_eq!(spec.out_dir.as_deref(), Some("client/assets/audio"));
        assert_eq!(spec.items.len(), 2);
        assert_eq!(spec.items[0].name(), "fire_rail");
        match spec.items[0].clone().into_job() {
            Job::Sfx(params) => {
                assert_eq!(params.format, DEFAULT_SFX_FORMAT);
                assert_eq!(params.seconds, Some(0.9));
                assert_eq!(params.influence, None);
            }
            Job::Music(_) => panic!("expected sfx"),
        }
        match spec.items[1].clone().into_job() {
            Job::Music(params) => {
                assert_eq!(params.model, DEFAULT_MUSIC_MODEL);
                assert_eq!(params.format, DEFAULT_MUSIC_FORMAT);
                assert!(params.instrumental);
            }
            Job::Sfx(_) => panic!("expected music"),
        }
    }

    #[test]
    fn spec_rejects_bad_input() {
        assert!(parse_spec("{").is_err());
        assert!(parse_spec(r#"{"items": []}"#).is_err());
        assert!(parse_spec(
            r#"{"items": [{"kind": "sfx", "name": "a", "prompt": "p", "bogus": 1}]}"#
        )
        .is_err());
        assert!(
            parse_spec(r#"{"items": [{"kind": "sfx", "name": "Bad Name", "prompt": "p"}]}"#)
                .is_err()
        );
        let dup = r#"{"items": [{"kind": "sfx", "name": "a", "prompt": "p"}, {"kind": "sfx", "name": "a", "prompt": "q"}]}"#;
        assert!(parse_spec(dup)
            .unwrap_err()
            .to_string()
            .contains("duplicate"));
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("fire_rail").is_ok());
        assert!(validate_name("music/match-01").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name("/abs").is_err());
        assert!(validate_name("trail/").is_err());
        assert!(validate_name("../escape").is_err());
        assert!(validate_name("a//b").is_err());
        assert!(validate_name("Fire").is_err());
        assert!(validate_name("fire.wav").is_err());
    }

    #[test]
    fn manifest_round_trips() {
        let dir = temp_dir("manifest");
        assert_eq!(load_manifest(&dir).unwrap(), Manifest::default());
        let mut manifest = Manifest::default();
        manifest.entries.insert(
            "fire".to_string(),
            ManifestEntry {
                kind: "sfx".into(),
                file: "fire.wav".into(),
                prompt: "p".into(),
                model: SFX_MODEL.into(),
                format: "pcm_24000".into(),
                seconds: Some(1.0),
                length_ms: None,
                looping: false,
                channels: 1,
                bytes: 10,
                generated_unix: 5,
            },
        );
        save_manifest(&dir, &manifest).unwrap();
        assert_eq!(load_manifest(&dir).unwrap(), manifest);
        fs::write(dir.join(MANIFEST_FILE), "{broken").unwrap();
        assert!(load_manifest(&dir).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn dotenv_parsing() {
        let dir = temp_dir("dotenv");
        let path = dir.join(".env");
        assert_eq!(read_dotenv_key(&path).unwrap(), None);
        fs::write(
            &path,
            "# comment\nOTHER=1\nexport elevenlabs = \"sk_abc\"\n",
        )
        .unwrap();
        assert_eq!(read_dotenv_key(&path).unwrap().as_deref(), Some("sk_abc"));
        fs::write(&path, "ELEVENLABS_API_KEY='quoted'\n").unwrap();
        assert_eq!(read_dotenv_key(&path).unwrap().as_deref(), Some("quoted"));
        fs::write(&path, "XI_API_KEY=\nELEVENLABS_KEY=later\nno-equals-line\n").unwrap();
        assert_eq!(read_dotenv_key(&path).unwrap().as_deref(), Some("later"));
        fs::write(&path, "UNRELATED=x\n").unwrap();
        assert_eq!(read_dotenv_key(&path).unwrap(), None);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn api_key_resolution() {
        assert!(matches!(
            resolve_api_key(None, None),
            Err(Error::MissingApiKey)
        ));
        assert!(matches!(
            resolve_api_key(Some("  ".into()), None),
            Err(Error::MissingApiKey)
        ));
        assert_eq!(resolve_api_key(Some(" abc\n".into()), None).unwrap(), "abc");
        let dir = temp_dir("key");
        let key_file = dir.join("elevenlabs.key");
        fs::write(&key_file, "from-file\n").unwrap();
        assert_eq!(
            resolve_api_key(Some("env".into()), Some(&key_file)).unwrap(),
            "from-file"
        );
        assert!(resolve_api_key(None, Some(&dir.join("missing"))).is_err());
        fs::remove_dir_all(&dir).unwrap();
        assert!(Error::MissingApiKey.to_string().contains(API_KEY_ENV));
    }

    #[test]
    fn generator_writes_wav_and_manifest() {
        let dir = temp_dir("gen");
        let transport = FakeTransport::ok(vec![0u8; 24000 * 2]);
        let generator = Generator {
            transport: &transport,
            api_key: "k".into(),
            out_dir: dir.clone(),
            overwrite: false,
            dry_run: false,
            retry: RetryPolicy::default(),
        };
        let outcome = generator
            .run("sfx/fire_rail", &Job::Sfx(sfx("rail")), &mut no_sleep())
            .unwrap();
        match outcome {
            Outcome::Written {
                path,
                bytes,
                channels,
            } => {
                assert_eq!(path, dir.join("sfx/fire_rail.wav"));
                assert_eq!(bytes, 44 + 48000);
                assert_eq!(channels, 1);
                assert!(path.exists());
            }
            other => panic!("unexpected {other:?}"),
        }
        let manifest = load_manifest(&dir).unwrap();
        let entry = &manifest.entries["sfx/fire_rail"];
        assert_eq!(entry.file, "sfx/fire_rail.wav");
        assert_eq!(entry.kind, "sfx");
        assert_eq!(entry.seconds, Some(1.0));
        assert!(entry.generated_unix > 0);
        let skipped = generator
            .run("sfx/fire_rail", &Job::Sfx(sfx("rail")), &mut no_sleep())
            .unwrap();
        assert!(matches!(skipped, Outcome::Skipped(_)));
        assert_eq!(transport.seen.borrow().len(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn generator_overwrites_music_and_records_length() {
        let dir = temp_dir("music");
        fs::write(dir.join("bed.mp3"), b"old").unwrap();
        let transport = FakeTransport::ok(b"new-mp3".to_vec());
        let generator = Generator {
            transport: &transport,
            api_key: "k".into(),
            out_dir: dir.clone(),
            overwrite: true,
            dry_run: false,
            retry: RetryPolicy::default(),
        };
        let outcome = generator
            .run("bed", &Job::Music(music("bed")), &mut no_sleep())
            .unwrap();
        assert!(matches!(
            outcome,
            Outcome::Written {
                channels: 0,
                bytes: 7,
                ..
            }
        ));
        assert_eq!(fs::read(dir.join("bed.mp3")).unwrap(), b"new-mp3");
        let entry = &load_manifest(&dir).unwrap().entries["bed"];
        assert_eq!(entry.length_ms, Some(10_000));
        assert_eq!(entry.seconds, None);
        assert_eq!(entry.model, "music_v2_5");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn generator_dry_run_and_failures() {
        let dir = temp_dir("dry");
        let transport = FakeTransport::ok(Vec::new());
        let generator = Generator {
            transport: &transport,
            api_key: "k".into(),
            out_dir: dir.clone(),
            overwrite: false,
            dry_run: true,
            retry: RetryPolicy::default(),
        };
        let outcome = generator
            .run("fire", &Job::Sfx(sfx("x")), &mut no_sleep())
            .unwrap();
        assert!(matches!(outcome, Outcome::DryRun { .. }));
        assert!(transport.seen.borrow().is_empty());
        assert!(generator
            .run("Bad", &Job::Sfx(sfx("x")), &mut no_sleep())
            .is_err());
        let live = Generator {
            dry_run: false,
            ..generator
        };
        let err = live
            .run("fire", &Job::Sfx(sfx("x")), &mut no_sleep())
            .unwrap_err();
        assert!(err.to_string().contains("empty audio body"));
        assert!(!dir.join("fire.wav").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn run_command_quota_prints_summary() {
        let transport = FakeTransport::ok(
            br#"{"tier":"pro","character_limit":10,"character_count":4}"#.to_vec(),
        );
        let mut out = Vec::new();
        run_command(
            &Command::Quota,
            &RunOptions::default(),
            &transport,
            "k",
            &mut out,
            &mut no_sleep(),
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("tier: pro"));
        assert!(text.contains("remaining 6"));
    }

    #[test]
    fn run_command_generate_reports_outcomes() {
        let dir = temp_dir("cmd");
        let transport = FakeTransport::ok(vec![0u8; 100]);
        let options = RunOptions {
            out_dir: Some(dir.clone()),
            overwrite: false,
            dry_run: false,
        };
        let command = Command::Generate {
            name: "hit".into(),
            job: Job::Sfx(SfxParams {
                seconds: None,
                ..sfx("hit")
            }),
        };
        let mut out = Vec::new();
        run_command(
            &command,
            &options,
            &transport,
            "k",
            &mut out,
            &mut no_sleep(),
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("wrote hit -> "));
        assert!(text.contains("144 bytes, 1 ch"));
        let mut out = Vec::new();
        run_command(
            &command,
            &options,
            &transport,
            "k",
            &mut out,
            &mut no_sleep(),
        )
        .unwrap();
        assert!(String::from_utf8(out).unwrap().starts_with("skip hit: "));
        let dry = RunOptions {
            dry_run: true,
            ..options.clone()
        };
        let mut out = Vec::new();
        run_command(&command, &dry, &transport, "k", &mut out, &mut no_sleep()).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("dry-run hit -> "));
        assert!(text.contains("POST /v1/sound-generation?output_format=pcm_24000"));
        assert!(text.contains("\"text\":\"hit\""));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn run_command_batch_filters_and_uses_spec_dir() {
        let dir = temp_dir("batch");
        let spec = parse_spec(&format!(
            r#"{{"out_dir": "{}", "items": [
                {{"kind": "sfx", "name": "a", "prompt": "pa"}},
                {{"kind": "music", "name": "b", "prompt": "pb", "length_ms": 5000}}
            ]}}"#,
            dir.display().to_string().replace('\\', "/")
        ))
        .unwrap();
        let transport = FakeTransport::new(vec![Response {
            status: 200,
            body: vec![1u8; 10],
        }]);
        let mut out = Vec::new();
        let command = Command::Batch {
            spec: spec.clone(),
            only: Some("b".into()),
        };
        run_command(
            &command,
            &RunOptions::default(),
            &transport,
            "k",
            &mut out,
            &mut no_sleep(),
        )
        .unwrap();
        assert!(String::from_utf8(out).unwrap().starts_with("wrote b -> "));
        assert!(dir.join("b.mp3").exists());
        assert!(!dir.join("a.wav").exists());
        let missing = Command::Batch {
            spec: spec.clone(),
            only: Some("zzz".into()),
        };
        let err = run_command(
            &missing,
            &RunOptions::default(),
            &transport,
            "k",
            &mut Vec::new(),
            &mut no_sleep(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no spec item named 'zzz'"));
        let all = Command::Batch { spec, only: None };
        let transport = FakeTransport::new(vec![
            Response {
                status: 200,
                body: vec![1u8; 10],
            },
            Response {
                status: 422,
                body: br#"{"detail":[{"msg":"bad"}]}"#.to_vec(),
            },
        ]);
        let options = RunOptions {
            out_dir: Some(dir.clone()),
            overwrite: true,
            dry_run: false,
        };
        let err = run_command(
            &all,
            &options,
            &transport,
            "k",
            &mut Vec::new(),
            &mut no_sleep(),
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "API error 422: validation failed: bad");
        assert!(dir.join("a.wav").exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn error_display_covers_variants() {
        assert_eq!(
            Error::InvalidArgument("x".into()).to_string(),
            "invalid argument: x"
        );
        assert_eq!(
            Error::Transport("t".into()).to_string(),
            "transport error: t"
        );
        assert_eq!(Error::Spec("s".into()).to_string(), "spec error: s");
        let io: Error = std::io::Error::other("disk").into();
        assert!(io.to_string().contains("disk"));
    }
}
