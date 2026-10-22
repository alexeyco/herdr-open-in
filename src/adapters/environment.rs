//! Access to trimmed, non-blank process environment values.

use anyhow::{Result, bail};

pub(crate) fn value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn required(name: &str) -> Result<String> {
    let Some(value) = value(name) else {
        bail!("{name} is not set or blank");
    };
    Ok(value)
}
