//! Command line front end for developer-only audio generation.
//!
//! Argument parsing and the real HTTP transport live here. Everything else is in
//! the library so it is tested without touching the network.

use clap::{Parser, Subcommand};
use fragr_audiogen::{
    parse_spec, read_dotenv_key, resolve_api_key, run_command, Command, Error, Job, Method,
    MusicParams, Request, Response, RunOptions, SfxParams, Transport, TtsParams, API_KEY_ENV,
    DEFAULT_BASE_URL, DEFAULT_MUSIC_FORMAT, DEFAULT_MUSIC_MODEL, DEFAULT_SFX_FORMAT,
    DEFAULT_TTS_FORMAT, DEFAULT_TTS_MODEL,
};
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "fragr-audiogen",
    version,
    about = "Developer-only sound effect and music generation for fragr through the ElevenLabs API. Never runs in CI or in the game."
)]
struct Cli {
    /// File containing the API key. Overrides ELEVENLABS_API_KEY. Keep it out of git.
    #[arg(long, global = true)]
    api_key_file: Option<PathBuf>,
    /// Dotenv file consulted when ELEVENLABS_API_KEY is unset (accepts elevenlabs=...).
    #[arg(long, global = true, default_value = ".env")]
    env_file: PathBuf,
    /// API host.
    #[arg(long, global = true, default_value = DEFAULT_BASE_URL)]
    base_url: String,
    /// Output directory. Defaults to client/assets/audio, or the spec's out_dir for batch.
    #[arg(long, global = true)]
    out_dir: Option<PathBuf>,
    /// Replace files that already exist.
    #[arg(long, global = true)]
    overwrite: bool,
    /// Print the requests without calling the API or writing files.
    #[arg(long, global = true)]
    dry_run: bool,
    /// Refuse to start when the estimated credits for the run exceed this number.
    #[arg(long, global = true)]
    max_credits: Option<u64>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Generate one sound effect.
    Sfx {
        /// Asset name without extension, for example fire_rail or ui/click.
        #[arg(long)]
        name: String,
        /// What the sound should be. Concrete and physical beats poetic.
        #[arg(long)]
        prompt: String,
        /// Duration in seconds (0.5 to 30). Omit to let the model choose.
        #[arg(long)]
        seconds: Option<f64>,
        /// Prompt influence from 0 to 1. Higher follows the prompt more literally.
        #[arg(long)]
        influence: Option<f64>,
        /// Ask for a seamless loop.
        #[arg(long = "loop")]
        looping: bool,
        /// ElevenLabs output format. pcm_* becomes .wav, mp3_* becomes .mp3.
        #[arg(long, default_value = DEFAULT_SFX_FORMAT)]
        format: String,
    },
    /// Generate one music track.
    Music {
        /// Asset name without extension, for example music/match_01.
        #[arg(long)]
        name: String,
        /// Display title recorded in the manifest.
        #[arg(long)]
        title: Option<String>,
        /// Style, tempo, mood, instrumentation.
        #[arg(long)]
        prompt: String,
        /// Length in milliseconds (3000 to 600000). Omit to let the model choose.
        #[arg(long)]
        length_ms: Option<u32>,
        /// Music model id.
        #[arg(long, default_value = DEFAULT_MUSIC_MODEL)]
        model: String,
        /// No vocals.
        #[arg(long)]
        instrumental: bool,
        /// ElevenLabs output format.
        #[arg(long, default_value = DEFAULT_MUSIC_FORMAT)]
        format: String,
    },
    /// Generate every item in a JSON spec (see tools/audiogen/specs).
    Batch {
        /// Path to the spec file.
        #[arg(long)]
        spec: PathBuf,
        /// Only generate the item with this name.
        #[arg(long)]
        only: Option<String>,
        /// Only generate items whose name starts with this prefix, for example radio/rock/.
        #[arg(long)]
        prefix: Option<String>,
        /// Generate at most this many new files in this run (skipped files do not count).
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Speak one line with a voice (news bulletins, Host lines).
    Tts {
        /// Asset name without extension, for example radio/news/generic-01-count.
        #[arg(long)]
        name: String,
        /// Text to speak. Square-bracket delivery tags such as [sighs] work on eleven_v3.
        #[arg(long)]
        text: String,
        /// ElevenLabs voice id (see the voices command).
        #[arg(long)]
        voice: String,
        /// Speech model id.
        #[arg(long, default_value = DEFAULT_TTS_MODEL)]
        model: String,
        /// Stability from 0 (creative) to 1 (robust).
        #[arg(long)]
        stability: Option<f64>,
        /// ElevenLabs output format.
        #[arg(long, default_value = DEFAULT_TTS_FORMAT)]
        format: String,
        /// Display title recorded in the manifest.
        #[arg(long)]
        title: Option<String>,
    },
    /// Turn a scripts file (see specs/radio-news-scripts.json) into a batch spec with voices cast.
    Scripts {
        /// Path to the scripts file.
        #[arg(long)]
        scripts: PathBuf,
        /// Cast a speaker: --voice host=<id>, --voice caller=<id1>,<id2>. Repeat per speaker.
        #[arg(long = "voice", value_parser = parse_voice_assignment, required = true)]
        voices: Vec<(String, Vec<String>)>,
        /// Stability for every spoken item, 0 (creative) to 1 (robust).
        #[arg(long, default_value_t = 0.5)]
        stability: f64,
        /// Where to write the batch spec.
        #[arg(long)]
        out: PathBuf,
    },
    /// List the account's default voices with ids for casting.
    Voices,
    /// Show remaining credits for the configured key.
    Quota,
}

/// `speaker=id` or `speaker=id1,id2` for the scripts command.
fn parse_voice_assignment(raw: &str) -> Result<(String, Vec<String>), String> {
    let (speaker, ids) = raw
        .split_once('=')
        .ok_or_else(|| format!("expected speaker=<voice_id>, got '{raw}'"))?;
    let speaker = speaker.trim();
    let ids: Vec<String> = ids
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect();
    if speaker.is_empty() || ids.is_empty() {
        return Err(format!("expected speaker=<voice_id>, got '{raw}'"));
    }
    Ok((speaker.to_string(), ids))
}

fn to_command(cmd: Cmd) -> Result<Command, Error> {
    Ok(match cmd {
        Cmd::Sfx {
            name,
            prompt,
            seconds,
            influence,
            looping,
            format,
        } => Command::Generate {
            name,
            job: Job::Sfx(SfxParams {
                prompt,
                seconds,
                influence,
                looping,
                format,
                title: None,
            }),
        },
        Cmd::Music {
            name,
            title,
            prompt,
            length_ms,
            model,
            instrumental,
            format,
        } => Command::Generate {
            name,
            job: Job::Music(MusicParams {
                prompt,
                length_ms,
                model,
                instrumental,
                format,
                title,
            }),
        },
        Cmd::Batch {
            spec,
            only,
            prefix,
            limit,
        } => {
            let text = std::fs::read_to_string(&spec)?;
            Command::Batch {
                spec: parse_spec(&text)?,
                only,
                prefix,
                limit,
            }
        }
        Cmd::Tts {
            name,
            text,
            voice,
            model,
            stability,
            format,
            title,
        } => Command::Generate {
            name,
            job: Job::Tts(TtsParams {
                text,
                voice_id: voice,
                model,
                stability,
                format,
                title,
                lines: Vec::new(),
            }),
        },
        Cmd::Scripts {
            scripts,
            voices,
            stability,
            out,
        } => {
            let scripts_json = std::fs::read_to_string(&scripts)?;
            let mut cast = fragr_audiogen::VoiceCast::new();
            for (speaker, ids) in voices {
                cast.entry(speaker).or_default().extend(ids);
            }
            Command::Scripts {
                scripts_json,
                cast,
                stability,
                out,
            }
        }
        Cmd::Voices => Command::Voices,
        Cmd::Quota => Command::Quota,
    })
}

struct HttpTransport {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl HttpTransport {
    fn new(base_url: &str) -> Result<Self, Error> {
        // Music generation can take minutes for long tracks; keep the timeout generous.
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .map_err(|err| Error::Transport(err.to_string()))?;
        Ok(HttpTransport {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
}

impl Transport for HttpTransport {
    fn send(&self, api_key: &str, request: &Request) -> Result<Response, Error> {
        let url = format!("{}{}", self.base_url, request.path);
        let mut builder = match request.method {
            Method::Get => self.client.get(&url),
            Method::Post => self.client.post(&url),
        };
        builder = builder.header("xi-api-key", api_key).query(&request.query);
        if let Some(body) = &request.body {
            builder = builder.json(body);
        }
        let response = builder
            .send()
            .map_err(|err| Error::Transport(err.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|err| Error::Transport(err.to_string()))?
            .to_vec();
        Ok(Response { status, body })
    }
}

fn run(cli: Cli, out: &mut dyn std::io::Write) -> Result<(), Error> {
    // Dry runs never need a key. Quota always does.
    // Dry runs and the scripts converter never touch the network. Quota and voices always do.
    let needs_key = (!cli.dry_run && !matches!(cli.command, Cmd::Scripts { .. }))
        || matches!(cli.command, Cmd::Quota | Cmd::Voices);
    let api_key = if needs_key {
        let from_env = match std::env::var(API_KEY_ENV) {
            Ok(value) => Some(value),
            Err(_) => read_dotenv_key(&cli.env_file)?,
        };
        resolve_api_key(from_env, cli.api_key_file.as_deref())?
    } else {
        String::new()
    };
    let command = to_command(cli.command)?;
    let options = RunOptions {
        out_dir: cli.out_dir,
        overwrite: cli.overwrite,
        dry_run: cli.dry_run,
        max_credits: cli.max_credits,
    };
    let transport = HttpTransport::new(&cli.base_url)?;
    run_command(
        &command,
        &options,
        &transport,
        &api_key,
        out,
        &mut |delay| std::thread::sleep(delay),
    )
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let mut stdout = std::io::stdout();
    if let Err(err) = run(cli, &mut stdout) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sfx_arguments() {
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "--out-dir",
            "out",
            "--overwrite",
            "sfx",
            "--name",
            "fire_rail",
            "--prompt",
            "rail shot",
            "--seconds",
            "0.8",
            "--influence",
            "0.6",
            "--loop",
        ])
        .unwrap();
        assert_eq!(cli.out_dir, Some(PathBuf::from("out")));
        assert!(cli.overwrite);
        assert!(!cli.dry_run);
        match to_command(cli.command).unwrap() {
            Command::Generate {
                name,
                job: Job::Sfx(params),
            } => {
                assert_eq!(name, "fire_rail");
                assert_eq!(params.prompt, "rail shot");
                assert_eq!(params.seconds, Some(0.8));
                assert_eq!(params.influence, Some(0.6));
                assert!(params.looping);
                assert_eq!(params.format, DEFAULT_SFX_FORMAT);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parses_music_arguments() {
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "music",
            "--name",
            "music/match_01",
            "--prompt",
            "industrial arena loop",
            "--length-ms",
            "45000",
            "--instrumental",
        ])
        .unwrap();
        match to_command(cli.command).unwrap() {
            Command::Generate {
                name,
                job: Job::Music(params),
            } => {
                assert_eq!(name, "music/match_01");
                assert_eq!(params.length_ms, Some(45_000));
                assert_eq!(params.model, DEFAULT_MUSIC_MODEL);
                assert_eq!(params.format, DEFAULT_MUSIC_FORMAT);
                assert!(params.instrumental);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parses_batch_and_quota() {
        let dir = std::env::temp_dir().join("fragr-audiogen-main-batch");
        std::fs::create_dir_all(&dir).unwrap();
        let spec = dir.join("spec.json");
        std::fs::write(
            &spec,
            r#"{"items": [{"kind": "sfx", "name": "a", "prompt": "p"}]}"#,
        )
        .unwrap();
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "batch",
            "--spec",
            spec.to_str().unwrap(),
            "--only",
            "a",
        ])
        .unwrap();
        match to_command(cli.command).unwrap() {
            Command::Batch { spec, only, .. } => {
                assert_eq!(spec.items.len(), 1);
                assert_eq!(only.as_deref(), Some("a"));
            }
            other => panic!("unexpected {other:?}"),
        }
        let missing =
            Cli::try_parse_from(["fragr-audiogen", "batch", "--spec", "does-not-exist.json"])
                .unwrap();
        assert!(matches!(to_command(missing.command), Err(Error::Io(_))));
        let quota = Cli::try_parse_from(["fragr-audiogen", "quota"]).unwrap();
        assert_eq!(to_command(quota.command).unwrap(), Command::Quota);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parses_tts_and_voices() {
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "tts",
            "--name",
            "radio/news/generic-01-count",
            "--text",
            "[sighs] Good evening.",
            "--voice",
            "JBFqnCBsd6RMkjVDRZzb",
            "--stability",
            "0.4",
            "--title",
            "Count",
        ])
        .unwrap();
        match to_command(cli.command).unwrap() {
            Command::Generate {
                name,
                job: Job::Tts(params),
            } => {
                assert_eq!(name, "radio/news/generic-01-count");
                assert_eq!(params.voice_id, "JBFqnCBsd6RMkjVDRZzb");
                assert_eq!(params.model, DEFAULT_TTS_MODEL);
                assert_eq!(params.format, DEFAULT_TTS_FORMAT);
                assert_eq!(params.stability, Some(0.4));
                assert_eq!(params.title.as_deref(), Some("Count"));
            }
            other => panic!("unexpected {other:?}"),
        }
        let voices = Cli::try_parse_from(["fragr-audiogen", "voices"]).unwrap();
        assert_eq!(to_command(voices.command).unwrap(), Command::Voices);
    }

    #[test]
    fn parses_scripts_command() {
        let dir = std::env::temp_dir().join("fragr-audiogen-main-scripts");
        std::fs::create_dir_all(&dir).unwrap();
        let scripts = dir.join("scripts.json");
        std::fs::write(
            &scripts,
            r#"{"items": [{"name": "radio/news/generic-01-a", "class": "generic", "speaker": "host", "title": "A", "text": "Hello."}]}"#,
        )
        .unwrap();
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "scripts",
            "--scripts",
            scripts.to_str().unwrap(),
            "--voice",
            "host=h1",
            "--voice",
            "caller=c1,c2",
            "--voice",
            "caller=c3",
            "--stability",
            "0.4",
            "--out",
            dir.join("out.json").to_str().unwrap(),
        ])
        .unwrap();
        match to_command(cli.command).unwrap() {
            Command::Scripts {
                cast, stability, ..
            } => {
                assert_eq!(cast["host"], vec!["h1".to_string()]);
                assert_eq!(
                    cast["caller"],
                    vec!["c1".to_string(), "c2".into(), "c3".into()]
                );
                assert_eq!(stability, 0.4);
            }
            other => panic!("unexpected {other:?}"),
        }
        assert!(parse_voice_assignment("host").is_err());
        assert!(parse_voice_assignment("=id").is_err());
        assert!(parse_voice_assignment("host=").is_err());
        assert_eq!(
            parse_voice_assignment(" tina = a , b ").unwrap(),
            ("tina".to_string(), vec!["a".to_string(), "b".to_string()])
        );
        // The converter needs no key: a missing key file must not stop it.
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "--api-key-file",
            "definitely-missing.key",
            "scripts",
            "--scripts",
            scripts.to_str().unwrap(),
            "--voice",
            "host=h1",
            "--out",
            dir.join("out2.json").to_str().unwrap(),
        ])
        .unwrap();
        let mut out = Vec::new();
        run(cli, &mut out).unwrap();
        assert!(String::from_utf8(out).unwrap().contains("with 1 items"));
        assert!(dir.join("out2.json").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rejects_missing_required_arguments() {
        assert!(Cli::try_parse_from(["fragr-audiogen", "sfx", "--name", "x"]).is_err());
        assert!(Cli::try_parse_from(["fragr-audiogen"]).is_err());
    }

    #[test]
    fn http_transport_trims_base_url() {
        let transport = HttpTransport::new("https://example.test/").unwrap();
        assert_eq!(transport.base_url, "https://example.test");
    }

    #[test]
    fn dry_run_needs_no_key_and_writes_nothing() {
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "--dry-run",
            "--api-key-file",
            "definitely-missing.key",
            "--out-dir",
            "unused-dir",
            "sfx",
            "--name",
            "hit",
            "--prompt",
            "thud",
        ])
        .unwrap();
        let mut out = Vec::new();
        run(cli, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("dry-run hit -> "));
        assert!(!PathBuf::from("unused-dir").exists());
    }

    #[test]
    fn live_run_reads_a_dotenv_file() {
        let dir = std::env::temp_dir().join("fragr-audiogen-main-dotenv");
        std::fs::create_dir_all(&dir).unwrap();
        let env_file = dir.join(".env");
        std::fs::write(&env_file, "elevenlabs=sk_test_only\n").unwrap();
        // An unreachable host proves the key resolved and the request was attempted.
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "--env-file",
            env_file.to_str().unwrap(),
            "--base-url",
            "http://127.0.0.1:9",
            "quota",
        ])
        .unwrap();
        let err = run(cli, &mut Vec::new()).unwrap_err();
        assert!(matches!(err, Error::Transport(_)), "unexpected {err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn live_run_requires_a_readable_key() {
        let cli = Cli::try_parse_from([
            "fragr-audiogen",
            "--api-key-file",
            "definitely-missing.key",
            "quota",
        ])
        .unwrap();
        let err = run(cli, &mut Vec::new()).unwrap_err();
        assert!(matches!(err, Error::Io(_)));
    }
}
