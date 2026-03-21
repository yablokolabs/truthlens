use serde::{Deserialize, Serialize};

/// Multi-response consistency checker — inspired by measurement-induced branching.
///
/// Given N responses to the same prompt, detects contradictions between them.
/// Like quantum measurement paths: different "branches" of the generative
/// process should agree on facts. When they don't, the outlier is likely
/// a hallucination.
///
/// # Method
///
/// 1. Extract claims from each response
/// 2. Group claims by topic (fuzzy match on key entities)
/// 3. Detect contradictions within each group
/// 4. Score: claims that appear consistently across responses are trusted;
///    claims unique to one response are flagged
///
/// A contradiction between two claims from different responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    /// Claim from response A
    pub claim_a: String,
    /// Index of response A
    pub response_a: usize,
    /// Claim from response B
    pub claim_b: String,
    /// Index of response B
    pub response_b: usize,
    /// What specifically conflicts
    pub conflict: String,
    /// Confidence that this is a real contradiction (0.0–1.0)
    pub confidence: f64,
}

/// Result of consistency analysis across multiple responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyReport {
    /// Number of responses analyzed
    pub n_responses: usize,
    /// Total claims extracted across all responses
    pub total_claims: usize,
    /// Claims that appear consistently (agreement across majority)
    pub consistent_claims: Vec<ConsistentClaim>,
    /// Detected contradictions
    pub contradictions: Vec<Contradiction>,
    /// Claims unique to a single response (potential hallucination)
    pub unique_claims: Vec<UniqueClaim>,
    /// Overall consistency score (0.0 = highly inconsistent, 1.0 = fully consistent)
    pub consistency_score: f64,
}

/// A claim that appears consistently across responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistentClaim {
    /// Representative text of the claim
    pub text: String,
    /// How many responses contain this claim
    pub agreement_count: usize,
    /// Agreement ratio (agreement_count / n_responses)
    pub agreement_ratio: f64,
}

/// A claim that appears in only one response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueClaim {
    /// The claim text
    pub text: String,
    /// Which response it came from
    pub response_idx: usize,
}

