//! A fragr agent with a remote decision brain and a local controller.
//!
//! The brain is a decision model (TypeSafe Jev, natively or through OpenRouter)
//! that answers a fixed set of typed questions about the current fight a few
//! times a second: which stance to take, which weapon to hold, how much danger
//! the fighter is in. The controller turns that intent into wire actions every
//! tick without waiting on the network. When the brain is slow, wrong, unsure,
//! or out of budget, local rules take over and the fighter keeps playing.
//!
//! Nothing here bills unless a run is started with an explicit spend cap, and
//! every paid call is estimated before it is sent and written to a ledger after
//! it returns. The same loop runs with `Provider::Local` at zero cost, which is
//! what CI exercises.

pub mod bot;
pub mod budget;
pub mod decision;
pub mod dotenv;
pub mod plan;
pub mod provider;
pub mod telemetry;

use std::fmt;

/// Every failure the crate reports. Transport and API errors carry the provider's
/// own message so a refused key or a rate limit reads as itself in the log.
#[derive(Debug)]
pub enum Error {
    /// No key was found for a paid provider.
    MissingApiKey(String),
    /// A spend cap would be crossed by the next call.
    Budget(budget::Refusal),
    /// The request never completed (connect, timeout, read).
    Transport(String),
    /// The provider answered with a non-success status.
    Api {
        status: u16,
        message: String,
    },
    /// The provider answered with a body this crate cannot read.
    Malformed(String),
    Io(std::io::Error),
    InvalidArgument(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingApiKey(names) => write!(
                f,
                "no API key: set one of {names} in the environment or .env, or pass --api-key-file"
            ),
            Error::Budget(refusal) => write!(f, "budget: {refusal}"),
            Error::Transport(msg) => write!(f, "transport: {msg}"),
            Error::Api { status, message } => write!(f, "api error {status}: {message}"),
            Error::Malformed(msg) => write!(f, "malformed response: {msg}"),
            Error::Io(err) => write!(f, "io: {err}"),
            Error::InvalidArgument(msg) => write!(f, "invalid argument: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<budget::Refusal> for Error {
    fn from(refusal: budget::Refusal) -> Self {
        Error::Budget(refusal)
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn error_display_covers_variants() {
        let cases: Vec<(Error, &str)> = vec![
            (Error::MissingApiKey("A, B".into()), "no API key"),
            (Error::Budget(budget::Refusal::NoCap), "budget:"),
            (Error::Transport("x".into()), "transport: x"),
            (
                Error::Api {
                    status: 402,
                    message: "credits".into(),
                },
                "api error 402: credits",
            ),
            (Error::Malformed("m".into()), "malformed response: m"),
            (Error::Io(std::io::Error::other("disk")), "io: disk"),
            (
                Error::InvalidArgument("arg".into()),
                "invalid argument: arg",
            ),
        ];
        for (err, needle) in cases {
            let text = err.to_string();
            assert!(text.contains(needle), "{text} should contain {needle}");
        }
        let from_io: Error = std::io::Error::other("z").into();
        assert!(matches!(from_io, Error::Io(_)));
        let from_refusal: Error = budget::Refusal::NoCap.into();
        assert!(matches!(from_refusal, Error::Budget(_)));
    }
}
