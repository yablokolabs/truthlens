//! Entity Cross-Reference with Wikidata SPARQL (v0.4)
//!
//! Extracts named entities (people, places, dates) from claims and optionally
//! verifies them against Wikidata's SPARQL endpoint.

use serde::{Deserialize, Serialize};

/// A matched entity from Wikidata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatch {
    /// The entity name as extracted from text
    pub entity_name: String,
    /// Wikidata QID (e.g., "Q937" for Albert Einstein)
    pub wikidata_id: Option<String>,
    /// Properties verified against Wikidata (e.g., ["birth year: 1879", "birthplace: Ulm"])
    pub verified_properties: Vec<String>,
    /// Confidence in the match (0.0–1.0)
    pub confidence: f64,
}

/// Verification result for a single claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// The claim text
    pub claim_text: String,
    /// Entity matches found for this claim
    pub matches: Vec<EntityMatch>,
    /// Overall verification status
    pub status: VerificationStatus,
}

/// Status of entity verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    /// At least one entity was verified against Wikidata
    Verified,
    /// An entity was found but properties contradict the claim
    Contradicted,
    /// No entities could be verified (not found or no network)
    Unknown,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationStatus::Verified => write!(f, "VERIFIED"),
            VerificationStatus::Contradicted => write!(f, "CONTRADICTED"),
            VerificationStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Extract named entities from a claim text.
///
/// Pulls out:
/// - Capitalized multi-word names (e.g., "Albert Einstein", "New York")
/// - 4-digit years (e.g., "1879", "2024")
pub fn extract_entities_from_claim(claim: &str) -> Vec<String> {
    let mut entities = Vec::new();

    // Extract capitalized multi-word names
    let words: Vec<&str> = claim.split_whitespace().collect();
    let mut i = 0;
    while i < words.len() {
        let word = words[i].trim_matches(|c: char| c.is_ascii_punctuation());
        if word.len() > 1
            && word.chars().next().is_some_and(|c| c.is_uppercase())
            && word.chars().skip(1).any(|c| c.is_lowercase())
            && !is_skip_word(word)
            && (i > 0 || (i == 0 && words.len() > 1 && is_capitalized_name_word(word)))
        {
            // Start of a potential multi-word name
            let mut name_parts = vec![word.to_string()];
            let mut j = i + 1;
            while j < words.len() {
                let next = words[j].trim_matches(|c: char| c.is_ascii_punctuation());
                if next.len() > 1
                    && next.chars().next().is_some_and(|c| c.is_uppercase())
                    && next.chars().skip(1).any(|c| c.is_lowercase())
                    && !is_skip_word(next)
                {
                    name_parts.push(next.to_string());
                    j += 1;
                } else {
                    break;
                }
            }
            if name_parts.len() >= 2 {
                entities.push(name_parts.join(" "));
                i = j;
                continue;
            } else if i > 0 && is_capitalized_name_word(word) {
                // Single capitalized word after start of sentence — only include if
                // it looks like a proper noun (not a common word)
                entities.push(word.to_string());
            }
        }
        i += 1;
    }

    // Extract 4-digit years
    let year_re = regex::Regex::new(r"\b(\d{4})\b").unwrap();
    for cap in year_re.captures_iter(claim) {
        let year: u32 = cap[1].parse().unwrap_or(0);
        if (1000..=2100).contains(&year) {
            entities.push(cap[1].to_string());
        }
    }

    entities.sort();
    entities.dedup();
    entities
}

fn is_skip_word(word: &str) -> bool {
    matches!(
        word,
        "The"
            | "This"
            | "That"
            | "These"
            | "Those"
            | "It"
            | "They"
            | "He"
            | "She"
            | "We"
            | "However"
            | "Although"
            | "Moreover"
            | "Furthermore"
            | "Additionally"
            | "In"
            | "On"
            | "At"
            | "For"
            | "With"
            | "But"
            | "And"
            | "Or"
            | "As"
            | "If"
            | "Was"
            | "Were"
            | "Is"
            | "Are"
            | "Has"
            | "Had"
            | "Have"
            | "His"
            | "Her"
            | "Its"
            | "Their"
            | "Some"
            | "Many"
            | "Several"
            | "Each"
            | "Every"
            | "No"
            | "Not"
            | "By"
            | "From"
            | "To"
            | "Of"
            | "A"
            | "An"
    )
}

