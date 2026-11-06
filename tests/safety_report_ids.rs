//! Report identities must survive concurrent publication without overwriting reports.

use memscope_rs::analysis::safety::analyzer::{SafetyAnalysisConfig, SafetyAnalyzer};
use memscope_rs::analysis::safety::types::UnsafeSource;
use std::collections::HashSet;
use std::sync::Barrier;

/// Objective: Exercise rapid publication by 50 threads into the same report map.
/// Invariants: every returned ID identifies a retained report; counters match map size.
#[test]
fn concurrent_reports_are_retained_by_unique_identity() {
    let analyzer = SafetyAnalyzer::new(SafetyAnalysisConfig {
        detailed_risk_assessment: false,
        ..SafetyAnalysisConfig::default()
    });
    let start = Barrier::new(50);
    let ids = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..50)
            .map(|_| {
                let analyzer = &analyzer;
                let start = &start;
                scope.spawn(move || {
                    start.wait();
                    (0..10)
                        .map(|_| {
                            analyzer
                                .generate_unsafe_report(
                                    UnsafeSource::RawPointer {
                                        operation: "read".to_string(),
                                        location: "report regression".to_string(),
                                    },
                                    &[],
                                    &[],
                                )
                                .expect("publish a report")
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("report worker must finish"))
            .collect::<HashSet<_>>()
    });
    let reports = analyzer.get_unsafe_reports();
    assert_eq!(ids.len(), 500, "every publication must have a distinct ID");
    assert_eq!(reports.len(), 500, "no report may overwrite another report");
    assert!(
        ids.iter().all(|id| reports.contains_key(id)),
        "all returned IDs must resolve"
    );
    assert_eq!(
        analyzer.get_stats().total_reports,
        reports.len(),
        "stats must match retained reports"
    );
}
