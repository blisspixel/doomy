//! Command line front end for sprite generation.
//!
//! The default is to price a run and generate nothing. Spending requires typing
//! a cap, and the cap is checked against the whole priced run before the first
//! image is requested.

use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use fragr_spritegen::reduce::{nearest_upscale, reduce_file, Palette, Reduction};
use fragr_spritegen::{
    append_ledger, check_budget, estimate, file_name, image_urls, ledger_ids, parse_spec, poll,
    read_dotenv_credential, read_ledger, submit, Error, Frame, LedgerEntry, Method, Request,
    Response, Spec, Transport,
};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "fragr-spritegen",
    about = "Developer-only sprite generation for fragr through the Higgsfield API"
)]
struct Cli {
    /// Dotenv file holding `higgsfield=<id>:<secret>`. Keep it out of git.
    #[arg(long, global = true, default_value = ".env")]
    env_file: PathBuf,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Price a spec and generate nothing.
    Price {
        #[arg(long)]
        spec: PathBuf,
        /// Only frames whose id contains this.
        #[arg(long)]
        only: Option<String>,
    },
    /// Generate. Refuses without a cap, and refuses if the priced run exceeds it.
    Gen {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        only: Option<String>,
        /// Dollar ceiling for this run. Required, and itself capped.
        #[arg(long)]
        max_spend_usd: Option<f64>,
        /// Print the prompt for each frame before sending it.
        #[arg(long)]
        show_prompts: bool,
    },
    /// Print the assembled prompt for each frame. Costs nothing, sends nothing.
    Prompts {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        only: Option<String>,
    },
    /// Turn generated pictures into sprites. Local, free, no network.
    Reduce {
        /// Input PNG, or a directory of them.
        #[arg(long)]
        input: PathBuf,
        /// Where the sprites go.
        #[arg(long)]
        out: PathBuf,
        /// Target height in pixels.
        #[arg(long, default_value_t = 128)]
        height: u32,
        /// Palette JSON to quantise to. Omit to keep the generated colours.
        #[arg(long)]
        palette: Option<PathBuf>,
        /// Do not trim to the alpha bounding box first.
        #[arg(long)]
        no_trim: bool,
        /// Keep soft edges instead of forcing alpha fully on or off.
        #[arg(long)]
        soft_alpha: bool,
        /// Also write a nearest-neighbour upscale by this factor, for looking at.
        #[arg(long)]
        preview_scale: Option<u32>,
    },
}

/// The real network.
struct HttpTransport {
    client: reqwest::blocking::Client,
}

impl HttpTransport {
    fn new() -> Result<Self, Error> {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map(|client| HttpTransport { client })
            .map_err(|e| Error::Transport(e.to_string()))
    }
}

impl Transport for HttpTransport {
    fn send(&self, credential: &str, request: &Request) -> Result<Response, Error> {
        let mut builder = match request.method {
            Method::Get => self.client.get(&request.url),
            Method::Post => self.client.post(&request.url),
        }
        .header("Authorization", format!("Key {credential}"));
        if let Some(body) = &request.body {
            builder = builder
                .header("Content-Type", "application/json")
                .body(body.clone());
        }
        let response = builder
            .send()
            .map_err(|e| Error::Transport(e.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|e| Error::Transport(e.to_string()))?;
        Ok(Response { status, body })
    }

    fn download(&self, url: &str) -> Result<Vec<u8>, Error> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| Error::Transport(e.to_string()))?;
        if !response.status().is_success() {
            return Err(Error::Api {
                status: response.status().as_u16(),
                body: format!("downloading {url}"),
            });
        }
        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| Error::Transport(e.to_string()))
    }
}

fn select<'a>(spec: &'a Spec, only: &Option<String>) -> Vec<&'a Frame> {
    match only {
        Some(pattern) => spec
            .frames
            .iter()
            .filter(|f| f.id.contains(pattern.as_str()))
            .collect(),
        None => spec.frames.iter().collect(),
    }
}

fn load(path: &PathBuf) -> Result<Spec, Error> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| Error::Io(format!("could not read {}: {e}", path.display())))?;
    parse_spec(&text)
}

/// Price every selected frame and print the table.
fn price_run(
    transport: &dyn Transport,
    credential: &str,
    spec: &Spec,
    frames: &[&Frame],
    out: &mut dyn IoWrite,
) -> Result<f64, Error> {
    let mut total = 0.0;
    for frame in frames {
        let usd = estimate(transport, credential, &spec.model, frame)?;
        total += usd;
        writeln!(out, "  {:<28} ${usd:.4}", frame.id).map_err(|e| Error::Io(e.to_string()))?;
    }
    Ok(total)
}

