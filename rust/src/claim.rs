use serde::{Deserialize, Serialize};

/// An atomic claim extracted from text.
///
/// Text is decomposed into individual factual claims that can be
/// independently verified. Each claim carries metadata about its
/// verifiability and specificity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The original text of the claim
    pub text: String,
    /// Sentence index in the original text
    pub sentence_idx: usize,
    /// Whether the claim contains verifiable facts (names, dates, numbers)
    pub is_verifiable: bool,
    /// Specificity score: how specific/concrete the claim is (0.0–1.0)
    pub specificity: f64,
    /// Whether the claim contains hedging language
    pub is_hedged: bool,
}

/// Extract atomic claims from a text passage.
///
/// Splits text into sentences, then analyzes each for factual content.
/// A claim is a sentence that makes an assertion about the world.
pub fn extract_claims(text: &str) -> Vec<Claim> {
    let sentences = split_sentences(text);
    sentences
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.trim().is_empty())
        .map(|(idx, sentence)| {
            let text = sentence.trim().to_string();
            let is_verifiable = contains_verifiable_content(&text);
            let specificity = compute_specificity(&text);
            let is_hedged = contains_hedging(&text);

            Claim {
                text,
                sentence_idx: idx,
                is_verifiable,
                specificity,
                is_hedged,
            }
        })
        .collect()
}

/// Split text into sentences using basic heuristics.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if (ch == '.' || ch == '!' || ch == '?') && current.len() > 10 {
            // Avoid splitting on abbreviations like "Dr.", "U.S.", etc.
            let trimmed = current.trim();
            if !trimmed.ends_with("Dr.")
                && !trimmed.ends_with("Mr.")
                && !trimmed.ends_with("Mrs.")
                && !trimmed.ends_with("vs.")
                && !trimmed.ends_with("etc.")
                && !trimmed.ends_with("e.g.")
                && !trimmed.ends_with("i.e.")
            {
                sentences.push(current.trim().to_string());
                current = String::new();
            }
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }
    sentences
}

/// Check if text contains verifiable content (entities, numbers, dates).
fn contains_verifiable_content(text: &str) -> bool {
    // Contains numbers
    let has_numbers = text.chars().any(|c| c.is_ascii_digit());

    // Contains capitalized words (potential proper nouns) — skip first word
    let words: Vec<&str> = text.split_whitespace().collect();
    let has_proper_nouns = words.iter().skip(1).any(|w| {
        w.len() > 1
            && w.chars().next().map_or(false, |c| c.is_uppercase())
            && !is_common_sentence_starter(w)
    });

    // Contains date patterns
    let has_dates = regex::Regex::new(r"\b\d{4}\b|\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\b")
        .map_or(false, |re| re.is_match(text));

    // Contains measurement/quantity words
    let has_quantities = text.contains('%')
        || text.contains("million")
        || text.contains("billion")
        || text.contains("thousand")
        || text.contains("kilometers")
        || text.contains("miles")
        || text.contains("years");

    has_numbers || has_proper_nouns || has_dates || has_quantities
}

fn is_common_sentence_starter(word: &str) -> bool {
    matches!(
        word,
        "The" | "This" | "That" | "These" | "Those" | "It" | "They" | "He" | "She"
        | "We" | "However" | "Although" | "Moreover" | "Furthermore" | "Additionally"
        | "In" | "On" | "At" | "For" | "With" | "But" | "And" | "Or" | "As" | "If"
    )
}

/// Compute specificity score (0.0 = vague, 1.0 = highly specific).
fn compute_specificity(text: &str) -> f64 {
    let mut score = 0.0;
    let mut signals = 0;

    // Numbers add specificity
    let digit_count = text.chars().filter(|c| c.is_ascii_digit()).count();
    if digit_count > 0 {
        score += 0.3;
        signals += 1;
    }

    // Proper nouns add specificity
    let words: Vec<&str> = text.split_whitespace().collect();
    let proper_noun_count = words
        .iter()
        .skip(1)
        .filter(|w| {
            w.len() > 1
                && w.chars().next().map_or(false, |c| c.is_uppercase())
                && !is_common_sentence_starter(w)
        })
        .count();
    if proper_noun_count > 0 {
        score += 0.2 * (proper_noun_count as f64).min(3.0) / 3.0;
        signals += 1;
    }

    // Vague quantifiers reduce specificity
    let vague_words = [
        "some", "many", "several", "few", "various", "numerous", "often",
        "sometimes", "generally", "typically", "usually", "about", "approximately",
        "roughly", "around", "nearly",
    ];
    let vague_count = words
        .iter()
        .filter(|w| vague_words.contains(&w.to_lowercase().as_str()))
        .count();
    if vague_count > 0 {
        score -= 0.15 * vague_count as f64;
        signals += 1;
    }

    // Definitive language adds specificity
    let definitive_words = [
        "exactly", "precisely", "specifically", "always", "never", "every",
    ];
    let definitive_count = words
        .iter()
        .filter(|w| definitive_words.contains(&w.to_lowercase().as_str()))
        .count();
    if definitive_count > 0 {
        score += 0.15;
        signals += 1;
    }

    // Sentence length: very short or very long sentences tend to be less specific
    let word_count = words.len();
    if (10..30).contains(&word_count) {
        score += 0.1;
    }

    if signals == 0 {
        return 0.3; // Default: moderate specificity
    }

    score.clamp(0.0, 1.0)
}

/// Check if text contains hedging language.
///
/// Hedging indicates uncertainty and is often (but not always) a sign
/// that the model is less confident — which correlates with lower
/// hallucination risk for that specific claim.
pub fn contains_hedging(text: &str) -> bool {
    let lower = text.to_lowercase();
    let hedge_phrases = [
        "might be", "could be", "may be", "possibly", "perhaps", "likely",
        "probably", "it seems", "it appears", "reportedly", "allegedly",
        "it is believed", "it is thought", "some suggest", "arguably",
        "it is possible", "there is evidence", "to some extent",
        "not entirely clear", "remains uncertain", "is debated",
        "as far as we know", "to the best of", "i think", "i believe",
    ];
    hedge_phrases.iter().any(|h| lower.contains(h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_basic_claims() {
        let text = "Albert Einstein was born in 1879 in Ulm, Germany. He developed the theory of relativity. The sky is blue.";
        let claims = extract_claims(text);
        assert_eq!(claims.len(), 3);
        assert!(claims[0].is_verifiable); // Has date + proper noun
        assert!(claims[0].specificity > 0.3);
    }

    #[test]
    fn detect_hedging() {
        assert!(contains_hedging("This might be related to climate change."));
        assert!(contains_hedging("It is believed that the universe is expanding."));
        assert!(!contains_hedging("The Earth orbits the Sun."));
    }

    #[test]
    fn specificity_scoring() {
        let specific = compute_specificity("Einstein published 300 papers in 1905 at the University of Zurich.");
        let vague = compute_specificity("Some researchers have found various interesting results.");
        assert!(specific > vague, "specific={specific} should be > vague={vague}");
    }

    #[test]
    fn empty_text() {
        let claims = extract_claims("");
        assert!(claims.is_empty());
    }
}
