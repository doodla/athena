//! `athena status` prints the latest build measurement (`athena measure-build`) on a seeded record in a
//! temporary repository, and a clear line while there is none. No build runs and no model is called.

mod common;

use athena_data::RecordId;
use athena_data::records::{AnyRecord, BuildMeasurement, SccacheCounts, SccacheFigures};
use common::*;

fn measurement() -> BuildMeasurement {
    BuildMeasurement {
        id: RecordId::parse("20261029-090000-5abc").unwrap(),
        at: "2026-10-29T09:00:00Z".parse().unwrap(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        cargo_lock_hash: "89abcdef0123456789abcdef0123456789abcdef".into(),
        target_bytes: 4_100_000_000,
        source_bytes: 2_500_000,
        first_build_seconds: 61.5,
        second_build_seconds: 12.25,
        sccache: SccacheFigures::Measured {
            first: SccacheCounts { hits: 0, misses: 9 },
            second: SccacheCounts { hits: 4, misses: 5 },
        },
        first_peak_resident_bytes: Some(3_500_000_000),
        first_cores_used: Some(5.75),
        second_peak_resident_bytes: Some(2_500_000_000),
        second_cores_used: Some(3.5),
    }
}

#[test]
fn status_prints_the_latest_build_measurement_line() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());

    let none = athena_cli::display::status(&athena.status().unwrap());
    let line = none
        .lines()
        .find(|l| l.starts_with("build      measurement:"))
        .unwrap_or_else(|| panic!("no build measurement line in:\n{none}"));
    assert!(line.contains("none yet"), "{line}");

    let record = AnyRecord::BuildMeasurement(Box::new(measurement()));
    write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Add a fixture build measurement"])
        .unwrap();

    let view = athena.status().unwrap();
    let text = athena_cli::display::status(&view);
    let line = text
        .lines()
        .find(|l| l.starts_with("build      measurement:"))
        .unwrap_or_else(|| panic!("no build measurement line in:\n{text}"));
    assert_eq!(line, view.build_measurement);
    for want in [
        "T 4.1 GB",
        "S 2.5 MB",
        "build 1 61.50s, build 2 12.25s",
        "peak memory build 1 3.5 GB, build 2 2.5 GB",
        "cores used build 1 5.75, build 2 3.50",
        "sccache build 1 0 hits / 9 misses, build 2 4 hits / 5 misses",
        "commit 0123456789ab",
    ] {
        assert!(line.contains(want), "{want:?} not in {line}");
    }
    assert!(!text.contains("measurement: none yet"), "{text}");
}
