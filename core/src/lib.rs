//! Oracle's brand projects, shared by Brand Kit, Brand Guide and Mockup Studio.
//!
//! - [`model`]: the `brand.json` schema (identity, logos in roles, palette, type, rules, presets).
//! - [`project`]: opening, migrating and saving a project; source change detection.
//! - [`render`]: drawing a master with Oracle's engines, then sizing, margins, background, density.
//! - [`export`]: planning and making client deliveries, with `manifest.json` and `README.txt`.
//!
//! Nothing here creates design: every logo, variant, colour and rule comes from the designer.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod export;
pub mod model;
pub mod presets;
pub mod project;
pub mod render;

pub use model::{Asset, BrandProject, Format, Role, Variant};
pub use project::{Project, SourceState};

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
