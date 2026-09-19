//! Sprite generation for fragr through the Higgsfield API.
//!
//! Developer-only. Never in CI, never called by the game. It exists because the
//! art list in `docs/ART-ASSET-LIST.md` is thousands of frames and the contract
//! in `docs/ART-GENERATION-SPEC.md` is precise enough to execute rather than
//! interpret.
//!
//! Three rules shape the whole file.
//!
//! Spending is gated before anything is generated, not after. Higgsfield prices
//! a request through an estimate endpoint that costs nothing, so the tool prices
//! the entire run, compares the total against a cap the caller had to type, and
//! refuses the run as a whole rather than discovering halfway through that it
//! has spent more than it meant to.
//!
//! Every frame is recorded in an append-only ledger the moment it lands. A run
//! that is interrupted, rate limited or cancelled can be repeated and generates
//! only what is missing, because the expensive failure for this tool is paying
//! twice for the same picture.
//!
//! Model parameters are passed through rather than modelled. The providers
//! behind this one endpoint disagree about what resolution means, whether a
//! batch exists and what a quality knob is called, and encoding those dialects
//! in Rust would mean editing this file every time a model is added. The spec
//! carries a `params` object, the tool adds the prompt, and the provider
//! validates the rest.

pub mod reduce;

use std::collections::BTreeSet;
use std::fmt;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Map, Value};

/// The fixed house style from `docs/ART-GENERATION-SPEC.md`.
///
/// It never varies across the project, which is the only reason frames
/// generated in different sessions months apart can sit beside each other in
/// one game and look like one game.
pub const STYLE: &str = "retro pixel art sprite, 1990s PC shooter, chunky readable silhouette, \
limited palette of desaturated rust orange, bone white, gunmetal grey and deep shadow, \
flat even lighting, no gradients";

/// The fixed technical tail, also from the spec.
pub const TECHNICAL: &str = "transparent background, no shadow, no ambient occlusion, \
no specular highlights, no rim light, no ground plane, no text, no watermark, no border";

/// The fixed negative prompt.
///
/// Carried as words rather than as a field because none of the image models
/// reachable through this endpoint accept a negative prompt. Appending it as
/// avoidance language is weaker than a real negative conditioning, so the
/// contract is enforced twice more: by the palette quantisation step and by
/// throwing frames away.
pub const NEGATIVE: &str = "photorealistic, 3d render, smooth shading, gradient, blur, \
anti-aliasing, drop shadow, cast shadow, ambient occlusion, specular, glossy, reflection, \
rim light, lens flare, bloom, depth of field, text, signature, watermark, frame, border, \
background scenery";

/// Base of the Higgsfield API.
pub const API_BASE: &str = "https://api.higgsfield.ai";

/// The most a single run may spend, whatever the caller asks for.
///
/// The number is small on purpose, and matches the ceiling the brain agent
/// uses for paid model calls. A run that genuinely needs more than this is a
/// run that should be split and approved a piece at a time.
pub const HARD_CAP_USD: f64 = 5.0;

