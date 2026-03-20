use crate::claim::Claim;
use serde::{Deserialize, Serialize};

/// Trust score for an individual claim or entire passage.
///
/// Score ranges from 0.0 (likely hallucinated) to 1.0 (likely factual).
/// The score is composed from multiple signals:
///   - confidence_signal: linguistic confidence markers
///   - specificity_signal: how concrete/specific the claim is
///   - hedging_signal: presence of uncertainty markers
///   - consistency_signal: self-consistency across samples (optional)
///   - verifiability_signal: whether the claim can be fact-checked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    /// Overall trust score in [0.0, 1.0]
    pub score: f64,
    /// Individual signal contributions
    pub signals: ScoreSignals,
    /// Human-readable risk level
    pub risk_level: RiskLevel,
    /// Explanation of the score
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreSignals {
    /// Confidence signal: overconfident language without hedging (0 = overconfident, 1 = well-calibrated)
    pub confidence: f64,
    /// Specificity signal: vague claims score lower (0 = vague, 1 = specific)
    pub specificity: f64,
    /// Hedging signal: hedged claims are less likely hallucinated (0 = no hedge, 1 = well-hedged)
    pub hedging: f64,
    /// Verifiability signal: verifiable claims with high confidence are riskier if wrong (0 = risky, 1 = safe)
    pub verifiability: f64,
    /// Consistency signal: set by external N-sample consistency check (0 = inconsistent, 1 = consistent)
    pub consistency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Score a single claim for hallucination risk.
pub fn score_claim(claim: &Claim) -> TrustScore {
    let signals = compute_signals(claim);
    let score = aggregate_score(&signals);
    let risk_level = classify_risk(score);
    let explanation = explain_score(claim, &signals, &risk_level);

    TrustScore {
        score,
        signals,
        risk_level,
        explanation,
    }
}

/// Score an entire passage by aggregating claim-level scores.
pub fn score_passage(claims: &[Claim]) -> TrustScore {
    if claims.is_empty() {
        return TrustScore {
            score: 0.5,
            signals: ScoreSignals {
                confidence: 0.5,
                specificity: 0.5,
                hedging: 0.5,
                verifiability: 0.5,
                consistency: None,
            },
            risk_level: RiskLevel::Medium,
            explanation: "No claims to analyze.".to_string(),
        };
    }

    let claim_scores: Vec<TrustScore> = claims.iter().map(score_claim).collect();
    let n = claim_scores.len() as f64;

    let avg_score = claim_scores.iter().map(|s| s.score).sum::<f64>() / n;
    let min_score = claim_scores.iter().map(|s| s.score).fold(f64::INFINITY, f64::min);

    // Passage score is weighted: 70% average + 30% worst claim
    // This penalizes passages with even one highly suspicious claim
    let passage_score = 0.7 * avg_score + 0.3 * min_score;
    let passage_score = passage_score.clamp(0.0, 1.0);

    let avg_signals = ScoreSignals {
        confidence: claim_scores.iter().map(|s| s.signals.confidence).sum::<f64>() / n,
        specificity: claim_scores.iter().map(|s| s.signals.specificity).sum::<f64>() / n,
        hedging: claim_scores.iter().map(|s| s.signals.hedging).sum::<f64>() / n,
        verifiability: claim_scores.iter().map(|s| s.signals.verifiability).sum::<f64>() / n,
        consistency: None,
    };

    let risk_level = classify_risk(passage_score);
    let n_high_risk = claim_scores.iter().filter(|s| s.risk_level == RiskLevel::High || s.risk_level == RiskLevel::Critical).count();

    let explanation = format!(
        "{} claims analyzed. {} high-risk claims detected. Average trust: {:.0}%.",
        claims.len(),
        n_high_risk,
        passage_score * 100.0
    );

    TrustScore {
        score: passage_score,
        signals: avg_signals,
        risk_level,
        explanation,
    }
}

fn compute_signals(claim: &Claim) -> ScoreSignals {
    // Confidence signal: overconfident + specific + not hedged = risky
    // Well-calibrated claims either hedge or are genuinely verifiable
    let confidence = if claim.is_hedged {
        0.8 // Hedged claims are well-calibrated
    } else if claim.specificity > 0.5 && claim.is_verifiable {
        0.4 // Specific verifiable claims without hedging: risky if wrong
    } else if claim.specificity > 0.3 {
        0.6 // Moderate specificity, no hedging
    } else {
        0.5 // Vague, no hedging: unclear
    };

    // Specificity: more specific = more verifiable = better if correct
    // But also more damaging if hallucinated
    // We score higher for specific claims (they're more useful)
    let specificity = claim.specificity;

    // Hedging: hedged claims are more trustworthy (model knows its limits)
    let hedging = if claim.is_hedged { 0.85 } else { 0.5 };

    // Verifiability: verifiable claims can be checked → higher baseline trust
    // (if the model makes a checkable claim, it's either right or catchable)
    let verifiability = if claim.is_verifiable { 0.7 } else { 0.5 };

    ScoreSignals {
        confidence,
        specificity,
        hedging,
        verifiability,
        consistency: None,
    }
}

/// Aggregate signals into a single score.
///
/// Weights:
///   confidence: 35% (most important — overconfidence is the hallucination signal)
///   hedging: 25% (hedging strongly correlates with lower hallucination)
///   specificity: 20% (specific claims are more useful and checkable)
///   verifiability: 15% (verifiable claims have external recourse)
///   consistency: 5% bonus when available
const W_CONFIDENCE: f64 = 0.35;
const W_HEDGING: f64 = 0.25;
const W_SPECIFICITY: f64 = 0.20;
const W_VERIFIABILITY: f64 = 0.15;
const W_CONSISTENCY: f64 = 0.05;

fn aggregate_score(signals: &ScoreSignals) -> f64 {
    let base = W_CONFIDENCE * signals.confidence
        + W_HEDGING * signals.hedging
        + W_SPECIFICITY * signals.specificity
        + W_VERIFIABILITY * signals.verifiability;

    let consistency_bonus = signals.consistency.unwrap_or(0.5) * W_CONSISTENCY;

    (base + consistency_bonus).clamp(0.0, 1.0)
}

fn classify_risk(score: f64) -> RiskLevel {
    if score >= 0.75 {
        RiskLevel::Low
    } else if score >= 0.55 {
        RiskLevel::Medium
    } else if score >= 0.35 {
        RiskLevel::High
    } else {
        RiskLevel::Critical
    }
}

fn explain_score(claim: &Claim, signals: &ScoreSignals, risk: &RiskLevel) -> String {
    let mut reasons = Vec::new();

    if signals.confidence < 0.5 {
        reasons.push("overconfident language without hedging");
    }
    if claim.is_hedged {
        reasons.push("appropriately hedged");
    }
    if claim.is_verifiable && !claim.is_hedged {
        reasons.push("specific verifiable claim — verify independently");
    }
    if claim.specificity < 0.3 {
        reasons.push("vague claim with low specificity");
    }
    if claim.specificity > 0.6 {
        reasons.push("highly specific claim");
    }

    if reasons.is_empty() {
        reasons.push("no strong signals detected");
    }

    format!(
        "[{risk}] Trust: {:.0}% — {}",
        signals.confidence * 100.0,
        reasons.join("; ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::extract_claims;

    #[test]
    fn score_is_bounded() {
        let claims = extract_claims("Einstein was born in 1879. The sky might be purple.");
        for claim in &claims {
            let score = score_claim(claim);
            assert!(score.score >= 0.0 && score.score <= 1.0,
                "Score out of bounds: {}", score.score);
        }
    }

    #[test]
    fn hedged_claims_score_higher() {
        let hedged = Claim {
            text: "This might be related to the discovery.".to_string(),
            sentence_idx: 0,
            is_verifiable: false,
            specificity: 0.3,
            is_hedged: true,
        };
        let confident = Claim {
            text: "Einstein discovered exactly 47 particles in 1903.".to_string(),
            sentence_idx: 0,
            is_verifiable: true,
            specificity: 0.8,
            is_hedged: false,
        };
        let hedged_score = score_claim(&hedged);
        let confident_score = score_claim(&confident);
        assert!(hedged_score.score > confident_score.score,
            "Hedged {:.3} should score higher than overconfident {:.3}",
            hedged_score.score, confident_score.score);
    }

    #[test]
    fn passage_scoring() {
        let claims = extract_claims(
            "Albert Einstein was born in 1879. He might have visited Paris. \
             The theory of relativity was published in exactly 1905."
        );
        let passage = score_passage(&claims);
        assert!(passage.score >= 0.0 && passage.score <= 1.0);
        assert!(!passage.explanation.is_empty());
    }

    #[test]
    fn empty_passage() {
        let passage = score_passage(&[]);
        assert_eq!(passage.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn risk_classification() {
        assert_eq!(classify_risk(0.8), RiskLevel::Low);
        assert_eq!(classify_risk(0.6), RiskLevel::Medium);
        assert_eq!(classify_risk(0.4), RiskLevel::High);
        assert_eq!(classify_risk(0.2), RiskLevel::Critical);
    }
}
