//! Command line front end for the agent playtest harness.

use clap::Parser;
use fragr_playtest::{check_thresholds, run, Config};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "fragr-playtest",
    version,
    about = "Scripted agents play fragr rounds in-process and file a metrics report."
)]
struct Cli {
    /// Number of scripted agents.
    #[arg(long, default_value_t = 4)]
    agents: usize,
    /// Rounds to complete before stopping.
    #[arg(long, default_value_t = 1)]
    rounds: u32,
    /// Map: 1/arena or 2/compliance-yard.
    #[arg(long, default_value = "1")]
    map: String,
    /// Frag limit for each round.
    #[arg(long, default_value_t = 5)]
    frag_limit: u32,
    /// Round time limit in seconds.
    #[arg(long, default_value_t = 60)]
    time_limit_seconds: u32,
    /// Hard stop for the whole run, in seconds of match time.
    #[arg(long, default_value_t = 120)]
    max_seconds: u64,
    /// Where to write the JSON report.
    #[arg(long, default_value = ".agents/playtest/report.json")]
    report: PathBuf,
    /// Exit non-zero when a frustration threshold is crossed.
    #[arg(long)]
    assert: bool,
    /// Simulation seed, so a run can be reproduced and two runs compared.
    #[arg(long, default_value_t = 1)]
    seed: u64,
}

fn config_from(cli: &Cli) -> Result<Config, String> {
    let map = fragr_server::sim::MapKind::from_cli(&cli.map)
        .ok_or_else(|| format!("invalid --map {:?}", cli.map))?;
    Ok(Config {
        agents: cli.agents,
        rounds: cli.rounds,
        map,
        frag_limit: cli.frag_limit,
        time_limit_ticks: cli.time_limit_seconds * 20,
        max_ticks: cli.max_seconds * 20,
        seed: cli.seed,
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let config = match config_from(&cli) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    };
    let (report, _observation) = match run(config).await {
        Ok(result) => result,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    };
    if let Some(parent) = cli.report.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                eprintln!("error: cannot create {}: {err}", parent.display());
                std::process::exit(2);
            }
        }
    }
    match serde_json::to_string_pretty(&report) {
        Ok(json) => {
            if let Err(err) = std::fs::write(&cli.report, format!("{json}\n")) {
                eprintln!("error: cannot write {}: {err}", cli.report.display());
                std::process::exit(2);
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(2);
        }
    }
    println!(
        "playtest: {} agents, {} round(s), {:.1} s, {} frags ({:.2} per minute), first frag {}, longest gap {:.1} s, {} spawn deaths, {:.0} bytes per snapshot",
        report.agents,
        report.rounds_completed,
        report.seconds,
        report.frags,
        report.frags_per_minute,
        report
            .time_to_first_frag_s
            .map(|s| format!("{s:.1} s"))
            .unwrap_or_else(|| "never".to_string()),
        report.longest_gap_without_frag_s,
        report.spawn_deaths,
        report.snapshot_bytes_per_tick
    );
    println!("report: {}", cli.report.display());
    let problems = check_thresholds(&report);
    for problem in &problems {
        println!("threshold: {problem}");
    }
    if cli.assert && !problems.is_empty() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults_and_map() {
        let cli = Cli::try_parse_from(["fragr-playtest"]).unwrap();
        let config = config_from(&cli).unwrap();
        assert_eq!(config.agents, 4);
        assert_eq!(config.rounds, 1);
        assert_eq!(config.frag_limit, 5);
        assert_eq!(config.time_limit_ticks, 1200);
        assert_eq!(config.max_ticks, 2400);
        assert_eq!(config.map, fragr_server::sim::MapKind::ArenaDuel);
        assert!(!cli.assert);
    }

    #[test]
    fn parses_custom_arguments() {
        let cli = Cli::try_parse_from([
            "fragr-playtest",
            "--agents",
            "8",
            "--rounds",
            "2",
            "--map",
            "compliance-yard",
            "--frag-limit",
            "3",
            "--time-limit-seconds",
            "45",
            "--max-seconds",
            "200",
            "--assert",
        ])
        .unwrap();
        let config = config_from(&cli).unwrap();
        assert_eq!(config.agents, 8);
        assert_eq!(config.rounds, 2);
        assert_eq!(config.map, fragr_server::sim::MapKind::ComplianceYard);
        assert_eq!(config.frag_limit, 3);
        assert_eq!(config.time_limit_ticks, 900);
        assert_eq!(config.max_ticks, 4000);
        assert!(cli.assert);
        let bad = Cli::try_parse_from(["fragr-playtest", "--map", "moon"]).unwrap();
        assert!(config_from(&bad).is_err());
    }
}