/// Request states that end a poll, from `docs/concepts/polling`.
pub const TERMINAL_STATUSES: [&str; 4] = ["completed", "failed", "nsfw", "canceled"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub url: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

#[derive(Debug)]
pub enum Error {
    Transport(String),
    Api { status: u16, body: String },
    Spec(String),
    Budget(String),
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Transport(msg) => write!(f, "transport error: {msg}"),
            Error::Api { status, body } => write!(f, "api error {status}: {body}"),
            Error::Spec(msg) => write!(f, "spec error: {msg}"),
            Error::Budget(msg) => write!(f, "budget refused: {msg}"),
            Error::Io(msg) => write!(f, "io error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// The network, behind a trait so every test in this crate runs offline.
pub trait Transport {
    fn send(&self, credential: &str, request: &Request) -> Result<Response, Error>;
    fn download(&self, url: &str) -> Result<Vec<u8>, Error>;
}

/// One frame to generate.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub id: String,
    pub subject: String,
    pub view: String,
    /// The house style, or an override for it.
    ///
    /// Overridable only so that two ways of asking for the same sprite can be
    /// run side by side and compared. Production specs leave it alone; a run
    /// where every frame sets its own style has given up the one property that
    /// makes thousands of generated frames look like one game.
    pub style: String,
    /// Model parameters, already merged from the spec defaults.
    pub params: Map<String, Value>,
}

/// A whole run.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub model: String,
    pub out_dir: PathBuf,
    pub frames: Vec<Frame>,
}

impl Frame {
    /// `<STYLE> <SUBJECT> <VIEW> <TECHNICAL>`, in that order, always, then the
    /// negative prompt as avoidance language.
    pub fn prompt(&self) -> String {
        let mut parts: Vec<&str> = vec![self.style.as_str(), self.subject.as_str()];
        if !self.view.trim().is_empty() {
            parts.push(self.view.as_str());
        }
        parts.push(TECHNICAL);
        let positive = parts
            .iter()
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{positive}. Avoid: {NEGATIVE}.")
    }

    /// The request body: the spec's parameters with the prompt added.
    ///
    /// The prompt is inserted last and overwrites, so a spec cannot accidentally
    /// bypass the house style by setting `prompt` itself.
    pub fn body(&self) -> Value {
        let mut map = self.params.clone();
        map.insert("prompt".into(), Value::String(self.prompt()));
        Value::Object(map)
    }
}

/// Read a spec file.
///
/// Defaults live in one `params` object on the spec so that adding a frame is
/// two lines rather than eight, and so that changing the resolution of a run is
/// one edit rather than one per frame.
pub fn parse_spec(text: &str) -> Result<Spec, Error> {
    let root: Value =
        serde_json::from_str(text).map_err(|e| Error::Spec(format!("not valid json: {e}")))?;

    let model = root
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Spec("missing \"model\"".into()))?
        .to_string();
    let out_dir = root
        .get("out_dir")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Spec("missing \"out_dir\"".into()))?;

    let defaults = root
        .get("params")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let default_view = root
        .get("view")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let default_style = root
        .get("style")
        .and_then(Value::as_str)
        .unwrap_or(STYLE)
        .to_string();

    let raw_frames = root
        .get("frames")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Spec("missing \"frames\" array".into()))?;
    if raw_frames.is_empty() {
        return Err(Error::Spec("\"frames\" is empty".into()));
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut frames = Vec::with_capacity(raw_frames.len());
    for (index, raw) in raw_frames.iter().enumerate() {
        let id = raw
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Spec(format!("frame {index} is missing \"id\"")))?
            .to_string();
        if id.trim().is_empty() {
            return Err(Error::Spec(format!("frame {index} has an empty \"id\"")));
        }
        // The id becomes a file name, so a stray slash would silently write
        // outside the output directory.
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return Err(Error::Spec(format!(
                "frame id \"{id}\" is not usable as a file name"
            )));
        }
        if !seen.insert(id.clone()) {
            return Err(Error::Spec(format!("duplicate frame id \"{id}\"")));
        }
        let subject = raw
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Spec(format!("frame \"{id}\" is missing \"subject\"")))?
            .to_string();

        let mut params = defaults.clone();
        if let Some(overrides) = raw.get("params").and_then(Value::as_object) {
            for (key, value) in overrides {
                params.insert(key.clone(), value.clone());
            }
        }

        frames.push(Frame {
            id,
            subject,
            view: raw
                .get("view")
                .and_then(Value::as_str)
                .map_or_else(|| default_view.clone(), str::to_string),
            style: raw
                .get("style")
                .and_then(Value::as_str)
                .map_or_else(|| default_style.clone(), str::to_string),
            params,
        });
    }

    Ok(Spec {
        model,
        out_dir: PathBuf::from(out_dir),
        frames,
    })
}

