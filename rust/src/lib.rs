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
pub mod scorer;

pub use claim::{extract_claims, Claim};
pub use scorer::{score_claim, score_passage, RiskLevel, TrustScore};

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
        })
        .collect();

    let passage_score = score_passage(&claims);
    let high_risk = claim_reports
        .iter()
        .filter(|c| c.trust.risk_level == RiskLevel::High || c.trust.risk_level == RiskLevel::Critical)
        .count();

    TrustReport {
        score: passage_score.score,
        risk_level: passage_score.risk_level,
        summary: passage_score.explanation,
        total_claims: claim_reports.len(),
        high_risk_claims: high_risk,
        claims: claim_reports,
    }
}
