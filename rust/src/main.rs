use truthlens::analyze;

fn print_report(title: &str, text: &str) {
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ {title:<55} │");
    println!("└─────────────────────────────────────────────────────────┘");
    println!();

    let report = analyze(text);

    // Overall score
    let bar_len = (report.score * 30.0) as usize;
    let bar = "█".repeat(bar_len) + &"░".repeat(30 - bar_len);
    println!(
        "  Trust: {:.0}% [{bar}] {}\n",
        report.score * 100.0,
        report.risk_level
    );
    println!("  {}", report.summary);
    println!();

    // Trajectory analysis (v0.2)
    println!(
        "  📈 Trajectory: {} (ζ≈{:.2}, modifier={:+.2})",
        report.trajectory.pattern, report.trajectory.damping_estimate, report.trajectory.trust_modifier
    );
    println!();

    // Per-claim breakdown
    for (i, claim) in report.claims.iter().enumerate() {
        let icon = match claim.trust.risk_level {
            truthlens::RiskLevel::Low => "✅",
            truthlens::RiskLevel::Medium => "⚠️ ",
            truthlens::RiskLevel::High => "🔴",
            truthlens::RiskLevel::Critical => "💀",
        };
        println!(
            "  {icon} Claim {}: {:.0}% — \"{}\"",
            i + 1,
            claim.trust.score * 100.0,
            truncate(&claim.text, 70)
        );
        println!("     {}", claim.trust.explanation);
    }
    println!();
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn main() {
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  TruthLens — AI Hallucination Detector                  ║");
    println!("║  Formally Verified Trust Scoring for LLM Outputs        ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Example 1: Factual text (should score high)
    print_report(
        "Example 1: Factual Wikipedia-style text",
        "Albert Einstein was born on March 14, 1879, in Ulm, Germany. \
         He developed the theory of special relativity in 1905. \
         Einstein received the Nobel Prize in Physics in 1921 for his \
         explanation of the photoelectric effect.",
    );

    // Example 2: Hedged/uncertain text (should score medium-high)
    print_report(
        "Example 2: Well-hedged uncertain text",
        "Climate change might be linked to increased hurricane frequency. \
         Some researchers believe that ocean temperatures could affect \
         storm intensity. It is possible that sea levels will rise by \
         several meters over the next century.",
    );

    // Example 3: Overconfident hallucination-style text
    print_report(
        "Example 3: Overconfident suspicious claims",
        "The Great Wall of China is exactly 21,196.18 kilometers long. \
         It was built by precisely 3,247,862 workers over 47 years. \
         The wall can be clearly seen from the International Space Station \
         with the naked eye at all times of day.",
    );

    // Example 4: Mixed factual + hallucinated
    print_report(
        "Example 4: Mixed factual and suspicious",
        "Python was created by Guido van Rossum and first released in 1991. \
         It is the most popular programming language with exactly 47.3 million \
         developers worldwide. Python 4.0 was released in December 2025 with \
         native quantum computing support built into the standard library.",
    );

    // Example 5: Vague filler text
    print_report(
        "Example 5: Vague non-specific text",
        "There are various factors that contribute to the situation. \
         Some experts have noted several interesting developments in the field. \
         Many people believe that things will generally improve over time.",
    );

    // JSON output demo
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│ JSON Output (for API integration)                       │");
    println!("└─────────────────────────────────────────────────────────┘\n");

    let report = analyze("Einstein invented the telephone in 1876.");
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
