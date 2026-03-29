use crate::claim::Claim;
use serde::{Deserialize, Serialize};

/// Confidence trajectory analysis — inspired by control theory (laplace-gm).
///
/// Maps the confidence pattern across a passage to a second-order system model:
///
///   - **Underdamped (oscillating confidence)**: confident → uncertain → confident
///     → HIGH hallucination risk. Like an overfit model oscillating around truth.
///
///   - **Overdamped (consistently cautious)**: uniformly hedged throughout
///     → LOW hallucination risk. Model knows its limits.
///
///   - **Critically damped (smooth convergence)**: starts uncertain, becomes
///     progressively more confident → GOOD pattern. Model builds on established facts.
///
///   - **Flat high confidence**: uniformly confident with no hedging
///     → SUSPICIOUS. Real knowledge has nuance.
///
/// # Per-sentence confidence level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfidenceLevel {
    /// Strong definitive language, no hedging
    High,
    /// Moderate — some specifics but not overconfident
    Medium,
    /// Hedged, uncertain, or vague
    Low,
}

/// Trajectory pattern classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrajectoryPattern {
    /// Confidence oscillates: H→L→H or L→H→L (underdamped)
    Oscillating,
    /// Uniformly high confidence throughout (flat, suspicious)
    FlatHigh,
    /// Uniformly low/hedged throughout (overdamped, cautious)
    FlatLow,
    /// Confidence increases over the passage (convergent)
    Convergent,
    /// Confidence decreases over the passage (divergent, losing confidence)
    Divergent,
    /// Mixed/no clear pattern
    Mixed,
    /// Too few claims to determine pattern
    Insufficient,
}

impl std::fmt::Display for TrajectoryPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrajectoryPattern::Oscillating => {
                write!(f, "OSCILLATING (underdamped — hallucination risk)")
            }
            TrajectoryPattern::FlatHigh => {
                write!(f, "FLAT HIGH (uniformly overconfident — suspicious)")
            }
            TrajectoryPattern::FlatLow => {
                write!(f, "FLAT LOW (consistently cautious — trustworthy)")
            }
            TrajectoryPattern::Convergent => {
                write!(f, "CONVERGENT (builds confidence — good pattern)")
            }
            TrajectoryPattern::Divergent => write!(f, "DIVERGENT (losing confidence — uncertain)"),
            TrajectoryPattern::Mixed => write!(f, "MIXED (no clear pattern)"),
            TrajectoryPattern::Insufficient => write!(f, "INSUFFICIENT (too few claims)"),
        }
    }
}

/// Full trajectory analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryAnalysis {
    /// Classified pattern
    pub pattern: TrajectoryPattern,
    /// Damping ratio estimate (0 = oscillating, 1 = critically damped, >1 = overdamped)
    pub damping_estimate: f64,
    /// Trust modifier from trajectory analysis (-0.15 to +0.15)
    pub trust_modifier: f64,
    /// Number of confidence transitions (direction changes)
    pub transitions: usize,
    /// Explanation
    pub explanation: String,
}

/// Analyze the confidence trajectory across claims.
pub fn analyze_trajectory(claims: &[Claim]) -> TrajectoryAnalysis {
    if claims.len() < 3 {
        return TrajectoryAnalysis {
            pattern: TrajectoryPattern::Insufficient,
            damping_estimate: 1.0,
            trust_modifier: 0.0,
            transitions: 0,
            explanation: "Too few claims for trajectory analysis.".to_string(),
        };
    }

    // Compute per-claim confidence levels
    let levels: Vec<ConfidenceLevel> = claims.iter().map(classify_confidence).collect();
    let scores: Vec<f64> = levels
        .iter()
        .map(|l| match l {
            ConfidenceLevel::High => 1.0,
            ConfidenceLevel::Medium => 0.5,
            ConfidenceLevel::Low => 0.0,
        })
        .collect();

    // Count transitions (direction changes in confidence)
    let transitions = count_transitions(&scores);

    // Compute trend (is confidence increasing or decreasing?)
    let trend = compute_trend(&scores);

    // Classify the pattern
    let avg_confidence = scores.iter().sum::<f64>() / scores.len() as f64;
    let variance = scores
        .iter()
        .map(|s| (s - avg_confidence).powi(2))
        .sum::<f64>()
        / scores.len() as f64;

    let pattern = classify_pattern(transitions, trend, avg_confidence, variance, scores.len());

    // Map to damping estimate
    let damping_estimate = match &pattern {
        TrajectoryPattern::Oscillating => 0.2 + 0.3 * (1.0 / (transitions as f64 + 1.0)),
        TrajectoryPattern::FlatHigh => 0.1, // Looks stable but suspiciously so
        TrajectoryPattern::FlatLow => 1.5,  // Overdamped — cautious
        TrajectoryPattern::Convergent => 0.9, // Nearly critical — good
        TrajectoryPattern::Divergent => 0.6, // Moderately underdamped
        TrajectoryPattern::Mixed => 0.7,
        TrajectoryPattern::Insufficient => 1.0,
    };

    // Trust modifier: how much the trajectory adjusts the base score
    let trust_modifier = match &pattern {
        TrajectoryPattern::FlatLow => 0.10, // Consistent caution → bonus
        TrajectoryPattern::Convergent => 0.08, // Good pattern → bonus
        TrajectoryPattern::Mixed => 0.0,    // Neutral
        TrajectoryPattern::Divergent => -0.05, // Losing confidence → slight penalty
        TrajectoryPattern::Oscillating => -0.10, // Oscillation → penalty
        TrajectoryPattern::FlatHigh => -0.12, // Uniform overconfidence → penalty
        TrajectoryPattern::Insufficient => 0.0,
    };

    let explanation = format!(
        "{pattern} — damping ζ≈{damping_estimate:.2}, {} transitions, avg confidence {:.0}%, variance {:.3}",
        transitions,
        avg_confidence * 100.0,
        variance
    );

    TrajectoryAnalysis {
        pattern,
        damping_estimate,
        trust_modifier,
        transitions,
        explanation,
    }
}