/// What one frame costs, in dollars, according to the provider.
pub fn estimate(
    transport: &dyn Transport,
    credential: &str,
    model: &str,
    frame: &Frame,
) -> Result<f64, Error> {
    let request = Request {
        method: Method::Post,
        url: format!("{API_BASE}/estimate/{model}"),
        body: Some(frame.body().to_string()),
    };
    let response = transport.send(credential, &request)?;
    if response.status != 200 {
        return Err(Error::Api {
            status: response.status,
            body: response.body,
        });
    }
    let value: Value = serde_json::from_str(&response.body)
        .map_err(|e| Error::Transport(format!("estimate was not json: {e}")))?;
    parse_usd(&value)
}

/// Read the dollar cost out of an estimate.
///
/// The API sends money as a decimal string, which is right of it. A missing
/// field is an error rather than zero, because the one thing a budget gate must
/// never do is treat an unreadable price as free.
pub fn parse_usd(value: &Value) -> Result<f64, Error> {
    let raw = value
        .get("usd")
        .ok_or_else(|| Error::Transport(format!("estimate had no \"usd\" field: {value}")))?;
    let parsed = match raw {
        Value::String(s) => s.parse::<f64>().ok(),
        Value::Number(n) => n.as_f64(),
        _ => None,
    };
    let usd = parsed
        .ok_or_else(|| Error::Transport(format!("estimate \"usd\" was not a number: {raw}")))?;
    if !usd.is_finite() || usd < 0.0 {
        return Err(Error::Transport(format!("estimate \"usd\" was {usd}")));
    }
    Ok(usd)
}

/// The gate. Called with the whole run priced, before a single generation.
pub fn check_budget(total_usd: f64, max_spend_usd: Option<f64>) -> Result<f64, Error> {
    let Some(cap) = max_spend_usd else {
        return Err(Error::Budget(
            "a live run needs --max-spend-usd. Nothing was generated.".into(),
        ));
    };
    if !cap.is_finite() || cap <= 0.0 {
        return Err(Error::Budget(format!(
            "--max-spend-usd {cap} is not a spend"
        )));
    }
    if cap > HARD_CAP_USD {
        return Err(Error::Budget(format!(
            "--max-spend-usd {cap:.2} is above the {HARD_CAP_USD:.2} ceiling for one run. \
Split the spec and approve it a piece at a time."
        )));
    }
    if total_usd > cap {
        return Err(Error::Budget(format!(
            "this run costs {total_usd:.2} and the cap is {cap:.2}. Nothing was generated."
        )));
    }
    Ok(total_usd)
}

/// Pull every image URL out of a completed response.
///
/// Written as a walk rather than against one shape because the documented
/// completed body and the SDK example disagree about where the URL lives, and
/// because this endpoint fronts many providers. A tolerant reader costs less
/// than a run that pays for images and then cannot find them.
pub fn image_urls(value: &Value) -> Vec<String> {
    fn walk(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if key == "url" {
                        if let Some(url) = child.as_str() {
                            let url = url.to_string();
                            if !out.contains(&url) {
                                out.push(url);
                            }
                            continue;
                        }
                    }
                    walk(child, out);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(item, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(value, &mut out);
    // The submission response echoes these back beside the results and they are
    // not pictures.
    out.retain(|u| !u.ends_with("/status") && !u.ends_with("/cancel"));
    out
}

/// Submit one frame and return its status URL.
pub fn submit(
    transport: &dyn Transport,
    credential: &str,
    model: &str,
    frame: &Frame,
) -> Result<String, Error> {
    let request = Request {
        method: Method::Post,
        url: format!("{API_BASE}/{model}"),
        body: Some(frame.body().to_string()),
    };
    let response = transport.send(credential, &request)?;
    if response.status != 200 && response.status != 201 {
        return Err(Error::Api {
            status: response.status,
            body: response.body,
        });
    }
    let value: Value = serde_json::from_str(&response.body)
        .map_err(|e| Error::Transport(format!("submit was not json: {e}")))?;
    value
        .get("status_url")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| Error::Transport(format!("submit had no status_url: {}", response.body)))
}

/// Poll to a terminal state with the backoff the docs ask for: two seconds,
/// rising by half each time, capped at ten.
pub fn poll(
    transport: &dyn Transport,
    credential: &str,
    status_url: &str,
    sleep: &mut dyn FnMut(Duration),
    max_attempts: u32,
) -> Result<Value, Error> {
    let mut delay = Duration::from_secs(2);
    for _ in 0..max_attempts {
        let response = transport.send(
            credential,
            &Request {
                method: Method::Get,
                url: status_url.to_string(),
                body: None,
            },
        )?;
        match response.status {
            200 => {
                let value: Value = serde_json::from_str(&response.body)
                    .map_err(|e| Error::Transport(format!("status was not json: {e}")))?;
                let status = value.get("status").and_then(Value::as_str).unwrap_or("");
                if TERMINAL_STATUSES.contains(&status) {
                    return Ok(value);
                }
            }
            // Credentials and identity are not going to improve by waiting.
            401 | 403 | 404 => {
                return Err(Error::Api {
                    status: response.status,
                    body: response.body,
                });
            }
            _ => {}
        }
        sleep(delay);
        delay = std::cmp::min(delay.mul_f64(1.5), Duration::from_secs(10));
    }
    Err(Error::Transport(format!(
        "gave up polling {status_url} after {max_attempts} attempts"
    )))
}

/// One line of the ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    pub id: String,
    pub usd: f64,
    pub files: Vec<String>,
}

