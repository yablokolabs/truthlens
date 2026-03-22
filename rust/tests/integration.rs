//! Integration tests for TruthLens v0.4
//!
//! These tests verify the public API works end-to-end.

use truthlens::{analyze, analyze_with_verification, check_consistency, RiskLevel};

#[test]
fn analyze_factual_text() {
    let report = analyze("Albert Einstein was born on March 14, 1879, in Ulm, Germany.");
    assert!(report.score > 0.0 && report.score <= 1.0);
    assert!(report.total_claims >= 1);
    // Factual text with specific dates should not be critical
    assert_ne!(report.risk_level, RiskLevel::Critical);
}

#[test]
fn analyze_hedged_text() {
    let report = analyze(
        "Climate change might be linked to increased hurricane frequency. \
         Some researchers believe that ocean temperatures could affect storm intensity.",
    );
    // Hedged text should score higher (less risky)
    assert!(report.score > 0.4, "Hedged text should score reasonably: {}", report.score);
}

#[test]
fn analyze_overconfident_text() {
    let report = analyze(
        "The Great Wall of China is exactly 21,196.18 kilometers long. \
         It was built by precisely 3,247,862 workers over 47 years. \
         The wall can be clearly seen from the International Space Station.",
    );
    // Overconfident text with suspicious precision
    assert!(report.high_risk_claims > 0, "Should flag some high-risk claims");
}

#[test]
fn consistency_identical_responses() {
    let report = check_consistency(&[
        "Einstein was born in 1879 in Ulm, Germany.",
        "Einstein was born in 1879 in Ulm, Germany.",
    ]);
    assert!(report.contradictions.is_empty());
    assert!(report.consistency_score > 0.5);
}

#[test]
fn consistency_contradicting_responses() {
    let report = check_consistency(&[
        "Einstein was born in 1879 in Ulm, Germany.",
        "Einstein was born in 1879 in Munich, Germany.",
    ]);
    assert!(!report.contradictions.is_empty() || !report.unique_claims.is_empty());
}

#[test]
fn analyze_with_verification_returns_results() {
    let report = analyze_with_verification("Albert Einstein was born in 1879 in Ulm, Germany.");
    assert!(report.score > 0.0 && report.score <= 1.0);
    // Should have verification results on claims
    let has_verification = report.claims.iter().any(|c| c.verification.is_some());
    assert!(has_verification, "analyze_with_verification should populate verification field");
}

#[test]
fn analyze_empty_text() {
    let report = analyze("");
    assert_eq!(report.total_claims, 0);
    // Empty text has no claims; score depends on trajectory modifier
    assert!(report.score >= 0.0 && report.score <= 1.0);
}

#[test]
fn json_roundtrip() {
    let report = analyze("Einstein was born in 1879. He developed relativity in 1905.");
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: truthlens::TrustReport = serde_json::from_str(&json).unwrap();
    assert!((deserialized.score - report.score).abs() < 0.001);
    assert_eq!(deserialized.total_claims, report.total_claims);
}