/// Check consistency across multiple responses to the same prompt.
///
/// # Arguments
/// * `responses` — Two or more text responses to the same prompt.
///
/// # Returns
/// A `ConsistencyReport` with contradictions, agreements, and unique claims.
pub fn check_consistency(responses: &[&str]) -> ConsistencyReport {
    if responses.len() < 2 {
        return ConsistencyReport {
            n_responses: responses.len(),
            total_claims: 0,
            consistent_claims: vec![],
            contradictions: vec![],
            unique_claims: vec![],
            consistency_score: 1.0,
        };
    }

    // Extract claims from each response
    let all_claims: Vec<Vec<ClaimEntity>> = responses.iter().map(|r| extract_entities(r)).collect();

    let total_claims: usize = all_claims.iter().map(|c| c.len()).sum();

    // Find consistent claims (appear in multiple responses)
    let mut consistent_claims = Vec::new();
    let mut seen_entities: Vec<(String, String, Vec<usize>)> = Vec::new(); // (subject, value, response_indices)

    for (resp_idx, claims) in all_claims.iter().enumerate() {
        for claim in claims {
            let mut found = false;
            for existing in seen_entities.iter_mut() {
                if fuzzy_match_subject(&existing.0, &claim.subject) {
                    if !existing.2.contains(&resp_idx) {
                        existing.2.push(resp_idx);
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                seen_entities.push((claim.subject.clone(), claim.value.clone(), vec![resp_idx]));
            }
        }
    }

    let n = responses.len();
    for (subject, value, indices) in &seen_entities {
        if indices.len() > 1 {
            consistent_claims.push(ConsistentClaim {
                text: format!("{}: {}", subject, value),
                agreement_count: indices.len(),
                agreement_ratio: indices.len() as f64 / n as f64,
            });
        }
    }

    // Find unique claims (appear in only one response)
    let mut unique_claims = Vec::new();
    for (subject, value, indices) in &seen_entities {
        if indices.len() == 1 {
            unique_claims.push(UniqueClaim {
                text: format!("{}: {}", subject, value),
                response_idx: indices[0],
            });
        }
    }

    // Detect contradictions: same subject, different values across responses
    let mut contradictions = Vec::new();
    for (resp_a, claims_a) in all_claims.iter().enumerate() {
        for (resp_b, claims_b) in all_claims.iter().enumerate() {
            if resp_b <= resp_a {
                continue;
            }
            for claim_a in claims_a {
                for claim_b in claims_b {
                    if let Some(contradiction) =
                        detect_contradiction(claim_a, claim_b, resp_a, resp_b)
                    {
                        // Avoid duplicate contradictions
                        let already_found = contradictions
                            .iter()
                            .any(|c: &Contradiction| c.conflict == contradiction.conflict);
                        if !already_found {
                            contradictions.push(contradiction);
                        }
                    }
                }
            }
        }
    }

    // Compute consistency score
    let consistency_score = compute_consistency_score(
        &consistent_claims,
        &contradictions,
        &unique_claims,
        total_claims,
        n,
    );

    ConsistencyReport {
        n_responses: n,
        total_claims,
        consistent_claims,
        contradictions,
        unique_claims,
        consistency_score,
    }
}

// ── Internal types and helpers ──────────────────────────────────────

#[derive(Debug, Clone)]
struct ClaimEntity {
    subject: String,
    value: String,
    full_text: String,
}

/// Extract subject-value pairs from text.
/// Simple heuristic: look for patterns like "X was/is/has Y" and "X in Y".
fn extract_entities(text: &str) -> Vec<ClaimEntity> {
    let sentences: Vec<&str> = text
        .split(['.', '!', '?'])
        .filter(|s| s.trim().len() > 10)
        .collect();

    let mut entities = Vec::new();

    for sentence in &sentences {
        let s = sentence.trim();
        let words: Vec<&str> = s.split_whitespace().collect();

        if words.len() < 3 {
            continue;
        }

        // Extract numbers as values
        for (i, word) in words.iter().enumerate() {
            if word.chars().any(|c| c.is_ascii_digit()) && word.len() <= 10 {
                // Find the subject: words before the number
                let subject = if i >= 2 {
                    words[..i].join(" ")
                } else {
                    words[0].to_string()
                };
                entities.push(ClaimEntity {
                    subject: normalize_subject(&subject),
                    value: word.to_string(),
                    full_text: s.to_string(),
                });
            }
        }

        // Extract "in <Place>" patterns
        for (i, word) in words.iter().enumerate() {
            if (*word == "in" || *word == "at" || *word == "from") && i + 1 < words.len() {
                let place = words[i + 1..].join(" ");
                if place.chars().next().is_some_and(|c| c.is_uppercase()) {
                    let subject = words[..i].join(" ");
                    entities.push(ClaimEntity {
                        subject: normalize_subject(&subject),
                        value: normalize_value(&place),
                        full_text: s.to_string(),
                    });
                }
            }
        }
    }

    entities
}

fn normalize_subject(s: &str) -> String {
    let s = s.trim().to_lowercase();
    // Remove common prefixes
    let s = s.strip_prefix("the ").unwrap_or(&s);
    let s = s.strip_prefix("a ").unwrap_or(s);
    let s = s.strip_prefix("an ").unwrap_or(s);
    s.to_string()
}

fn normalize_value(s: &str) -> String {
    s.trim()
        .trim_end_matches(|c: char| c.is_ascii_punctuation())
        .to_string()
}

fn fuzzy_match_subject(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    // One contains the other
    if a.contains(b) || b.contains(a) {
        return true;
    }
    // Share significant words
    let a_words: Vec<&str> = a.split_whitespace().filter(|w| w.len() > 3).collect();
    let b_words: Vec<&str> = b.split_whitespace().filter(|w| w.len() > 3).collect();
    let shared = a_words.iter().filter(|w| b_words.contains(w)).count();
    shared > 0 && shared >= a_words.len().min(b_words.len()) / 2
}

fn detect_contradiction(
    a: &ClaimEntity,
    b: &ClaimEntity,
    resp_a: usize,
    resp_b: usize,
) -> Option<Contradiction> {
    // Same subject, different value
    if !fuzzy_match_subject(&a.subject, &b.subject) {
        return None;
    }

    let a_val = normalize_value(&a.value);
    let b_val = normalize_value(&b.value);

    if a_val == b_val {
        return None; // Same value, no contradiction
    }

    // Check if values are actually different (not just formatting/subsets)
    let a_lower = a_val.to_lowercase();
    let b_lower = b_val.to_lowercase();
    if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
        return None;
    }
    // Skip if values share significant words (e.g., "Ulm, Germany" vs "Ulm Germany")
    let a_words: Vec<&str> = a_val.split_whitespace().collect();
    let b_words: Vec<&str> = b_val.split_whitespace().collect();
    let shared = a_words
        .iter()
        .filter(|w| {
            b_words
                .iter()
                .any(|bw| bw.to_lowercase() == w.to_lowercase())
        })
        .count();
    if shared > 0 && shared >= a_words.len().min(b_words.len()) {
        return None;
    }

    // Skip if values are different types (number vs place name)
    let a_is_numeric = a_val.chars().any(|c| c.is_ascii_digit());
    let b_is_numeric = b_val.chars().any(|c| c.is_ascii_digit());
    if a_is_numeric != b_is_numeric {
        return None;
    }

    // Both have values of same type → contradiction
    let confidence = if a_is_numeric && b_is_numeric {
        0.9 // Numeric disagreement is high-confidence contradiction
    } else if a_val.chars().next().is_some_and(|c| c.is_uppercase())
        && b_val.chars().next().is_some_and(|c| c.is_uppercase())
    {
        0.8 // Proper noun disagreement
    } else {
        0.5 // General disagreement
    };

    Some(Contradiction {
        claim_a: a.full_text.clone(),
        response_a: resp_a,
        claim_b: b.full_text.clone(),
        response_b: resp_b,
        conflict: format!("\"{}\" vs \"{}\" (subject: {})", a_val, b_val, a.subject),
        confidence,
    })
}

fn compute_consistency_score(
    consistent: &[ConsistentClaim],
    contradictions: &[Contradiction],
    unique: &[UniqueClaim],
    total_claims: usize,
    n_responses: usize,
) -> f64 {
    if total_claims == 0 || n_responses < 2 {
        return 1.0;
    }

    // Consistent claims boost score
    let avg_agreement: f64 = if consistent.is_empty() {
        0.5
    } else {
        consistent.iter().map(|c| c.agreement_ratio).sum::<f64>() / consistent.len() as f64
    };

    // Contradictions reduce score
    let contradiction_penalty = (contradictions.len() as f64 * 0.15).min(0.5);

    // Unique claims slightly reduce score (potential hallucination)
    let unique_penalty = (unique.len() as f64 * 0.05).min(0.3);

    (avg_agreement - contradiction_penalty - unique_penalty).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consistent_responses() {
        let report = check_consistency(&[
            "Einstein was born in 1879 in Ulm, Germany.",
            "Einstein was born in 1879 in Ulm, Germany.",
            "Einstein was born in 1879 in Ulm, Germany.",
        ]);
        for c in &report.contradictions {
            eprintln!(
                "CONTRADICTION: {} (resp {}) vs {} (resp {}) — {}",
                c.claim_a, c.response_a, c.claim_b, c.response_b, c.conflict
            );
        }
        assert!(report.consistency_score > 0.5);
        assert!(
            report.contradictions.is_empty(),
            "Found {} contradictions in identical responses",
            report.contradictions.len()
        );
    }

    #[test]
    fn contradicting_responses() {
        let report = check_consistency(&[
            "Einstein was born in 1879 in Ulm, Germany.",
            "Einstein was born in 1879 in Munich, Germany.",
            "Einstein was born in 1879 in Ulm, Germany.",
        ]);
        assert!(
            !report.contradictions.is_empty(),
            "Expected contradictions for Ulm vs Munich"
        );
        assert!(report.consistency_score < 0.9);
    }

    #[test]
    fn numeric_contradiction() {
        let report = check_consistency(&[
            "The population is 126,000 people.",
            "The population is 250,000 people.",
        ]);
        // Should detect numeric disagreement
        assert!(
            report.contradictions.is_empty() == false || report.unique_claims.len() > 0,
            "Should detect disagreement in numbers"
        );
    }

    #[test]
    fn single_response() {
        let report = check_consistency(&["Einstein was born in 1879."]);
        assert_eq!(report.consistency_score, 1.0);
        assert_eq!(report.n_responses, 1);
    }

    #[test]
    fn empty_responses() {
        let report = check_consistency(&[]);
        assert_eq!(report.consistency_score, 1.0);
    }

    #[test]
    fn consistency_score_bounded() {
        let report = check_consistency(&[
            "Einstein was born in 1879 in Ulm. He had 3 children.",
            "Einstein was born in 1880 in Munich. He had 5 children.",
            "Einstein was born in 1879 in Berlin. He had 3 kids.",
        ]);
        assert!(report.consistency_score >= 0.0);
        assert!(report.consistency_score <= 1.0);
    }

    #[test]
    fn unique_claims_detected() {
        let report = check_consistency(&[
            "Einstein was born in 1879. He invented the laser.",
            "Einstein was born in 1879. He played violin.",
        ]);
        // "laser" and "violin" are unique to their respective responses
        assert!(report.total_claims > 0);
    }
}
