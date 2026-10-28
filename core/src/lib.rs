//! Oracle's brand projects, shared by Brand Kit, Brand Guide and Mockup Studio.
//!
//! - [`model`]: the `brand.json` schema (identity, logos in roles, palette, type, rules, presets).
//! - [`project`]: opening, migrating and saving a project; source change detection.
//! - [`render`]: drawing a master with Oracle's engines, then sizing, margins, background, density.
//! - [`export`]: planning and making client deliveries, with `manifest.json` and `README.txt`.
//! - [`mockup`]: Mockup Studio's templates and scenes; [`composite`] draws them.
//!
//! Nothing here creates design: every logo, variant, colour and rule comes from the designer.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod composite;
pub mod export;
pub mod guide;
pub mod mockup;
pub mod mockup_demo;
pub mod model;
pub mod presets;
pub mod project;
pub mod recolor;
pub mod tokens;
pub mod vectorize;
pub mod render;
pub mod studio;

pub use model::{Asset, BrandProject, Format, Role, Variant};
pub use project::{Project, SourceState};

/// The folder of the apps' local logs: `%LOCALAPPDATA%/Oracle/logs` on Windows,
/// `~/.local/state/oracle/logs` elsewhere.
pub fn log_dir() -> Option<std::path::PathBuf> {
    if let Some(d) = std::env::var_os("LOCALAPPDATA") {
        return Some(std::path::PathBuf::from(d).join("Oracle").join("logs"));
    }
    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local").join("state").join("oracle").join("logs"))
}

/// Appends one line to `<log_dir>/<app>.log` (UTC time, level, message). The file is started
/// afresh past 1 MB (the previous one kept as `.old.log`). Never fails the caller: a log that
/// can't be written is skipped.
pub fn log_line(app: &str, level: &str, message: &str) {
    use std::io::Write;
    let Some(dir) = log_dir() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let file = dir.join(format!("{app}.log"));
    if std::fs::metadata(&file).is_ok_and(|m| m.len() > 1 << 20) {
        let _ = std::fs::rename(&file, dir.join(format!("{app}.old.log")));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&file) {
        let _ = writeln!(f, "{} {level} {}", now_rfc3339(), message.replace(['\r', '\n'], " "));
    }
}

/// Now, as RFC 3339 in UTC (`2026-10-22T14:03:09Z`), without a date library.
pub fn now_rfc3339() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    rfc3339(secs)
}

fn rfc3339(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    #[test]
    fn rfc3339_dates() {
        assert_eq!(super::rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(super::rfc3339(1_792_670_589), "2026-10-22T12:03:09Z");
        assert_eq!(super::rfc3339(951_782_400), "2000-02-29T00:00:00Z");
    }
}
