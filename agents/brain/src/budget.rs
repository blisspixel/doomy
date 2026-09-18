//! Hard spend controls for paid decision calls. Nothing bills without an
//! explicit per-run cap, every call is estimated before it is sent and settled
//! after it returns, and a ledger on disk carries the running total across runs
//! so a second session cannot quietly start the meter again.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Jev list price on 2026-09-18 (TypeSafe native and OpenRouter alike):
/// 0.042 dollars per million input tokens, output tokens free.
pub const JEV_INPUT_PER_MILLION: f64 = 0.042;
/// Output tokens are free for Jev; kept as a field so other models can be priced.
pub const JEV_OUTPUT_PER_MILLION: f64 = 0.0;

/// Dollars per million tokens, split by direction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

impl Default for Pricing {
    fn default() -> Self {
        Pricing {
            input_per_million: JEV_INPUT_PER_MILLION,
            output_per_million: JEV_OUTPUT_PER_MILLION,
        }
    }
}

impl Pricing {
    /// Dollars for a call of this size.
    pub fn cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        input_tokens as f64 * self.input_per_million / 1_000_000.0
            + output_tokens as f64 * self.output_per_million / 1_000_000.0
    }
}

/// Characters per token assumed when estimating. Measured on 2026-09-18: a
/// 1.5 kB decision body billed 726 input tokens through OpenRouter, about two
/// characters per token, so the usual four would undercount by half.
pub const CHARS_PER_TOKEN: u64 = 2;

/// Planning estimate of tokens in a text, rounded up, so caps err on the side
/// of refusing. Settlement uses the provider's reported count.
pub fn estimate_tokens(text: &str) -> u64 {
    (text.len() as u64).div_ceil(CHARS_PER_TOKEN)
}

/// One paid call as the ledger remembers it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Charge {
    pub unix: u64,
    pub provider: String,
    pub model: String,
    /// What the call was expected to cost before it was sent.
    pub estimated_usd: f64,
    /// What the provider reported after it returned, when it reported anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// False when the call failed after it was sent; it may still have billed.
    pub ok: bool,
}

impl Charge {
    /// What the ledger counts: the reported actual when it is a finite,
    /// non-negative number, else the estimate. A provider can never lower the
    /// running total by reporting a negative or nonsense cost.
    pub fn billed_usd(&self) -> f64 {
        self.actual_usd
            .filter(|a| a.is_finite() && *a >= 0.0)
            .unwrap_or(self.estimated_usd)
    }
}

/// Append-only record of every paid call, one JSON object per line under
/// `.agents/spend/`. Appending a line is crash safe (a torn last line is
/// skipped with a warning, never a reason to refuse the file) and lets several
/// processes share one ledger without overwriting each other's charges.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    #[serde(default)]
    pub charges: Vec<Charge>,
}

impl Ledger {
    /// A missing file is an empty ledger. The old single-object format is still
    /// read. A malformed line in the middle is an error; a torn final line is
    /// skipped so a crash mid-write cannot lock the next run out.
    pub fn load(path: &Path) -> Result<Self, Error> {
        if !path.exists() {
            return Ok(Ledger::default());
        }
        let text = fs::read_to_string(path)?;
        Ledger::parse(&text, &path.display().to_string())
    }

    fn parse(text: &str, label: &str) -> Result<Self, Error> {
        if text.trim().is_empty() {
            return Ok(Ledger::default());
        }
        // The old format was one object with a "charges" array. A single JSON
        // line would also parse as that struct (unknown fields are ignored), so
        // look for the key before accepting the old shape.
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(text)
        {
            if map.contains_key("charges") {
                return serde_json::from_value(serde_json::Value::Object(map))
                    .map_err(|err| Error::Malformed(format!("ledger {label}: {err}")));
            }
        }
        let lines: Vec<(usize, &str)> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .collect();
        let mut charges = Vec::with_capacity(lines.len());
        for (index, (number, line)) in lines.iter().enumerate() {
            match serde_json::from_str::<Charge>(line) {
                Ok(charge) => charges.push(charge),
                Err(err) if index + 1 == lines.len() => {
                    tracing::warn!(
                        "ledger {label}: skipping torn last line {}: {err}",
                        number + 1
                    );
                }
                Err(err) => {
                    return Err(Error::Malformed(format!(
                        "ledger {label} line {}: {err}",
                        number + 1
                    )));
                }
            }
        }
        Ok(Ledger { charges })
    }