/// Ids already generated, read from the ledger beside the output.
pub fn ledger_ids(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|v| v.get("id").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// What the ledger says has been spent so far in this output directory.
pub fn ledger_spent(text: &str) -> f64 {
    text.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|v| v.get("usd").and_then(Value::as_f64))
        .sum()
}

pub fn ledger_path(out_dir: &Path) -> PathBuf {
    out_dir.join("ledger.jsonl")
}

/// Read the ledger, treating a missing file as an empty one.
pub fn read_ledger(out_dir: &Path) -> String {
    std::fs::read_to_string(ledger_path(out_dir)).unwrap_or_default()
}

/// Append one entry. Called the moment a frame lands, so an interrupted run
/// never loses the record of something it has already paid for.
pub fn append_ledger(out_dir: &Path, entry: &LedgerEntry) -> Result<(), Error> {
    let path = ledger_path(out_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
    }
    let line = serde_json::json!({
        "id": entry.id,
        "usd": entry.usd,
        "files": entry.files,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| Error::Io(e.to_string()))?;
    writeln!(file, "{line}").map_err(|e| Error::Io(e.to_string()))
}

/// Frames still to do, in spec order.
pub fn pending<'a>(spec: &'a Spec, done: &BTreeSet<String>) -> Vec<&'a Frame> {
    spec.frames
        .iter()
        .filter(|frame| !done.contains(&frame.id))
        .collect()
}

