//! Command line for the decision-brain agent.
//!
//! `play` joins a server and fights; `ask` sends one decision for a state you
//! type (or prints the request with `--dry-run`); `spend` shows the ledger;
//! `key` reads an OpenRouter key's own limit. Paid providers refuse to start
//! until `--max-spend-usd` names a cap.

use clap::{Args, Parser, Subcommand};
use fragr_brain::bot::{run_bot, BotConfig};
use fragr_brain::budget::{Budget, Caps, Pricing};
use fragr_brain::decision::tactical_questions;
use fragr_brain::dotenv::resolve_key;
use fragr_brain::provider::{
    decide, decision_request, key_request, parse_key_status, HttpTransport, Provider, Transport,
};
use fragr_brain::Error;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "fragr-brain",
    version,
    about = "A fragr agent whose intent comes from a decision model (TypeSafe Jev, natively or through OpenRouter) under a hard spend cap; local rules play for free."
)]
struct Cli {
    #[command(flatten)]
    common: Common,
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Debug, Clone)]
struct Common {
    /// Decision source: local (free, default), typesafe, or openrouter.
    #[arg(long, default_value = "local", global = true)]
    provider: String,
    /// Model id; defaults to jev-latest (typesafe) or typesafe/jev-1.13 (openrouter).
    #[arg(long, global = true)]
    model: Option<String>,
    /// Dollars this run may spend. Zero (the default) means no paid call at all.
    #[arg(long, default_value_t = 0.0, global = true)]
    max_spend_usd: f64,
    /// Dollars the ledger may reach across every run.
    #[arg(long, global = true)]
    max_total_usd: Option<f64>,
    /// Paid calls this run may make, whatever they cost.
    #[arg(long, global = true)]
    max_calls: Option<u64>,
    /// Ledger of every paid call; totals carry across runs.
    #[arg(long, default_value = ".agents/spend/brain.json", global = true)]
    ledger: PathBuf,
    /// Keep the ledger in memory only.
    #[arg(long, global = true)]
    no_ledger: bool,
    /// Dollars per million input tokens used for estimates and settlement.
    #[arg(long, default_value_t = fragr_brain::budget::JEV_INPUT_PER_MILLION, global = true)]
    price_input_per_million: f64,
    /// Dollars per million output tokens.
    #[arg(long, default_value_t = fragr_brain::budget::JEV_OUTPUT_PER_MILLION, global = true)]
    price_output_per_million: f64,
    /// Read keys from this .env file when the environment has none.
    #[arg(long, default_value = ".env", global = true)]
    env_file: PathBuf,
    /// Read the key from this file instead of the environment.
    #[arg(long, global = true)]
    api_key_file: Option<PathBuf>,
    /// Per-call HTTP timeout.
    #[arg(long, default_value_t = 2000, global = true)]
    timeout_ms: u64,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Join a server as an agent and fight.
    Play {
        #[arg(long, default_value = "ws://127.0.0.1:6767")]
        server: String,
        /// Display name; falls back to FRAGR_AGENT_NAME, then "Brain".
        #[arg(long)]
        name: Option<String>,
        /// Decisions per second (0.1 to 5).
        #[arg(long, default_value_t = 3.0)]
        decision_hz: f64,
        /// Below this confidence the local rules decide the stance.
        #[arg(long, default_value_t = 0.65)]
        confidence_floor: f64,
        /// Leave after this many seconds.
        #[arg(long)]
        max_seconds: Option<u64>,
    },
    /// Send one decision for a state string and print the answers.
    Ask {
        /// The state text; use the same line format the bot sends.
        #[arg(long)]
        state: String,
        /// Print the request (token masked) and the estimate, send nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Print the ledger totals.
    Spend,
    /// Print the calling key's limit and usage (OpenRouter).
    Key,
}

fn parse_provider(text: &str) -> Result<Provider, Error> {
    Provider::parse(text).ok_or_else(|| {
        Error::InvalidArgument(format!(
            "unknown provider {text:?}; use local, typesafe, or openrouter"
        ))
    })
}

fn budget_from(common: &Common) -> Result<Budget, Error> {
    if common.max_spend_usd < 0.0 || common.max_total_usd.is_some_and(|c| c < 0.0) {
        return Err(Error::InvalidArgument(
            "caps cannot be negative".to_string(),
        ));
    }
    if common.price_input_per_million < 0.0 || common.price_output_per_million < 0.0 {
        return Err(Error::InvalidArgument(
            "prices cannot be negative".to_string(),
        ));
    }
    let caps = Caps {
        run_usd: common.max_spend_usd,
        total_usd: common.max_total_usd,
        run_calls: common.max_calls,
    };
    let pricing = Pricing {
        input_per_million: common.price_input_per_million,
        output_per_million: common.price_output_per_million,
    };
    if common.no_ledger {
        Ok(Budget::new(caps, pricing))
    } else {
        Budget::with_ledger(caps, pricing, &common.ledger)
    }
}

fn key_for(common: &Common, provider: Provider) -> Result<Option<String>, Error> {
    if !provider.is_paid() {
        return Ok(None);
    }
    let env = |name: &str| std::env::var(name).ok();
    resolve_key(
        provider.key_names(),
        &env,
        Some(&common.env_file),
        common.api_key_file.as_deref(),
    )
}

fn require_key(common: &Common, provider: Provider) -> Result<String, Error> {
    key_for(common, provider)?.ok_or_else(|| Error::MissingApiKey(provider.key_names().join(", ")))
}

fn resolve_name(name: Option<&str>, fallback: &str) -> String {
    name.map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var("FRAGR_AGENT_NAME")
                .ok()
                .filter(|n| !n.trim().is_empty())
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn run(cli: Cli, transport: Arc<dyn Transport>, out: &mut dyn std::io::Write) -> Result<(), Error> {
    let provider = parse_provider(&cli.common.provider)?;
    let model = cli
        .common
        .model
        .clone()
        .unwrap_or_else(|| provider.default_model().to_string());
    match cli.command {
        Command::Play {
            server,
            name,
            decision_hz,
            confidence_floor,
            max_seconds,
        } => {
            let budget = budget_from(&cli.common)?;
            if provider.is_paid() {
                budget.check(0.0)?;
            }
            let api_key = key_for(&cli.common, provider)?;
            if provider.is_paid() && api_key.is_none() {
                return Err(Error::MissingApiKey(provider.key_names().join(", ")));
            }
            let config = BotConfig {
                server_url: server,
                name: resolve_name(name.as_deref(), "Brain"),
                provider,
                model,
                api_key,
                decision_hz,
                confidence_floor,
                max_seconds,
            };
            let budget = Arc::new(Mutex::new(budget));
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|err| Error::Transport(format!("runtime: {err}")))?;
            let summary = runtime.block_on(run_bot(
                config,
                transport,
                budget,
                Arc::new(AtomicBool::new(false)),
            ))?;
            let text = serde_json::to_string_pretty(&summary)
                .map_err(|err| Error::Malformed(err.to_string()))?;
            writeln!(out, "{text}")?;
            Ok(())
        }
        Command::Ask { state, dry_run } => {
            if !provider.is_paid() {
                return Err(Error::InvalidArgument(
                    "ask needs a paid provider (typesafe or openrouter)".to_string(),
                ));
            }
            let budget = budget_from(&cli.common)?;
            let api_key = if dry_run {
                key_for(&cli.common, provider)?.unwrap_or_else(|| "dry-run".to_string())
            } else {
                require_key(&cli.common, provider)?
            };
            let request =
                decision_request(provider, &model, &api_key, &state, &tactical_questions())?;
            let estimate = fragr_brain::provider::estimate_cost(&request, &budget.pricing);
            if dry_run {
                let shown = serde_json::json!({
                    "request": request.redacted(),
                    "estimated_usd": estimate,
                    "run_cap_usd": budget.caps.run_usd,
                    "ledger_total_usd": budget.total_usd(),
                });
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string_pretty(&shown).unwrap_or_default()
                )?;
                return Ok(());
            }
            let budget = Mutex::new(budget);
            let decision = decide(transport.as_ref(), &budget, provider, &model, &request)?;
            let guard = budget.lock().unwrap_or_else(|p| p.into_inner());
            let shown = serde_json::json!({
                "model": decision.response.model,
                "answers": answers_json(&decision.response.answers),
                "charge": decision.charge,
                "run_usd": guard.run_usd(),
                "ledger_total_usd": guard.total_usd(),
            });
            writeln!(
                out,
                "{}",
                serde_json::to_string_pretty(&shown).unwrap_or_default()
            )?;
            Ok(())
        }
        Command::Spend => {
            let budget = budget_from(&cli.common)?;
            let ledger = budget.ledger();
            let shown = serde_json::json!({
                "ledger": budget.ledger_path().map(|p| p.display().to_string()),
                "calls": ledger.calls(),
                "total_usd": ledger.total_usd(),
                "run_cap_usd": budget.caps.run_usd,
                "total_cap_usd": budget.caps.total_usd,
            });
            writeln!(
                out,
                "{}",
                serde_json::to_string_pretty(&shown).unwrap_or_default()
            )?;
            Ok(())
        }
        Command::Key => {
            let api_key = require_key(&cli.common, provider)?;
            let request = key_request(provider, &api_key)?;
            let response = transport.send(&request)?;
            let status = parse_key_status(&response)?;
            writeln!(
                out,
                "{}",
                serde_json::to_string_pretty(&status).unwrap_or_default()
            )?;
            if status.limit.is_none() {
                writeln!(
                    out,
                    "note: this key has no provider-side limit; set one in the OpenRouter dashboard as a backstop"
                )?;
            }
            Ok(())
        }
    }
}