    /// Rewrite the whole ledger as JSON lines (migrations and tests).
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        ensure_parent(path)?;
        let mut text = String::new();
        for charge in &self.charges {
            text.push_str(&encode_line(charge)?);
        }
        fs::write(path, text)?;
        Ok(())
    }

    /// Append one charge under an exclusive file lock.
    pub fn append(path: &Path, charge: &Charge) -> Result<(), Error> {
        ensure_parent(path)?;
        let line = encode_line(charge)?;
        // Read access is required for the lock on Windows; append keeps every
        // write at the end whatever the offset.
        let mut file = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;
        file.lock()?;
        let written = file.write_all(line.as_bytes()).and_then(|()| file.flush());
        let _ = file.unlock();
        written?;
        Ok(())
    }

    pub fn total_usd(&self) -> f64 {
        self.charges
            .iter()
            .fold(0.0, |acc, charge| acc + charge.billed_usd())
    }

    pub fn calls(&self) -> usize {
        self.charges.len()
    }
}

fn ensure_parent(path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn encode_line(charge: &Charge) -> Result<String, Error> {
    let mut line = serde_json::to_string(charge)
        .map_err(|err| Error::Malformed(format!("ledger encode: {err}")))?;
    line.push('\n');
    Ok(line)
}

/// The caps a run declares up front. `run_usd` at zero means no paid call at all.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caps {
    /// Dollars this run may spend, pre-approved on the command line.
    pub run_usd: f64,
    /// Dollars the ledger may reach in total, across every run, if set.
    pub total_usd: Option<f64>,
    /// Paid calls this run may make, if set. Bounds spend even if prices are wrong.
    pub run_calls: Option<u64>,
}

impl Default for Caps {
    fn default() -> Self {
        Caps {
            run_usd: 0.0,
            total_usd: None,
            run_calls: None,
        }
    }
}

/// The most one run may be allowed to spend, the developer ceiling in AGENTS.md.
/// Raising it is a code change, which is the written approval the rules ask for.
pub const MAX_RUN_CAP_USD: f64 = 5.0;

impl Caps {
    /// Every cap finite, non-negative, and under the ceiling.
    pub fn validate(&self) -> Result<(), Error> {
        let finite = |name: &str, value: f64| -> Result<(), Error> {
            if value.is_finite() && value >= 0.0 {
                Ok(())
            } else {
                Err(Error::InvalidArgument(format!(
                    "{name} must be a finite non-negative number, got {value}"
                )))
            }
        };
        finite("--max-spend-usd", self.run_usd)?;
        if self.run_usd > MAX_RUN_CAP_USD {
            return Err(Error::InvalidArgument(format!(
                "--max-spend-usd {} exceeds the {MAX_RUN_CAP_USD:.2} dollar per-run ceiling in AGENTS.md",
                self.run_usd
            )));
        }
        if let Some(total) = self.total_usd {
            finite("--max-total-usd", total)?;
        }
        Ok(())
    }
}

impl Pricing {
    /// Prices finite and non-negative.
    pub fn validate(&self) -> Result<(), Error> {
        for (name, value) in [
            ("--price-input-per-million", self.input_per_million),
            ("--price-output-per-million", self.output_per_million),
        ] {
            if !(value.is_finite() && value >= 0.0) {
                return Err(Error::InvalidArgument(format!(
                    "{name} must be a finite non-negative number, got {value}"
                )));
            }
        }
        Ok(())
    }
}

/// Why a call was not sent.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    NoCap,
    RunCap {
        spent: f64,
        estimate: f64,
        cap: f64,
    },
    TotalCap {
        total: f64,
        estimate: f64,
        cap: f64,
    },
    CallCap {
        calls: u64,
        cap: u64,
    },
    /// A cap, price, estimate, or running total is not a finite non-negative number.
    Invalid(String),
    /// The ledger on disk could not be read to enforce the total cap.
    LedgerUnreadable(String),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::NoCap => write!(
                f,
                "no spend approved; pass --max-spend-usd <dollars> to pre-approve a cap for this run"
            ),
            Refusal::RunCap {
                spent,
                estimate,
                cap,
            } => write!(
                f,
                "run cap reached: spent {spent:.6} plus next call {estimate:.6} exceeds {cap:.2} dollars"
            ),
            Refusal::TotalCap {
                total,
                estimate,
                cap,
            } => write!(
                f,
                "ledger cap reached: total {total:.6} plus next call {estimate:.6} exceeds {cap:.2} dollars"
            ),
            Refusal::CallCap { calls, cap } => {
                write!(f, "call cap reached: {calls} of {cap} paid calls used")
            }
            Refusal::Invalid(what) => write!(f, "refusing to spend: {what} is not a finite non-negative number"),
            Refusal::LedgerUnreadable(err) => write!(f, "refusing to spend: ledger unreadable: {err}"),
        }
    }
}