fn is_capitalized_name_word(word: &str) -> bool {
    // Heuristic: likely a proper noun if it's capitalized and not a common English word
    word.len() > 1
        && word.chars().next().is_some_and(|c| c.is_uppercase())
        && word.chars().skip(1).any(|c| c.is_lowercase())
        && !is_skip_word(word)
}

/// Compute a trust score modifier based on verification results.
///
/// Returns a value in [-0.15, +0.15] to adjust the claim's trust score.
/// - Verified entities increase trust
/// - Contradicted entities decrease trust
/// - Unknown entities have no effect
pub fn compute_verification_modifier(results: &[VerificationResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }

    let mut verified_count = 0;
    let mut contradicted_count = 0;
    let mut total = 0;

    for result in results {
        total += 1;
        match result.status {
            VerificationStatus::Verified => verified_count += 1,
            VerificationStatus::Contradicted => contradicted_count += 1,
            VerificationStatus::Unknown => {}
        }
    }

    if total == 0 {
        return 0.0;
    }

    let verified_ratio = verified_count as f64 / total as f64;
    let contradicted_ratio = contradicted_count as f64 / total as f64;

    // Range: -0.15 (all contradicted) to +0.15 (all verified)
    let modifier = 0.15 * verified_ratio - 0.15 * contradicted_ratio;
    modifier.clamp(-0.15, 0.15)
}

// ── Wikidata SPARQL verification (requires "verify" feature) ────────────

#[cfg(feature = "verify")]
pub fn verify_entities(entities: &[String]) -> Vec<EntityMatch> {
    let mut results = Vec::new();

    for (i, entity) in entities.iter().enumerate() {
        // Rate limit: sleep 1s between queries (skip before first)
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // Skip years — they aren't entities to look up directly
        if entity.len() == 4 && entity.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        match query_wikidata_entity(entity) {
            Ok(Some(m)) => results.push(m),
            Ok(None) => results.push(EntityMatch {
                entity_name: entity.clone(),
                wikidata_id: None,
                verified_properties: vec![],
                confidence: 0.0,
            }),
            Err(e) => {
                eprintln!("Warning: Wikidata query failed for '{}': {}", entity, e);
                results.push(EntityMatch {
                    entity_name: entity.clone(),
                    wikidata_id: None,
                    verified_properties: vec![],
                    confidence: 0.0,
                });
            }
        }
    }

    results
}

#[cfg(feature = "verify")]
fn query_wikidata_entity(entity: &str) -> Result<Option<EntityMatch>, String> {
    // Determine if this looks like a person name (two+ capitalized words)
    // or a place (single or multi-word)
    let words: Vec<&str> = entity.split_whitespace().collect();
    let is_person = words.len() >= 2
        && words
            .iter()
            .all(|w| w.chars().next().is_some_and(|c| c.is_uppercase()));

    let sparql = if is_person {
        format!(
            r#"SELECT ?item ?itemLabel ?birthYear ?birthPlaceLabel WHERE {{
  ?item rdfs:label "{}"@en .
  ?item wdt:P31 wd:Q5 .
  OPTIONAL {{ ?item wdt:P569 ?birthDate . BIND(YEAR(?birthDate) AS ?birthYear) }}
  OPTIONAL {{ ?item wdt:P19 ?birthPlace . }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en" . }}
}} LIMIT 1"#,
            entity.replace('"', r#"\""#)
        )
    } else {
        format!(
            r#"SELECT ?item ?itemLabel WHERE {{
  ?item rdfs:label "{}"@en .
  {{ ?item wdt:P31/wdt:P279* wd:Q515 . }}
  UNION
  {{ ?item wdt:P31/wdt:P279* wd:Q6256 . }}
  UNION
  {{ ?item wdt:P31/wdt:P279* wd:Q82794 . }}
  SERVICE wikibase:label {{ bd:serviceParam wikibase:language "en" . }}
}} LIMIT 1"#,
            entity.replace('"', r#"\""#)
        )
    };

    let url = format!(
        "https://query.wikidata.org/sparql?query={}",
        urlencoded(&sparql)
    );

    let response = ureq::get(&url)
        .set("Accept", "application/sparql-results+json")
        .set(
            "User-Agent",
            "TruthLens/0.4 (https://github.com/yablokolabs/truthlens)",
        )
        .call()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let body: serde_json::Value = response
        .into_json()
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let bindings = body["results"]["bindings"]
        .as_array()
        .ok_or("No bindings in response")?;

    if bindings.is_empty() {
        return Ok(None);
    }

    let first = &bindings[0];
    let qid: Option<String> = first["item"]["value"]
        .as_str()
        .and_then(|u: &str| u.rsplit('/').next())
        .map(|s: &str| s.to_string());

    let mut properties = Vec::new();

    if let Some(year) = first["birthYear"]["value"].as_str() {
        properties.push(format!("birth year: {}", year));
    }
    if let Some(place) = first["birthPlaceLabel"]["value"].as_str()
        && place != entity
    {
        properties.push(format!("birthplace: {}", place));
    }

    let confidence = if !properties.is_empty() { 0.8 } else { 0.6 };

    Ok(Some(EntityMatch {
        entity_name: entity.to_string(),
        wikidata_id: qid,
        verified_properties: properties,
        confidence,
    }))
}