fn answers_json(
    answers: &std::collections::BTreeMap<String, fragr_brain::decision::Answer>,
) -> serde_json::Value {
    use fragr_brain::decision::Answer;
    let mut out = serde_json::Map::new();
    for (name, answer) in answers {
        let value = match answer {
            Answer::Noul { noul } => serde_json::json!({"type": "noul", "noul": noul}),
            Answer::Choice {
                choice,
                confidence,
                probabilities,
            } => serde_json::json!({
                "type": "choice", "choice": choice, "confidence": confidence, "probabilities": probabilities
            }),
            Answer::Score {
                score,
                confidence,
                probabilities,
                ..
            } => serde_json::json!({
                "type": "score", "score": score, "confidence": confidence, "probabilities": probabilities
            }),
        };
        out.insert(name.clone(), value);
    }
    serde_json::Value::Object(out)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let transport = match HttpTransport::new(Duration::from_millis(cli.common.timeout_ms)) {
        Ok(transport) => Arc::new(transport),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    };
    let mut stdout = std::io::stdout();
    if let Err(err) = run(cli, transport, &mut stdout) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragr_brain::provider::{HttpRequest, HttpResponse};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Scripted {
        body: serde_json::Value,
        status: u16,
        calls: AtomicUsize,
    }

    impl Transport for Scripted {
        fn send(&self, _request: &HttpRequest) -> Result<HttpResponse, Error> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(HttpResponse {
                status: self.status,
                body: self.body.to_string().into_bytes(),
            })
        }
    }

    fn scripted(status: u16, body: serde_json::Value) -> Arc<Scripted> {
        Arc::new(Scripted {
            body,
            status,
            calls: AtomicUsize::new(0),
        })
    }

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(std::iter::once("fragr-brain").chain(args.iter().copied()))
            .expect("args parse")
    }

    fn temp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("fragr-brain-main-{name}-{}", std::process::id()))
    }

    fn answers() -> serde_json::Value {
        serde_json::json!({
            "model": "jev-1.13.0",
            "answers": {
                "stance": {"type": "choice", "choice": "hold_angle", "confidence": 0.7},
                "danger": {"type": "score", "score": 1.0},
                "extra": {"type": "noul", "noul": 0.2}
            },
            "usage": {"input_tokens": 100, "output_tokens": 0}
        })
    }

    #[test]
    fn args_parse_every_command() {
        let cli = parse(&[
            "play",
            "--server",
            "ws://h:1",
            "--name",
            "X",
            "--max-seconds",
            "5",
        ]);
        assert!(
            matches!(cli.command, Command::Play { ref server, ref name, max_seconds: Some(5), .. } if server == "ws://h:1" && name.as_deref() == Some("X"))
        );
        assert_eq!(cli.common.provider, "local");
        assert_eq!(cli.common.max_spend_usd, 0.0);
        let cli = parse(&[
            "--provider",
            "typesafe",
            "--max-spend-usd",
            "5",
            "ask",
            "--state",
            "s",
            "--dry-run",
        ]);
        assert!(matches!(cli.command, Command::Ask { dry_run: true, .. }));
        assert_eq!(cli.common.max_spend_usd, 5.0);
        assert!(matches!(parse(&["spend"]).command, Command::Spend));
        assert!(matches!(parse(&["key"]).command, Command::Key));
        assert!(Cli::try_parse_from(["fragr-brain"]).is_err());
        assert_eq!(parse_provider("openrouter").unwrap(), Provider::OpenRouter);
        assert!(parse_provider("cloud").is_err());
        assert_eq!(resolve_name(Some(" Z "), "Brain"), "Z");
        assert_eq!(resolve_name(Some("  "), "Brain"), "Brain");
    }

    #[test]
    fn budget_from_validates_and_loads_ledger() {
        let mut cli = parse(&["--no-ledger", "spend"]);
        let budget = budget_from(&cli.common).unwrap();
        assert_eq!(budget.ledger_path(), None);
        cli.common.max_spend_usd = -1.0;
        assert!(budget_from(&cli.common).is_err());
        cli.common.max_spend_usd = 1.0;
        cli.common.price_input_per_million = -1.0;
        assert!(budget_from(&cli.common).is_err());
        let ledger = temp("ledger.json");
        let _ = std::fs::remove_file(&ledger);
        let cli = parse(&["--ledger", ledger.to_str().unwrap(), "spend"]);
        let budget = budget_from(&cli.common).unwrap();
        assert_eq!(budget.ledger_path(), Some(ledger.as_path()));
        let mut out = Vec::new();
        run(cli, scripted(200, answers()), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"calls\": 0"));
    }

    #[test]
    fn play_refuses_paid_without_cap_or_key() {
        let env_file = temp("empty.env");
        std::fs::write(&env_file, "# nothing\n").unwrap();
        let cli = parse(&[
            "--provider",
            "typesafe",
            "--no-ledger",
            "--env-file",
            env_file.to_str().unwrap(),
            "play",
            "--max-seconds",
            "1",
        ]);
        let mut out = Vec::new();
        let err = run(cli, scripted(200, answers()), &mut out).unwrap_err();
        assert!(matches!(err, Error::Budget(_)), "{err}");
        let cli = parse(&[
            "--provider",
            "typesafe",
            "--no-ledger",
            "--max-spend-usd",
            "1",
            "--env-file",
            env_file.to_str().unwrap(),
            "play",
            "--max-seconds",
            "1",
        ]);
        let err = run(cli, scripted(200, answers()), &mut out).unwrap_err();
        assert!(matches!(err, Error::MissingApiKey(_)), "{err}");
        let _ = std::fs::remove_file(env_file);
    }

    #[test]
    fn play_local_fails_cleanly_when_no_server() {
        let cli = parse(&[
            "--no-ledger",
            "play",
            "--server",
            "ws://127.0.0.1:9",
            "--max-seconds",
            "1",
        ]);
        let mut out = Vec::new();
        let err = run(cli, scripted(200, answers()), &mut out).unwrap_err();
        assert!(matches!(err, Error::Transport(_)), "{err}");
    }

    #[test]
    fn ask_dry_run_sends_nothing_and_masks_the_key() {
        let env_file = temp("key.env");
        std::fs::write(&env_file, "typesafe=sk_secret\n").unwrap();
        let transport = scripted(200, answers());
        let cli = parse(&[
            "--provider",
            "typesafe",
            "--no-ledger",
            "--env-file",
            env_file.to_str().unwrap(),
            "ask",
            "--state",
            "SELF hp=10",
            "--dry-run",
        ]);
        let mut out = Vec::new();
        run(cli, transport.clone(), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("Bearer ***"));
        assert!(!text.contains("sk_secret"));
        assert!(text.contains("estimated_usd"));
        assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
        let cli = parse(&["--no-ledger", "ask", "--state", "x"]);
        assert!(
            run(cli, transport.clone(), &mut Vec::new()).is_err(),
            "local cannot ask"
        );
        let _ = std::fs::remove_file(env_file);
    }

    #[test]
    fn ask_sends_under_a_cap_and_reports_the_charge() {
        let env_file = temp("key2.env");
        std::fs::write(&env_file, "OPENROUTER_API_KEY=sk_or\n").unwrap();
        let transport = scripted(200, answers());
        let cli = parse(&[
            "--provider",
            "openrouter",
            "--no-ledger",
            "--max-spend-usd",
            "0.01",
            "--env-file",
            env_file.to_str().unwrap(),
            "ask",
            "--state",
            "SELF hp=10",
        ]);
        let mut out = Vec::new();
        run(cli, transport.clone(), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"choice\": \"hold_angle\""));
        assert!(text.contains("\"noul\": 0.2"));
        assert!(text.contains("\"score\": 1.0"));
        assert!(text.contains("\"estimated_usd\""));
        assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
        let cli = parse(&[
            "--provider",
            "openrouter",
            "--no-ledger",
            "--env-file",
            env_file.to_str().unwrap(),
            "ask",
            "--state",
            "SELF hp=10",
        ]);
        let err = run(cli, transport.clone(), &mut Vec::new()).unwrap_err();
        assert!(matches!(err, Error::Budget(_)));
        assert_eq!(
            transport.calls.load(Ordering::SeqCst),
            1,
            "refused calls are not sent"
        );
        let failing = scripted(429, serde_json::json!({"error": {"message": "slow down"}}));
        let cli = parse(&[
            "--provider",
            "openrouter",
            "--no-ledger",
            "--max-spend-usd",
            "1",
            "--env-file",
            env_file.to_str().unwrap(),
            "ask",
            "--state",
            "s",
        ]);
        let err = run(cli, failing, &mut Vec::new()).unwrap_err();
        assert!(err.to_string().contains("slow down"));
        let _ = std::fs::remove_file(env_file);
    }

    #[test]
    fn key_reads_openrouter_status() {
        let key_file = temp("key.txt");
        std::fs::write(&key_file, "sk_or\n").unwrap();
        let transport = scripted(
            200,
            serde_json::json!({"data": {"label": "fragr", "limit": null, "limit_remaining": null, "usage": 0.25}}),
        );
        let cli = parse(&[
            "--provider",
            "openrouter",
            "--no-ledger",
            "--api-key-file",
            key_file.to_str().unwrap(),
            "key",
        ]);
        let mut out = Vec::new();
        run(cli, transport, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\"usage\": 0.25"));
        assert!(text.contains("no provider-side limit"));
        let cli = parse(&[
            "--provider",
            "typesafe",
            "--no-ledger",
            "--api-key-file",
            key_file.to_str().unwrap(),
            "key",
        ]);
        assert!(run(cli, scripted(200, serde_json::json!({})), &mut Vec::new()).is_err());
        let _ = std::fs::remove_file(key_file);
    }
}