fn classify_confidence(claim: &Claim) -> ConfidenceLevel {
    if claim.is_hedged {
        return ConfidenceLevel::Low;
    }
    if claim.specificity > 0.5 && claim.is_verifiable {
        return ConfidenceLevel::High;
    }
    if claim.specificity < 0.25 {
        return ConfidenceLevel::Low;
    }
    ConfidenceLevel::Medium
}

fn count_transitions(scores: &[f64]) -> usize {
    if scores.len() < 2 {
        return 0;
    }
    let mut transitions = 0;
    let mut prev_direction: Option<bool> = None; // true = increasing

    for window in scores.windows(2) {
        let diff = window[1] - window[0];
        if diff.abs() < 0.1 {
            continue; // No significant change
        }
        let increasing = diff > 0.0;
        if let Some(prev) = prev_direction
            && prev != increasing
        {
            transitions += 1;
        }
        prev_direction = Some(increasing);
    }
    transitions
}

fn compute_trend(scores: &[f64]) -> f64 {
    // Simple linear regression slope
    let n = scores.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = scores.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den = 0.0;
    for (i, s) in scores.iter().enumerate() {
        let x = i as f64 - x_mean;
        let y = s - y_mean;
        num += x * y;
        den += x * x;
    }

    if den.abs() < 1e-10 { 0.0 } else { num / den }
}

fn classify_pattern(
    transitions: usize,
    trend: f64,
    avg_confidence: f64,
    variance: f64,
    n_claims: usize,
) -> TrajectoryPattern {
    // Oscillating: multiple direction changes
    if transitions >= 2 && variance > 0.1 {
        return TrajectoryPattern::Oscillating;
    }

    // Flat: low variance
    if variance < 0.05 {
        if avg_confidence > 0.7 {
            return TrajectoryPattern::FlatHigh;
        } else if avg_confidence < 0.3 {
            return TrajectoryPattern::FlatLow;
        }
    }

    // Convergent: strong positive trend
    if trend > 0.1 && transitions <= 1 {
        return TrajectoryPattern::Convergent;
    }

    // Divergent: strong negative trend
    if trend < -0.1 && transitions <= 1 {
        return TrajectoryPattern::Divergent;
    }

    // Need enough claims for the above
    if n_claims < 4 {
        return TrajectoryPattern::Mixed;
    }

    TrajectoryPattern::Mixed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claim(text: &str, specificity: f64, verifiable: bool, hedged: bool) -> Claim {
        Claim {
            text: text.to_string(),
            sentence_idx: 0,
            is_verifiable: verifiable,
            specificity,
            is_hedged: hedged,
        }
    }

    #[test]
    fn oscillating_pattern() {
        let claims = vec![
            make_claim("Einstein was born in 1879.", 0.7, true, false), // High
            make_claim("Something might be related.", 0.2, false, true), // Low
            make_claim("He published exactly 300 papers.", 0.8, true, false), // High
            make_claim("Perhaps this is unclear.", 0.1, false, true),   // Low
            make_claim("The answer is precisely 42.", 0.9, true, false), // High
        ];
        let analysis = analyze_trajectory(&claims);
        assert_eq!(analysis.pattern, TrajectoryPattern::Oscillating);
        assert!(analysis.trust_modifier < 0.0);
    }

    #[test]
    fn flat_high_pattern() {
        let claims = vec![
            make_claim("Einstein discovered 47 particles.", 0.8, true, false),
            make_claim("He won exactly 3 Nobel prizes.", 0.9, true, false),
            make_claim("The theory has 12 dimensions.", 0.7, true, false),
            make_claim("Physics explains 99.9% of reality.", 0.8, true, false),
        ];
        let analysis = analyze_trajectory(&claims);
        assert_eq!(analysis.pattern, TrajectoryPattern::FlatHigh);
        assert!(analysis.trust_modifier < 0.0);
    }

    #[test]
    fn flat_low_pattern() {
        let claims = vec![
            make_claim("This might be true.", 0.2, false, true),
            make_claim("Perhaps it could work.", 0.1, false, true),
            make_claim("It is possible that things improve.", 0.15, false, true),
            make_claim("Some evidence suggests this.", 0.2, false, true),
        ];
        let analysis = analyze_trajectory(&claims);
        assert_eq!(analysis.pattern, TrajectoryPattern::FlatLow);
        assert!(analysis.trust_modifier > 0.0);
    }

    #[test]
    fn insufficient_claims() {
        let claims = vec![
            make_claim("Hello.", 0.1, false, false),
            make_claim("World.", 0.1, false, false),
        ];
        let analysis = analyze_trajectory(&claims);
        assert_eq!(analysis.pattern, TrajectoryPattern::Insufficient);
        assert_eq!(analysis.trust_modifier, 0.0);
    }

    #[test]
    fn damping_bounded() {
        let claims = vec![
            make_claim("A.", 0.5, true, false),
            make_claim("B.", 0.5, false, true),
            make_claim("C.", 0.5, true, false),
            make_claim("D.", 0.5, false, true),
        ];
        let analysis = analyze_trajectory(&claims);
        assert!(analysis.damping_estimate > 0.0);
        assert!(analysis.damping_estimate < 2.0);
    }
}
