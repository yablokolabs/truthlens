//! TruthLens — AI Hallucination Detector
//!
//! Analyzes AI-generated text for hallucination risk using linguistic signals,
//! claim decomposition, and formally verified trust scoring.
//!
//! # Quick Start
//!
//! ```rust
//! use truthlens::{analyze, TrustReport};
//!
//! let text = "Albert Einstein was born in 1879 in Ulm, Germany.";
//! let report = analyze(text);
//! println!("Trust: {:.0}% — {}", report.score * 100.0, report.risk_level);
//! ```

pub mod claim;
pub mod consistency;
pub mod entity;
pub mod mcp;
pub mod scorer;
pub mod trajectory;

pub use claim::{extract_claims, Claim};
pub use consistency::{check_consistency, ConsistencyReport, Contradiction};
pub use scorer::{score_claim, score_passage, RiskLevel, TrustScore};
pub use trajectory::{analyze_trajectory, TrajectoryAnalysis, TrajectoryPattern};

use serde::{Deserialize, Serialize};

/// Full analysis report for a text passage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustReport {
    /// Overall trust score (0.0 = likely hallucinated, 1.0 = likely factual)
    pub score: f64,
    /// Risk level classification
    pub risk_level: RiskLevel,
    /// Human-readable summary
    pub summary: String,
    /// Per-claim breakdown
    pub claims: Vec<ClaimReport>,
    /// Confidence trajectory analysis (v0.2)
    pub trajectory: TrajectoryAnalysis,
    /// Total number of claims analyzed
    pub total_claims: usize,
    /// Number of high-risk claims
    pub high_risk_claims: usize,
}

/// Analysis report for a single claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimReport {
    /// The claim text
    pub text: String,
    /// Trust score for this claim
    pub trust: TrustScore,
    /// Entity verification results (v0.4, requires --features verify)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<entity::VerificationResult>,
}

/// Analyze a text passage for hallucination risk.
///
/// This is the main entry point. It:
/// 1. Extracts atomic claims from the text
/// 2. Scores each claim individually
/// 3. Aggregates into a passage-level trust score
/// 4. Returns a detailed report
pub fn analyze(text: &str) -> TrustReport {
    let claims = extract_claims(text);
    let claim_reports: Vec<ClaimReport> = claims
        .iter()
        .map(|c| ClaimReport {
            text: c.text.clone(),
            trust: score_claim(c),
            verification: None,
        })
        .collect();

    let passage_score = score_passage(&claims);
    let traj = analyze_trajectory(&claims);

    // Apply trajectory modifier to the passage score
    let adjusted_score = (passage_score.score + traj.trust_modifier).clamp(0.0, 1.0);
    let adjusted_risk = scorer::classify_risk_pub(adjusted_score);

    let high_risk = claim_reports
        .iter()
        .filter(|c| {
            c.trust.risk_level == RiskLevel::High || c.trust.risk_level == RiskLevel::Critical
        })
        .count();

    let summary = format!("{} Trajectory: {}", passage_score.explanation, traj.pattern);

    TrustReport {
        score: adjusted_score,
        risk_level: adjusted_risk,
        summary,
        trajectory: traj,
        total_claims: claim_reports.len(),
        high_risk_claims: high_risk,
        claims: claim_reports,
    }
}

/// Analyze text with entity verification against Wikidata.
///
/// Like `analyze()`, but also runs entity extraction and (when the `verify`
/// feature is enabled) cross-references entities against Wikidata SPARQL.
/// The verification modifier adjusts the overall trust score by up to +/-0.15.
pub fn analyze_with_verification(text: &str) -> TrustReport {
    let claims = extract_claims(text);

    let claim_reports: Vec<ClaimReport> = claims
        .iter()
        .map(|c| {
            let verification = Some(entity::verify_claim(&c.text));
            ClaimReport {
                text: c.text.clone(),
                trust: score_claim(c),
                verification,
            }
        })
        .collect();

    let passage_score = score_passage(&claims);
    let traj = analyze_trajectory(&claims);

    // Collect verification results for modifier computation
    let verification_results: Vec<&entity::VerificationResult> = claim_reports
        .iter()
        .filter_map(|c| c.verification.as_ref())
        .collect();
    let owned_results: Vec<entity::VerificationResult> =
        verification_results.into_iter().cloned().collect();
    let verification_modifier = entity::compute_verification_modifier(&owned_results);

    // Apply both trajectory and verification modifiers
    let adjusted_score =
        (passage_score.score + traj.trust_modifier + verification_modifier).clamp(0.0, 1.0);
    let adjusted_risk = scorer::classify_risk_pub(adjusted_score);

    let high_risk = claim_reports
        .iter()
        .filter(|c| {
            c.trust.risk_level == RiskLevel::High || c.trust.risk_level == RiskLevel::Critical
        })
        .count();

    let summary = format!(
        "{} Trajectory: {} | Verification modifier: {:+.2}",
        passage_score.explanation, traj.pattern, verification_modifier
    );

    TrustReport {
        score: adjusted_score,
        risk_level: adjusted_risk,
        summary,
        trajectory: traj,
        total_claims: claim_reports.len(),
        high_risk_claims: high_risk,
        claims: claim_reports,
    }
}
