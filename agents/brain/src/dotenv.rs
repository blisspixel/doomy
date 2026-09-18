//! Key discovery without a dotenv dependency: process environment first, then a
//! `.env` file, then an explicit key file. Values are trimmed and never logged.

use crate::Error;
use std::fs;
use std::path::Path;

/// Read the first matching `KEY=VALUE` from a `.env` style file. Names compare
/// case-insensitively, `export` prefixes and quotes are stripped, `#` lines are
/// comments. `Ok(None)` when the file or every name is absent.
pub fn read_dotenv_value(path: &Path, names: &[&str]) -> Result<Option<String>, Error> {
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
        if !names.iter().any(|known| known.eq_ignore_ascii_case(key)) {
            continue;
        }
        let value = value.trim().trim_matches(|c| c == '"' || c == '\'').trim();
        if !value.is_empty() {
            return Ok(Some(value.to_string()));
        }
    }
    Ok(None)
}

/// Resolve a key: an explicit key file wins, then the process environment under
/// any of `names`, then the `.env` file. `Ok(None)` means nothing was found.
pub fn resolve_key(
    names: &[&str],
    env: &dyn Fn(&str) -> Option<String>,
    env_file: Option<&Path>,
    key_file: Option<&Path>,
) -> Result<Option<String>, Error> {
    if let Some(path) = key_file {
        let text = fs::read_to_string(path)?;
        let key = text.trim();
        return Ok((!key.is_empty()).then(|| key.to_string()));
    }
    for name in names {
        if let Some(value) = env(name) {
            let value = value.trim();
            if !value.is_empty() {
                return Ok(Some(value.to_string()));
            }
        }
    }
    match env_file {
        Some(path) => read_dotenv_value(path, names),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_file(name: &str, body: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("fragr-brain-{name}-{}.env", std::process::id()));
        fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn dotenv_reads_first_matching_name() {
        let path = temp_file(
            "dotenv",
            "# comment\nelevenlabs=nope\nexport TYPESAFE_API_KEY = \"sk_one\"\ntypesafe=sk_two\n",
        );
        let names = ["TYPESAFE_API_KEY", "TYPESAFE"];
        assert_eq!(
            read_dotenv_value(&path, &names).unwrap().as_deref(),
            Some("sk_one")
        );
        assert_eq!(
            read_dotenv_value(&path, &["typesafe"]).unwrap().as_deref(),
            Some("sk_two")
        );
        assert_eq!(read_dotenv_value(&path, &["OPENROUTER"]).unwrap(), None);
        let empty = temp_file("dotenv-empty", "TYPESAFE=\nTYPESAFE=''\nno equals here\n");
        assert_eq!(read_dotenv_value(&empty, &names).unwrap(), None);
        assert_eq!(
            read_dotenv_value(Path::new("/definitely/missing.env"), &names).unwrap(),
            None
        );
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(empty);
    }

    #[test]
    fn resolve_key_orders_sources() {
        let names = ["TYPESAFE_API_KEY", "TYPESAFE"];
        let dotenv = temp_file("resolve", "TYPESAFE=from_file\n");
        let key_file = temp_file("keyfile", "  from_keyfile \n");
        let blank_key_file = temp_file("keyfile-blank", "  \n");
        let env_hit = |name: &str| (name == "TYPESAFE").then(|| " from_env ".to_string());
        let env_miss = |_: &str| None;
        assert_eq!(
            resolve_key(&names, &env_hit, Some(&dotenv), Some(&key_file))
                .unwrap()
                .as_deref(),
            Some("from_keyfile")
        );
        assert_eq!(
            resolve_key(&names, &env_hit, Some(&dotenv), None)
                .unwrap()
                .as_deref(),
            Some("from_env")
        );
        assert_eq!(
            resolve_key(&names, &env_miss, Some(&dotenv), None)
                .unwrap()
                .as_deref(),
            Some("from_file")
        );
        assert_eq!(resolve_key(&names, &env_miss, None, None).unwrap(), None);
        assert_eq!(
            resolve_key(&names, &env_miss, None, Some(&blank_key_file)).unwrap(),
            None
        );
        assert!(resolve_key(&names, &env_miss, None, Some(Path::new("/missing/key"))).is_err());
        let _ = fs::remove_file(dotenv);
        let _ = fs::remove_file(key_file);
        let _ = fs::remove_file(blank_key_file);
    }
}
