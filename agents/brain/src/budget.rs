//! Hard spend controls for paid decision calls. Nothing bills without an
//! explicit per-run cap, every call is estimated before it is sent and settled
//! after it returns, and a ledger on disk carries the running total across runs
//! so a second session cannot quietly start the meter again.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
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
    /// What the ledger counts: the reported actual, else the estimate.
    pub fn billed_usd(&self) -> f64 {
        self.actual_usd.unwrap_or(self.estimated_usd)
    }
}

/// Append-only record of every paid call, kept as JSON under `.agents/spend/`.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    #[serde(default)]
    pub charges: Vec<Charge>,
}

impl Ledger {
    /// A missing file is an empty ledger; an unreadable one is an error, never a reset.
    pub fn load(path: &Path) -> Result<Self, Error> {
        if !path.exists() {
            return Ok(Ledger::default());
        }
        let text = fs::read_to_string(path)?;
        if text.trim().is_empty() {
            return Ok(Ledger::default());
        }
        serde_json::from_str(&text)
            .map_err(|err| Error::Malformed(format!("ledger {}: {err}", path.display())))
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let text = serde_json::to_string_pretty(self)
            .map_err(|err| Error::Malformed(format!("ledger encode: {err}")))?;
        fs::write(path, text)?;
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

/// Why a call was not sent.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    NoCap,
    RunCap { spent: f64, estimate: f64, cap: f64 },
    TotalCap { total: f64, estimate: f64, cap: f64 },
    CallCap { calls: u64, cap: u64 },
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
    pub fn check(&self, estimate_usd: f64) -> Result<(), Refusal> {
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
            let total = self.total_usd();
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

    /// Count a call that was sent, whether or not it succeeded, and persist.
    pub fn record(&mut self, charge: Charge) -> Result<(), Error> {
        self.run_usd += charge.billed_usd();
        self.run_calls += 1;
        self.ledger.charges.push(charge);
        if let Some(path) = &self.ledger_path {
            self.ledger.save(path)?;
        }
        Ok(())
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
        fs::write(&path, "{not json").unwrap();
        assert!(matches!(Ledger::load(&path), Err(Error::Malformed(_))));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn no_cap_refuses_everything() {
        let budget = Budget::new(Caps::default(), Pricing::default());
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