fn run(cli: Cli) -> Result<(), Error> {
    let out_stream = &mut std::io::stdout();

    match &cli.command {
        Cmd::Prompts { spec, only } => {
            let spec = load(spec)?;
            for frame in select(&spec, only) {
                writeln!(out_stream, "--- {} ---\n{}\n", frame.id, frame.prompt())
                    .map_err(|e| Error::Io(e.to_string()))?;
            }
            Ok(())
        }

        Cmd::Reduce {
            input,
            out,
            height,
            palette,
            no_trim,
            soft_alpha,
            preview_scale,
        } => {
            let palette = match palette {
                Some(path) => {
                    let text = std::fs::read_to_string(path).map_err(|e| {
                        Error::Io(format!("could not read {}: {e}", path.display()))
                    })?;
                    Some(Palette::from_json(&text)?)
                }
                None => None,
            };
            let settings = Reduction {
                height: *height,
                trim: !no_trim,
                palette,
                harden_alpha: !soft_alpha,
            };

            let inputs: Vec<PathBuf> = if input.is_dir() {
                let mut found: Vec<PathBuf> = std::fs::read_dir(input)
                    .map_err(|e| Error::Io(e.to_string()))?
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| {
                        path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                            matches!(
                                e.to_ascii_lowercase().as_str(),
                                "png" | "jpg" | "jpeg" | "webp"
                            )
                        })
                    })
                    .collect();
                found.sort();
                found
            } else {
                vec![input.clone()]
            };

            if inputs.is_empty() {
                return Err(Error::Io(format!("no images in {}", input.display())));
            }

            for path in &inputs {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("sprite");
                let target = out.join(format!("{stem}.png"));
                let (w, h) = reduce_file(path, &target, &settings)?;
                writeln!(out_stream, "  {stem:<28} {w}x{h}")
                    .map_err(|e| Error::Io(e.to_string()))?;

                if let Some(factor) = preview_scale {
                    let reduced = image::open(&target)
                        .map_err(|e| Error::Io(e.to_string()))?
                        .to_rgba8();
                    let preview = nearest_upscale(&reduced, *factor);
                    let preview_path = out.join(format!("{stem}_preview.png"));
                    preview
                        .save(&preview_path)
                        .map_err(|e| Error::Io(e.to_string()))?;
                }
            }
            Ok(())
        }

        Cmd::Price { spec, only } => {
            let credential = read_dotenv_credential(&cli.env_file)?;
            let spec = load(spec)?;
            let frames = select(&spec, only);
            let transport = HttpTransport::new()?;
            writeln!(out_stream, "model {}", spec.model).map_err(|e| Error::Io(e.to_string()))?;
            let total = price_run(&transport, &credential, &spec, &frames, out_stream)?;
            writeln!(
                out_stream,
                "\n{} frames, ${total:.4}. Nothing was generated.",
                frames.len()
            )
            .map_err(|e| Error::Io(e.to_string()))?;
            Ok(())
        }

        Cmd::Gen {
            spec,
            only,
            max_spend_usd,
            show_prompts,
        } => {
            let credential = read_dotenv_credential(&cli.env_file)?;
            let spec = load(spec)?;
            let done = ledger_ids(&read_ledger(&spec.out_dir));
            let selected = select(&spec, only);
            let frames: Vec<&Frame> = selected
                .into_iter()
                .filter(|f| !done.contains(&f.id))
                .collect();

            if frames.is_empty() {
                writeln!(
                    out_stream,
                    "nothing to do; the ledger already has every frame"
                )
                .map_err(|e| Error::Io(e.to_string()))?;
                return Ok(());
            }

            let transport = HttpTransport::new()?;
            writeln!(out_stream, "model {}", spec.model).map_err(|e| Error::Io(e.to_string()))?;
            let total = price_run(&transport, &credential, &spec, &frames, out_stream)?;
            let approved = check_budget(total, *max_spend_usd)?;
            let count = frames.len();
            let cap = max_spend_usd.unwrap_or_default();
            writeln!(
                out_stream,
                "\n{count} frames priced at ${approved:.4}, cap ${cap:.2}. Generating.\n"
            )
            .map_err(|e| Error::Io(e.to_string()))?;

            std::fs::create_dir_all(&spec.out_dir).map_err(|e| Error::Io(e.to_string()))?;

            let mut spent = 0.0;
            for frame in frames {
                if *show_prompts {
                    writeln!(out_stream, "  prompt: {}", frame.prompt())
                        .map_err(|e| Error::Io(e.to_string()))?;
                }
                let usd = estimate(&transport, &credential, &spec.model, frame)?;
                // The price was checked as a whole, but a provider is free to
                // change its mind between the estimate and the submission.
                if spent + usd > approved + 1e-9 {
                    writeln!(
                        out_stream,
                        "  {:<28} stopping: would pass the approved ${approved:.4}",
                        frame.id
                    )
                    .map_err(|e| Error::Io(e.to_string()))?;
                    break;
                }

                let status_url = submit(&transport, &credential, &spec.model, frame)?;
                let result = poll(
                    &transport,
                    &credential,
                    &status_url,
                    &mut |d| std::thread::sleep(d),
                    120,
                )?;
                let status = result
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                if status != "completed" {
                    // Failed and moderated requests are not charged, so this
                    // costs nothing and is worth saying out loud.
                    writeln!(out_stream, "  {:<28} {status}, not charged", frame.id)
                        .map_err(|e| Error::Io(e.to_string()))?;
                    continue;
                }

                let urls = image_urls(&result);
                if urls.is_empty() {
                    writeln!(out_stream, "  {:<28} completed with no image url", frame.id)
                        .map_err(|e| Error::Io(e.to_string()))?;
                    continue;
                }

                let mut files = Vec::new();
                for (index, url) in urls.iter().enumerate() {
                    let bytes = transport.download(url)?;
                    let name = file_name(&frame.id, index, url);
                    std::fs::write(spec.out_dir.join(&name), &bytes)
                        .map_err(|e| Error::Io(e.to_string()))?;
                    files.push(name);
                }

                spent += usd;
                append_ledger(
                    &spec.out_dir,
                    &LedgerEntry {
                        id: frame.id.clone(),
                        usd,
                        files: files.clone(),
                    },
                )?;
                writeln!(
                    out_stream,
                    "  {:<28} ${usd:.4}  {}",
                    frame.id,
                    files.join(" ")
                )
                .map_err(|e| Error::Io(e.to_string()))?;
            }

            writeln!(
                out_stream,
                "\nspent ${spent:.4} into {}",
                spec.out_dir.display()
            )
            .map_err(|e| Error::Io(e.to_string()))?;
            Ok(())
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    if let Err(err) = run(Cli::parse()) {
        eprintln!("fragr-spritegen: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn gen_parses_a_cap() {
        let cli = Cli::parse_from([
            "fragr-spritegen",
            "gen",
            "--spec",
            "s.json",
            "--max-spend-usd",
            "0.50",
        ]);
        match cli.command {
            Cmd::Gen { max_spend_usd, .. } => assert_eq!(max_spend_usd, Some(0.50)),
            other => panic!("expected gen, got {other:?}"),
        }
    }

    #[test]
    fn gen_without_a_cap_parses_but_is_refused_later() {
        let cli = Cli::parse_from(["fragr-spritegen", "gen", "--spec", "s.json"]);
        match cli.command {
            Cmd::Gen { max_spend_usd, .. } => {
                assert_eq!(max_spend_usd, None);
                assert!(check_budget(0.01, max_spend_usd).is_err());
            }
            other => panic!("expected gen, got {other:?}"),
        }
    }

    #[test]
    fn prompts_needs_no_credential_and_no_network() {
        let dir = std::env::temp_dir().join("fragr-spritegen-main-prompts");
        std::fs::create_dir_all(&dir).unwrap();
        let spec = dir.join("spec.json");
        std::fs::write(
            &spec,
            r#"{"model":"m","out_dir":"o","frames":[{"id":"tack","subject":"a pistol"}]}"#,
        )
        .unwrap();

        let cli = Cli::parse_from([
            "fragr-spritegen",
            "prompts",
            "--spec",
            spec.to_str().unwrap(),
        ]);
        // No .env exists in the temp directory, so this passing proves the
        // prompt path never reaches for a credential.
        run(cli).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_filters_by_substring() {
        let spec = parse_spec(
            r#"{"model":"m","out_dir":"o","frames":[
                 {"id":"tack_side","subject":"x"},
                 {"id":"rail_side","subject":"y"},
                 {"id":"rail_front","subject":"z"}]}"#,
        )
        .unwrap();
        let picked: Vec<&str> = select(&spec, &Some("rail".to_string()))
            .iter()
            .map(|f| f.id.as_str())
            .collect();
        assert_eq!(picked, vec!["rail_side", "rail_front"]);
        assert_eq!(select(&spec, &None).len(), 3);
    }
}