const EPSILON: f64 = 1e-9;

/// Running totals for one process plus the ledger behind it.
#[derive(Debug)]
pub struct Budget {
    pub caps: Caps,
    pub pricing: Pricing,
    ledger: Ledger,
    ledger_path: Option<PathBuf>,
    prior_usd: f64,
    run_usd: f64,
    run_calls: u64,
}

impl Budget {
    /// In-memory budget with no ledger on disk (tests and one-shot dry runs).
    pub fn new(caps: Caps, pricing: Pricing) -> Self {
        Budget {
            caps,
            pricing,
            ledger: Ledger::default(),
            ledger_path: None,
            prior_usd: 0.0,
            run_usd: 0.0,
            run_calls: 0,
        }
    }

    /// Budget backed by a ledger file; prior spend is read from it.
    pub fn with_ledger(caps: Caps, pricing: Pricing, path: &Path) -> Result<Self, Error> {
        let ledger = Ledger::load(path)?;
        let prior_usd = ledger.total_usd();
        Ok(Budget {
            caps,
            pricing,
            ledger,
            ledger_path: Some(path.to_path_buf()),
            prior_usd,
            run_usd: 0.0,
            run_calls: 0,
        })
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn ledger_path(&self) -> Option<&Path> {
        self.ledger_path.as_deref()
    }

    /// Dollars the ledger held before this run started.
    pub fn prior_usd(&self) -> f64 {
        self.prior_usd
    }

    pub fn run_usd(&self) -> f64 {
        self.run_usd
    }

    pub fn run_calls(&self) -> u64 {
        self.run_calls
    }

    /// Prior plus this run.
    pub fn total_usd(&self) -> f64 {
        self.prior_usd + self.run_usd
    }

    /// Would a call costing `estimate_usd` fit every cap? Checked before sending.
    /// When a total cap is set and a ledger file exists, the total on disk is
    /// re-read so charges from other processes sharing the ledger count too.
    pub fn check(&mut self, estimate_usd: f64) -> Result<(), Refusal> {
        if !(estimate_usd.is_finite() && estimate_usd >= 0.0) {
            return Err(Refusal::Invalid("the estimate".to_string()));
        }
        if !self.run_usd.is_finite() || !self.caps.run_usd.is_finite() {
            return Err(Refusal::Invalid("the running total or cap".to_string()));
        }
        if self.caps.run_usd <= 0.0 {
            return Err(Refusal::NoCap);
        }
        if let Some(cap) = self.caps.run_calls {
            if self.run_calls >= cap {
                return Err(Refusal::CallCap {
                    calls: self.run_calls,
                    cap,
                });
            }
        }
        if self.run_usd + estimate_usd > self.caps.run_usd + EPSILON {
            return Err(Refusal::RunCap {
                spent: self.run_usd,
                estimate: estimate_usd,
                cap: self.caps.run_usd,
            });
        }
        if let Some(cap) = self.caps.total_usd {
            if let Some(path) = &self.ledger_path {
                let on_disk =
                    Ledger::load(path).map_err(|err| Refusal::LedgerUnreadable(err.to_string()))?;
                // Other processes may have appended since this run started.
                self.prior_usd = self.prior_usd.max(on_disk.total_usd() - self.run_usd);
            }
            let total = self.total_usd();
            if !total.is_finite() {
                return Err(Refusal::Invalid("the ledger total".to_string()));
            }
            if total + estimate_usd > cap + EPSILON {
                return Err(Refusal::TotalCap {
                    total,
                    estimate: estimate_usd,
                    cap,
                });
            }
        }
        Ok(())
    }

    /// Count a call that was sent, whether or not it succeeded, and append it
    /// to the ledger. The in-memory totals move first so the run cap holds even
    /// when the disk write fails.
    pub fn record(&mut self, charge: Charge) -> Result<(), Error> {
        self.run_usd += charge.billed_usd();
        self.run_calls += 1;
        let appended = match &self.ledger_path {
            Some(path) => Ledger::append(path, &charge),
            None => Ok(()),
        };
        self.ledger.charges.push(charge);
        appended
    }
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn charge(estimated: f64, actual: Option<f64>) -> Charge {
        Charge {
            unix: 1,
            provider: "typesafe".into(),
            model: "jev-latest".into(),
            estimated_usd: estimated,
            actual_usd: actual,
            input_tokens: Some(100),
            output_tokens: Some(0),
            request_id: None,
            ok: true,
        }
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("fragr-brain-{name}-{}.json", std::process::id()))
    }

