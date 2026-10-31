// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The fixture in this repository's `test_data/`, parsed from a file path.
//!
//! This was the `test_ias_importer` `[[bin]]` that MANAGEMENT.DataMigration built
//! from this repository's file, with the fixture path spelled from
//! DataMigration's manifest; it printed the first ten channels. It is a
//! `cargo test` here now, resolved from this crate's own manifest (2026-10-30 audit).

use std::path::PathBuf;

use apkaudio_wireless_ias::parse_ias_report;

#[test]
fn the_report_fixture_parses_from_its_path() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test_data/report.html");
    let r = parse_ias_report(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(r.status, "success");
    assert_eq!(r.total_channels, 168);
    assert!(r.channels.iter().all(|c| in_band(c.freq_mhz)));
}

/// The band every channel must sit in (the kernel rule, restated only as an assertion).
fn in_band(mhz: f64) -> bool {
    mhz > 100.0 && mhz < 3000.0
}