/// File name for one image of one frame.
pub fn file_name(id: &str, index: usize, url: &str) -> String {
    let extension = url
        .rsplit('?')
        .next_back()
        .unwrap_or(url)
        .rsplit('.')
        .next()
        .filter(|ext| ext.len() <= 4 && ext.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("png")
        .to_ascii_lowercase();
    format!("{id}_{index}.{extension}")
}

/// Read the Higgsfield credential from a dotenv file.
///
/// The credential is a single `id:secret` pair because that is exactly what the
/// `Authorization: Key` header wants, and splitting it into two settings would
/// only create a way to configure half of it.
pub fn read_dotenv_credential(path: &Path) -> Result<String, Error> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::Io(format!("could not read {}: {e}", path.display())))?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("higgsfield") {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if value.is_empty() {
                return Err(Error::Spec("higgsfield= in .env is empty".into()));
            }
            if !value.contains(':') {
                return Err(Error::Spec(
                    "higgsfield= in .env should be id:secret".into(),
                ));
            }
            return Ok(value.to_string());
        }
    }
    Err(Error::Spec(format!(
        "no higgsfield= line in {}",
        path.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn frame(id: &str) -> Frame {
        Frame {
            id: id.to_string(),
            subject: "a rusted pistol".into(),
            view: "side profile".into(),
            style: STYLE.to_string(),
            params: Map::new(),
        }
    }

    struct FakeTransport {
        responses: RefCell<Vec<Response>>,
        seen: RefCell<Vec<Request>>,
    }

    impl FakeTransport {
        fn new(responses: Vec<Response>) -> Self {
            FakeTransport {
                responses: RefCell::new(responses),
                seen: RefCell::new(Vec::new()),
            }
        }

        fn one(status: u16, body: &str) -> Self {
            FakeTransport::new(vec![Response {
                status,
                body: body.to_string(),
            }])
        }
    }

    impl Transport for FakeTransport {
        fn send(&self, _credential: &str, request: &Request) -> Result<Response, Error> {
            self.seen.borrow_mut().push(request.clone());
            let mut responses = self.responses.borrow_mut();
            if responses.is_empty() {
                return Err(Error::Transport("no scripted response".into()));
            }
            Ok(responses.remove(0))
        }

        fn download(&self, _url: &str) -> Result<Vec<u8>, Error> {
            Ok(vec![1, 2, 3])
        }
    }

    #[test]
    fn prompt_puts_style_first_and_technical_last() {
        let prompt = frame("tack").prompt();
        assert!(prompt.starts_with(STYLE), "style leads: {prompt}");
        assert!(prompt.contains("a rusted pistol"));
        assert!(prompt.contains("side profile"));
        assert!(prompt.contains(TECHNICAL));
        assert!(prompt.contains(NEGATIVE));
        let style_at = prompt.find(STYLE).unwrap();
        let subject_at = prompt.find("a rusted pistol").unwrap();
        let technical_at = prompt.find(TECHNICAL).unwrap();
        assert!(style_at < subject_at && subject_at < technical_at);
    }

    #[test]
    fn prompt_skips_an_empty_view_without_leaving_a_gap() {
        let mut f = frame("tack");
        f.view = "  ".into();
        let prompt = f.prompt();
        assert!(!prompt.contains(", ,"), "no empty slot: {prompt}");
    }

    #[test]
    fn body_carries_params_and_cannot_have_its_prompt_overridden() {
        let mut f = frame("tack");
        f.params
            .insert("quality".into(), Value::String("low".into()));
        f.params
            .insert("prompt".into(), Value::String("sneaky".into()));
        let body = f.body();
        assert_eq!(body["quality"], Value::String("low".into()));
        assert_ne!(body["prompt"], Value::String("sneaky".into()));
        assert!(body["prompt"].as_str().unwrap().starts_with(STYLE));
    }

    #[test]
    fn spec_merges_defaults_under_per_frame_overrides() {
        let spec = parse_spec(
            r#"{
              "model": "marketing-studio/image",
              "out_dir": "art/raw/weapons",
              "view": "side profile",
              "params": { "quality": "low", "resolution": "2k" },
              "frames": [
                { "id": "tack", "subject": "a pistol" },
                { "id": "rail", "subject": "a rifle", "view": "front", "params": { "quality": "medium" } }
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(spec.model, "marketing-studio/image");
        assert_eq!(spec.frames.len(), 2);
        assert_eq!(spec.frames[0].view, "side profile");
        assert_eq!(
            spec.frames[0].params["quality"],
            Value::String("low".into())
        );
        assert_eq!(spec.frames[1].view, "front");
        // The override wins, the untouched default survives.
        assert_eq!(
            spec.frames[1].params["quality"],
            Value::String("medium".into())
        );
        assert_eq!(
            spec.frames[1].params["resolution"],
            Value::String("2k".into())
        );
    }

    #[test]
    fn style_defaults_to_the_house_style_and_can_be_overridden_per_frame() {
        let spec = parse_spec(
            r#"{
              "model": "m",
              "out_dir": "o",
              "frames": [
                { "id": "house", "subject": "a pistol" },
                { "id": "other", "subject": "a pistol", "style": "high detail rendered prop" }
              ]
            }"#,
        )
        .unwrap();
        assert_eq!(spec.frames[0].style, STYLE);
        assert!(spec.frames[0].prompt().starts_with(STYLE));
        assert_eq!(spec.frames[1].style, "high detail rendered prop");
        assert!(spec.frames[1]
            .prompt()
            .starts_with("high detail rendered prop"));
        // The technical tail is not negotiable, whatever the style says.
        assert!(spec.frames[1].prompt().contains(TECHNICAL));
    }

    #[test]
    fn a_spec_level_style_applies_to_every_frame_that_does_not_override_it() {
        let spec = parse_spec(
            r#"{
              "model": "m",
              "out_dir": "o",
              "style": "run wide style",
              "frames": [{ "id": "a", "subject": "x" }, { "id": "b", "subject": "y" }]
            }"#,
        )
        .unwrap();
        assert!(spec.frames.iter().all(|f| f.style == "run wide style"));
    }

    #[test]
    fn spec_rejects_duplicate_ids() {
        let err = parse_spec(
            r#"{"model":"m","out_dir":"o","frames":[
                 {"id":"a","subject":"x"},{"id":"a","subject":"y"}]}"#,
        )
        .unwrap_err();
        assert!(format!("{err}").contains("duplicate"), "{err}");
    }

    #[test]
    fn spec_rejects_an_id_that_would_escape_the_output_directory() {
        for bad in ["../secrets", "a/b", "a\\b"] {
            // Built through serde rather than by formatting a string, so that
            // the backslash case is a real backslash in the parsed id rather
            // than a JSON escape that never reaches the check.
            let text = serde_json::json!({
                "model": "m",
                "out_dir": "o",
                "frames": [{ "id": bad, "subject": "x" }]
            })
            .to_string();
            assert!(parse_spec(&text).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn spec_rejects_an_empty_frame_list() {
        assert!(parse_spec(r#"{"model":"m","out_dir":"o","frames":[]}"#).is_err());
    }

    #[test]
    fn usd_is_read_from_a_string_or_a_number() {
        assert_eq!(
            parse_usd(&serde_json::json!({ "usd": "0.019" })).unwrap(),
            0.019
        );
        assert_eq!(
            parse_usd(&serde_json::json!({ "usd": 0.25 })).unwrap(),
            0.25
        );
    }

    #[test]
    fn a_missing_price_is_an_error_not_free() {
        assert!(parse_usd(&serde_json::json!({ "credits": "3" })).is_err());
        assert!(parse_usd(&serde_json::json!({ "usd": "gratis" })).is_err());
    }

    #[test]
    fn budget_refuses_without_a_cap() {
        let err = check_budget(0.01, None).unwrap_err();
        assert!(format!("{err}").contains("--max-spend-usd"), "{err}");
    }

    #[test]
    fn budget_refuses_a_cap_above_the_ceiling() {
        let err = check_budget(0.01, Some(HARD_CAP_USD + 0.01)).unwrap_err();
        assert!(format!("{err}").contains("ceiling"), "{err}");
    }

    #[test]
    fn budget_refuses_a_run_that_costs_more_than_the_cap() {
        let err = check_budget(2.0, Some(1.0)).unwrap_err();
        assert!(format!("{err}").contains("Nothing was generated"), "{err}");
    }

    #[test]
    fn budget_allows_a_run_inside_the_cap() {
        assert_eq!(check_budget(0.75, Some(1.0)).unwrap(), 0.75);
    }

    #[test]
    fn budget_refuses_a_nonsense_cap() {
        assert!(check_budget(0.01, Some(0.0)).is_err());
        assert!(check_budget(0.01, Some(-1.0)).is_err());
        assert!(check_budget(0.01, Some(f64::NAN)).is_err());
    }

    #[test]
    fn image_urls_are_found_in_both_documented_shapes() {
        let documented = serde_json::json!({
            "status": "completed",
            "images": [{ "url": "https://cdn/a.png" }, { "url": "https://cdn/b.png" }]
        });
        assert_eq!(
            image_urls(&documented),
            vec!["https://cdn/a.png", "https://cdn/b.png"]
        );

        let sdk_shape = serde_json::json!({
            "status": "completed",
            "jobs": [{ "results": { "raw": { "url": "https://cdn/c.png" } } }]
        });
        assert_eq!(image_urls(&sdk_shape), vec!["https://cdn/c.png"]);
    }

    #[test]
    fn image_urls_ignore_the_status_and_cancel_links() {
        let value = serde_json::json!({
            "status_url": "https://api/requests/1/status",
            "cancel_url": "https://api/requests/1/cancel",
            "url": "https://api/requests/1/status",
            "images": [{ "url": "https://cdn/a.png" }]
        });
        assert_eq!(image_urls(&value), vec!["https://cdn/a.png"]);
    }

    #[test]
    fn estimate_reads_the_price() {
        let transport = FakeTransport::one(200, r#"{"credits":"0.30","usd":"0.019"}"#);
        let usd = estimate(&transport, "id:secret", "m", &frame("tack")).unwrap();
        assert_eq!(usd, 0.019);
        let seen = transport.seen.borrow();
        assert_eq!(seen[0].url, format!("{API_BASE}/estimate/m"));
        assert_eq!(seen[0].method, Method::Post);
    }

    #[test]
    fn estimate_surfaces_an_api_error() {
        let transport = FakeTransport::one(404, r#"{"detail":"model_not_found"}"#);
        let err = estimate(&transport, "id:secret", "nope", &frame("tack")).unwrap_err();
        assert!(format!("{err}").contains("404"), "{err}");
    }

    #[test]
    fn submit_returns_the_status_url() {
        let transport = FakeTransport::one(
            200,
            r#"{"status":"queued","request_id":"r1","status_url":"https://api/requests/r1/status"}"#,
        );
        let url = submit(&transport, "id:secret", "m", &frame("tack")).unwrap();
        assert_eq!(url, "https://api/requests/r1/status");
    }

    #[test]
    fn poll_backs_off_and_returns_the_completed_body() {
        let transport = FakeTransport::new(vec![
            Response {
                status: 200,
                body: r#"{"status":"queued"}"#.into(),
            },
            Response {
                status: 200,
                body: r#"{"status":"processing"}"#.into(),
            },
            Response {
                status: 200,
                body: r#"{"status":"completed","images":[{"url":"https://cdn/a.png"}]}"#.into(),
            },
        ]);
        let mut slept: Vec<Duration> = Vec::new();
        let value = poll(
            &transport,
            "id:secret",
            "https://api/requests/r1/status",
            &mut |d| slept.push(d),
            10,
        )
        .unwrap();
        assert_eq!(image_urls(&value), vec!["https://cdn/a.png"]);
        assert_eq!(slept, vec![Duration::from_secs(2), Duration::from_secs(3)]);
    }

    #[test]
    fn poll_stops_on_every_terminal_status() {
        for status in TERMINAL_STATUSES {
            let transport = FakeTransport::one(200, &format!(r#"{{"status":"{status}"}}"#));
            let value = poll(&transport, "k", "https://api/s", &mut |_| {}, 3).unwrap();
            assert_eq!(value["status"], Value::String(status.into()));
        }
    }

    #[test]
    fn poll_gives_up_rather_than_spinning_forever() {
        let transport = FakeTransport::new(vec![
            Response {
                status: 200,
                body: r#"{"status":"queued"}"#.into(),
            };
            3
        ]);
        let err = poll(&transport, "k", "https://api/s", &mut |_| {}, 3).unwrap_err();
        assert!(format!("{err}").contains("gave up"), "{err}");
    }

    #[test]
    fn poll_stops_immediately_on_bad_credentials() {
        let transport = FakeTransport::one(401, "unauthorised");
        let err = poll(&transport, "k", "https://api/s", &mut |_| {}, 5).unwrap_err();
        assert!(format!("{err}").contains("401"), "{err}");
        assert_eq!(transport.seen.borrow().len(), 1, "no retry on a 401");
    }

    #[test]
    fn poll_caps_the_delay_at_ten_seconds() {
        let transport = FakeTransport::new(vec![
            Response {
                status: 200,
                body: r#"{"status":"queued"}"#.into(),
            };
            12
        ]);
        let mut slept: Vec<Duration> = Vec::new();
        let _ = poll(&transport, "k", "https://api/s", &mut |d| slept.push(d), 12);
        assert!(slept.iter().all(|d| *d <= Duration::from_secs(10)));
        assert_eq!(*slept.last().unwrap(), Duration::from_secs(10));
    }

    #[test]
    fn ledger_reports_what_is_done_and_what_was_spent() {
        let text = concat!(
            "{\"id\":\"tack\",\"usd\":0.019,\"files\":[\"tack_0.png\"]}\n",
            "{\"id\":\"rail\",\"usd\":0.021,\"files\":[\"rail_0.png\"]}\n"
        );
        let ids = ledger_ids(text);
        assert!(ids.contains("tack") && ids.contains("rail"));
        assert!((ledger_spent(text) - 0.040).abs() < 1e-9);
    }

    #[test]
    fn a_torn_ledger_line_does_not_stop_the_rest_being_read() {
        let text =
            "{\"id\":\"tack\",\"usd\":0.01}\n{ this is not json\n{\"id\":\"rail\",\"usd\":0.01}\n";
        assert_eq!(ledger_ids(text).len(), 2);
    }

    #[test]
    fn pending_skips_what_the_ledger_already_has() {
        let spec = parse_spec(
            r#"{"model":"m","out_dir":"o","frames":[
                 {"id":"a","subject":"x"},{"id":"b","subject":"y"},{"id":"c","subject":"z"}]}"#,
        )
        .unwrap();
        let done = ledger_ids("{\"id\":\"b\",\"usd\":0.01}\n");
        let todo: Vec<&str> = pending(&spec, &done)
            .iter()
            .map(|f| f.id.as_str())
            .collect();
        assert_eq!(todo, vec!["a", "c"]);
    }

    #[test]
    fn ledger_round_trips_through_a_file() {
        let dir = std::env::temp_dir().join("fragr-spritegen-ledger");
        let _ = std::fs::remove_dir_all(&dir);
        append_ledger(
            &dir,
            &LedgerEntry {
                id: "tack".into(),
                usd: 0.019,
                files: vec!["tack_0.png".into()],
            },
        )
        .unwrap();
        append_ledger(
            &dir,
            &LedgerEntry {
                id: "rail".into(),
                usd: 0.021,
                files: vec!["rail_0.png".into()],
            },
        )
        .unwrap();
        let text = read_ledger(&dir);
        assert_eq!(ledger_ids(&text).len(), 2);
        assert!((ledger_spent(&text) - 0.040).abs() < 1e-9);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_ledger_reads_as_empty_rather_than_failing() {
        let dir = std::env::temp_dir().join("fragr-spritegen-no-ledger");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(ledger_ids(&read_ledger(&dir)).is_empty());
    }

    #[test]
    fn file_names_take_the_extension_from_the_url() {
        assert_eq!(file_name("tack", 0, "https://cdn/x.png"), "tack_0.png");
        assert_eq!(file_name("tack", 1, "https://cdn/x.webp"), "tack_1.webp");
        assert_eq!(
            file_name("tack", 2, "https://cdn/x.jpg?sig=a"),
            "tack_2.jpg"
        );
        // No usable extension falls back rather than inventing one.
        assert_eq!(file_name("tack", 3, "https://cdn/x"), "tack_3.png");
    }

    #[test]
    fn credential_is_read_from_a_dotenv_file() {
        let dir = std::env::temp_dir().join("fragr-spritegen-env");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        std::fs::write(&path, "elevenlabs=sk_x\nhiggsfield=abc-123:def-456\n").unwrap();
        assert_eq!(read_dotenv_credential(&path).unwrap(), "abc-123:def-456");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_credential_without_a_secret_half_is_refused() {
        let dir = std::env::temp_dir().join("fragr-spritegen-env-bad");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        std::fs::write(&path, "higgsfield=only-an-id\n").unwrap();
        assert!(read_dotenv_credential(&path).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