    #[test]
    fn pricing_and_token_estimates() {
        let pricing = Pricing::default();
        assert!((pricing.cost(1_000_000, 0) - 0.042).abs() < 1e-12);
        assert_eq!(pricing.cost(0, 1_000_000), 0.0);
        let custom = Pricing {
            input_per_million: 1.0,
            output_per_million: 5.0,
        };
        assert!((custom.cost(1_000_000, 1_000_000) - 6.0).abs() < 1e-12);
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 2);
        assert_eq!(estimate_tokens("abcde"), 3);
    }

    #[test]
    fn charge_bills_actual_over_estimate() {
        assert_eq!(charge(0.5, None).billed_usd(), 0.5);
        assert_eq!(charge(0.5, Some(0.1)).billed_usd(), 0.1);
    }

    #[test]
    fn ledger_roundtrip_and_missing_file() {
        let path = temp_path("ledger");
        let _ = fs::remove_file(&path);
        assert_eq!(Ledger::load(&path).unwrap(), Ledger::default());
        let mut ledger = Ledger::default();
        ledger.charges.push(charge(0.01, Some(0.02)));
        ledger.charges.push(charge(0.03, None));
        ledger.save(&path).unwrap();
        let back = Ledger::load(&path).unwrap();
        assert_eq!(back, ledger);
        assert!((back.total_usd() - 0.05).abs() < 1e-12);
        assert_eq!(back.calls(), 2);
        fs::write(&path, "   ").unwrap();
        assert_eq!(Ledger::load(&path).unwrap(), Ledger::default());
        // A torn last line is skipped; a bad middle line is an error.
        let good = serde_json::to_string(&charge(0.01, None)).unwrap();
        fs::write(&path, format!("{good}\n{{\"unix\":1,\"prov")).unwrap();
        assert_eq!(Ledger::load(&path).unwrap().calls(), 1);
        fs::write(&path, format!("{{not json\n{good}\n")).unwrap();
        assert!(matches!(Ledger::load(&path), Err(Error::Malformed(_))));
        // The old single-object format still loads.
        let old = serde_json::json!({"charges": [charge(0.02, None)]});
        fs::write(&path, old.to_string()).unwrap();
        assert_eq!(Ledger::load(&path).unwrap().calls(), 1);
        // Append is one line per charge and survives a re-read.
        let _ = fs::remove_file(&path);
        Ledger::append(&path, &charge(0.03, None)).unwrap();
        Ledger::append(&path, &charge(0.04, Some(0.05))).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!((Ledger::load(&path).unwrap().total_usd() - 0.08).abs() < 1e-12);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn non_finite_and_negative_values_never_pass() {
        assert_eq!(charge(0.5, Some(f64::NAN)).billed_usd(), 0.5);
        assert_eq!(charge(0.5, Some(-1.0)).billed_usd(), 0.5);
        assert_eq!(charge(0.5, Some(f64::INFINITY)).billed_usd(), 0.5);
        let caps = Caps {
            run_usd: f64::NAN,
            total_usd: None,
            run_calls: None,
        };
        assert!(caps.validate().is_err());
        let mut budget = Budget::new(caps, Pricing::default());
        assert!(matches!(budget.check(0.0), Err(Refusal::Invalid(_))));
        let caps = Caps {
            run_usd: 1.0,
            total_usd: Some(f64::INFINITY),
            run_calls: None,
        };
        assert!(caps.validate().is_err());
        let mut budget = Budget::new(
            Caps {
                run_usd: 1.0,
                total_usd: None,
                run_calls: None,
            },
            Pricing::default(),
        );
        assert!(matches!(budget.check(f64::NAN), Err(Refusal::Invalid(_))));
        assert!(matches!(budget.check(-0.5), Err(Refusal::Invalid(_))));
        assert!(Refusal::Invalid("x".into()).to_string().contains("finite"));
        assert!(Caps {
            run_usd: MAX_RUN_CAP_USD + 0.01,
            total_usd: None,
            run_calls: None,
        }
        .validate()
        .is_err());
        assert!(Caps {
            run_usd: MAX_RUN_CAP_USD,
            total_usd: Some(0.0),
            run_calls: None,
        }
        .validate()
        .is_ok());
        assert!(Pricing {
            input_per_million: -1.0,
            output_per_million: 0.0
        }
        .validate()
        .is_err());
        assert!(Pricing {
            input_per_million: f64::NAN,
            output_per_million: 0.0
        }
        .validate()
        .is_err());
        assert!(Pricing::default().validate().is_ok());
    }

    #[test]
    fn total_cap_sees_charges_from_other_processes() {
        let path = temp_path("shared");
        let _ = fs::remove_file(&path);
        let caps = Caps {
            run_usd: 1.0,
            total_usd: Some(0.05),
            run_calls: None,
        };
        let mut budget = Budget::with_ledger(caps, Pricing::default(), &path).unwrap();
        assert_eq!(budget.check(0.01), Ok(()));
        // Another process appends 0.045 behind our back.
        Ledger::append(&path, &charge(0.045, None)).unwrap();
        assert!(matches!(budget.check(0.01), Err(Refusal::TotalCap { .. })));
        assert!((budget.prior_usd() - 0.045).abs() < 1e-12);
        fs::write(&path, "{bad\n{worse\n").unwrap();
        assert!(matches!(
            budget.check(0.0),
            Err(Refusal::LedgerUnreadable(_))
        ));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn no_cap_refuses_everything() {
        let mut budget = Budget::new(Caps::default(), Pricing::default());
        assert_eq!(budget.check(0.0), Err(Refusal::NoCap));
        assert!(Refusal::NoCap.to_string().contains("--max-spend-usd"));
    }

    #[test]
    fn run_cap_is_enforced_before_the_call() {
        let caps = Caps {
            run_usd: 0.05,
            total_usd: None,
            run_calls: None,
        };
        let mut budget = Budget::new(caps, Pricing::default());
        assert_eq!(budget.check(0.05), Ok(()));
        budget.record(charge(0.03, None)).unwrap();
        assert_eq!(budget.check(0.02), Ok(()));
        let refused = budget.check(0.021).unwrap_err();
        assert!(matches!(refused, Refusal::RunCap { .. }), "{refused:?}");
        assert!(refused.to_string().contains("run cap reached"));
        assert_eq!(budget.run_calls(), 1);
        assert!((budget.run_usd() - 0.03).abs() < 1e-12);
    }

    #[test]
    fn actuals_replace_estimates_in_the_running_total() {
        let caps = Caps {
            run_usd: 1.0,
            total_usd: None,
            run_calls: None,
        };
        let mut budget = Budget::new(caps, Pricing::default());
        budget.record(charge(0.5, Some(0.9))).unwrap();
        assert!((budget.run_usd() - 0.9).abs() < 1e-12);
        assert!(matches!(budget.check(0.2), Err(Refusal::RunCap { .. })));
    }

    #[test]
    fn call_cap_and_total_cap() {
        let caps = Caps {
            run_usd: 10.0,
            total_usd: Some(0.02),
            run_calls: Some(1),
        };
        let path = temp_path("total");
        let mut prior = Ledger::default();
        prior.charges.push(charge(0.015, None));
        prior.save(&path).unwrap();
        let mut budget = Budget::with_ledger(caps, Pricing::default(), &path).unwrap();
        assert!((budget.prior_usd() - 0.015).abs() < 1e-12);
        assert_eq!(budget.ledger_path(), Some(path.as_path()));
        let refused = budget.check(0.01).unwrap_err();
        assert!(matches!(refused, Refusal::TotalCap { .. }), "{refused:?}");
        assert!(refused.to_string().contains("ledger cap reached"));
        assert_eq!(budget.check(0.004), Ok(()));
        budget.record(charge(0.004, None)).unwrap();
        let refused = budget.check(0.0).unwrap_err();
        assert_eq!(
            refused,
            Refusal::CallCap { calls: 1, cap: 1 },
            "call cap wins once used up"
        );
        assert!(refused.to_string().contains("call cap reached"));
        let saved = Ledger::load(&path).unwrap();
        assert_eq!(saved.calls(), 2);
        assert!((budget.total_usd() - 0.019).abs() < 1e-12);
        assert_eq!(budget.ledger().calls(), 2);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn now_unix_is_after_2026() {
        assert!(now_unix() > 1_767_225_600);
    }
}