#[cfg(feature = "verify")]
fn urlencoded(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Verify a claim's entities and return a VerificationResult.
///
/// Requires the "verify" feature. Without it, always returns Unknown.
#[cfg(feature = "verify")]
pub fn verify_claim(claim_text: &str) -> VerificationResult {
    let entities = extract_entities_from_claim(claim_text);
    if entities.is_empty() {
        return VerificationResult {
            claim_text: claim_text.to_string(),
            matches: vec![],
            status: VerificationStatus::Unknown,
        };
    }

    let matches = verify_entities(&entities);

    let has_verified = matches
        .iter()
        .any(|m| m.wikidata_id.is_some() && m.confidence > 0.5);
    let status = if has_verified {
        VerificationStatus::Verified
    } else {
        VerificationStatus::Unknown
    };

    VerificationResult {
        claim_text: claim_text.to_string(),
        matches,
        status,
    }
}

#[cfg(not(feature = "verify"))]
pub fn verify_claim(claim_text: &str) -> VerificationResult {
    VerificationResult {
        claim_text: claim_text.to_string(),
        matches: vec![],
        status: VerificationStatus::Unknown,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_person_name() {
        let entities =
            extract_entities_from_claim("Albert Einstein was born in 1879 in Ulm, Germany.");
        assert!(
            entities.contains(&"Albert Einstein".to_string()),
            "Should extract 'Albert Einstein', got: {:?}",
            entities
        );
        assert!(
            entities.contains(&"1879".to_string()),
            "Should extract year '1879', got: {:?}",
            entities
        );
    }

    #[test]
    fn extract_multiple_names() {
        let entities =
            extract_entities_from_claim("Marie Curie and Pierre Curie discovered radium in Paris.");
        assert!(
            entities.contains(&"Marie Curie".to_string()),
            "Should extract 'Marie Curie', got: {:?}",
            entities
        );
        assert!(
            entities.contains(&"Pierre Curie".to_string()),
            "Should extract 'Pierre Curie', got: {:?}",
            entities
        );
    }

    #[test]
    fn extract_years() {
        let entities =
            extract_entities_from_claim("The theory was published in 1905 and revised in 1915.");
        assert!(entities.contains(&"1905".to_string()));
        assert!(entities.contains(&"1915".to_string()));
    }

    #[test]
    fn extract_no_entities_from_vague_text() {
        let entities =
            extract_entities_from_claim("Some researchers have found various interesting results.");
        // Should have no multi-word names or years
        let has_year = entities
            .iter()
            .any(|e| e.len() == 4 && e.chars().all(|c| c.is_ascii_digit()));
        assert!(
            !has_year,
            "Should not extract years from vague text, got: {:?}",
            entities
        );
    }

    #[test]
    fn verification_modifier_all_verified() {
        let results = vec![
            VerificationResult {
                claim_text: "test".to_string(),
                matches: vec![],
                status: VerificationStatus::Verified,
            },
            VerificationResult {
                claim_text: "test2".to_string(),
                matches: vec![],
                status: VerificationStatus::Verified,
            },
        ];
        let modifier = compute_verification_modifier(&results);
        assert!(
            (modifier - 0.15).abs() < 0.001,
            "All verified should give +0.15, got {}",
            modifier
        );
    }

    #[test]
    fn verification_modifier_all_contradicted() {
        let results = vec![VerificationResult {
            claim_text: "test".to_string(),
            matches: vec![],
            status: VerificationStatus::Contradicted,
        }];
        let modifier = compute_verification_modifier(&results);
        assert!(
            (modifier - (-0.15)).abs() < 0.001,
            "All contradicted should give -0.15, got {}",
            modifier
        );
    }

    #[test]
    fn verification_modifier_mixed() {
        let results = vec![
            VerificationResult {
                claim_text: "test".to_string(),
                matches: vec![],
                status: VerificationStatus::Verified,
            },
            VerificationResult {
                claim_text: "test2".to_string(),
                matches: vec![],
                status: VerificationStatus::Contradicted,
            },
        ];
        let modifier = compute_verification_modifier(&results);
        assert!(
            modifier.abs() < 0.001,
            "Mixed should be ~0.0, got {}",
            modifier
        );
    }

    #[test]
    fn verification_modifier_empty() {
        let modifier = compute_verification_modifier(&[]);
        assert!(
            modifier.abs() < 0.001,
            "Empty should give 0.0, got {}",
            modifier
        );
    }

    #[test]
    fn verification_modifier_all_unknown() {
        let results = vec![VerificationResult {
            claim_text: "test".to_string(),
            matches: vec![],
            status: VerificationStatus::Unknown,
        }];
        let modifier = compute_verification_modifier(&results);
        assert!(
            modifier.abs() < 0.001,
            "All unknown should give 0.0, got {}",
            modifier
        );
    }

    #[test]
    fn verification_modifier_bounded() {
        // Even extreme inputs should stay in [-0.15, 0.15]
        let results: Vec<VerificationResult> = (0..100)
            .map(|_| VerificationResult {
                claim_text: "test".to_string(),
                matches: vec![],
                status: VerificationStatus::Verified,
            })
            .collect();
        let modifier = compute_verification_modifier(&results);
        assert!((-0.15..=0.15).contains(&modifier));
    }

    #[test]
    fn entity_match_serialization() {
        let m = EntityMatch {
            entity_name: "Albert Einstein".to_string(),
            wikidata_id: Some("Q937".to_string()),
            verified_properties: vec!["birth year: 1879".to_string()],
            confidence: 0.8,
        };
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("Q937"));
        let deserialized: EntityMatch = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.entity_name, "Albert Einstein");
    }

    #[test]
    fn verification_status_display() {
        assert_eq!(format!("{}", VerificationStatus::Verified), "VERIFIED");
        assert_eq!(
            format!("{}", VerificationStatus::Contradicted),
            "CONTRADICTED"
        );
        assert_eq!(format!("{}", VerificationStatus::Unknown), "UNKNOWN");
    }

    // Integration test that hits Wikidata — only run with `cargo test --features verify -- --ignored`
    #[test]
    #[ignore]
    #[cfg(feature = "verify")]
    fn integration_verify_einstein() {
        let entities = vec!["Albert Einstein".to_string()];
        let matches = verify_entities(&entities);
        assert!(
            !matches.is_empty(),
            "Should find Albert Einstein on Wikidata"
        );
        let einstein = &matches[0];
        assert!(
            einstein.wikidata_id.is_some(),
            "Should have a QID for Einstein"
        );
        assert!(
            !einstein.verified_properties.is_empty(),
            "Should have verified properties for Einstein"
        );
        eprintln!("Einstein match: {:?}", einstein);
    }

    #[test]
    #[ignore]
    #[cfg(feature = "verify")]
    fn integration_verify_claim_text() {
        let result = verify_claim("Albert Einstein was born in 1879 in Ulm, Germany.");
        assert_eq!(result.status, VerificationStatus::Verified);
        assert!(!result.matches.is_empty());
        eprintln!("Verification result: {:?}", result);
    }
}
