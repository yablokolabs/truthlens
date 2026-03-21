use std::io::Read;
use truthlens::{analyze, check_consistency};

fn print_report(text: &str, json_mode: bool) {
    let report = analyze(text);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }

    // Overall score
    let bar_len = (report.score * 30.0) as usize;
    let bar = "█".repeat(bar_len) + &"░".repeat(30 - bar_len);
    println!(
        "\n  Trust: {:.0}% [{bar}] {}\n",
        report.score * 100.0,
        report.risk_level
    );
    println!("  {}", report.summary);

    // Trajectory
    println!(
        "\n  📈 Trajectory: {} (ζ≈{:.2}, modifier={:+.2})\n",
        report.trajectory.pattern,
        report.trajectory.damping_estimate,
        report.trajectory.trust_modifier
    );

    // Per-claim breakdown
    for (i, claim) in report.claims.iter().enumerate() {
        let icon = match claim.trust.risk_level {
            truthlens::RiskLevel::Low => "✅",
            truthlens::RiskLevel::Medium => "⚠️ ",
            truthlens::RiskLevel::High => "🔴",
            truthlens::RiskLevel::Critical => "💀",
        };
        let text = if claim.text.len() > 70 {
            format!("{}...", &claim.text[..67])
        } else {
            claim.text.clone()
        };
        println!(
            "  {icon} Claim {}: {:.0}% — \"{text}\"",
            i + 1,
            claim.trust.score * 100.0,
        );
        println!("     {}", claim.trust.explanation);
    }
    println!();
}

fn print_consistency(responses: &[&str], json_mode: bool) {
    let report = check_consistency(responses);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }

    let bar_len = (report.consistency_score * 30.0) as usize;
    let bar = "█".repeat(bar_len) + &"░".repeat(30 - bar_len);
    println!(
        "\n  Consistency: {:.0}% [{bar}]",
        report.consistency_score * 100.0
    );
    println!(
        "  {} responses, {} total claims\n",
        report.n_responses, report.total_claims
    );

    if !report.contradictions.is_empty() {
        println!("  ❌ Contradictions:");
        for c in &report.contradictions {
            println!(
                "     Response {} vs {}: {}",
                c.response_a + 1,
                c.response_b + 1,
                c.conflict
            );
        }
        println!();
    }

    if !report.consistent_claims.is_empty() {
        println!("  ✅ Consistent claims:");
        for c in &report.consistent_claims {
            println!(
                "     {}/{} agree: {}",
                c.agreement_count, report.n_responses, c.text
            );
        }
        println!();
    }

    if !report.unique_claims.is_empty() {
        println!("  🔍 Unique to one response (verify these):");
        for u in &report.unique_claims {
            println!("     Response {}: {}", u.response_idx + 1, u.text);
        }
        println!();
    }
}

fn print_usage() {
    eprintln!("TruthLens 🔍 — AI Hallucination Detector\n");
    eprintln!("Usage:");
    eprintln!("  truthlens \"text to analyze\"                    Analyze text");
    eprintln!("  truthlens --json \"text\"                        JSON output");
    eprintln!("  echo \"text\" | truthlens                        Read from stdin");
    eprintln!();
    eprintln!("  truthlens --consistency \"resp1\" \"resp2\" ...     Compare multiple responses");
    eprintln!("  truthlens --consistency --json \"r1\" \"r2\"       Consistency check as JSON");
    eprintln!();
    eprintln!("  truthlens --demo                               Run demo examples");
    eprintln!("  truthlens --help                               Show this help");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  truthlens \"Einstein was born in 1879 in Ulm.\"");
    eprintln!("  truthlens --consistency \\");
    eprintln!("    \"Einstein was born in 1879 in Ulm.\" \\");
    eprintln!("    \"Einstein was born in 1879 in Munich.\"");
}

fn run_demo() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  TruthLens 🔍 — AI Hallucination Detector               ║");
    println!("╚══════════════════════════════════════════════════════════╝");

    let examples = vec![
        (
            "Factual text",
            "Albert Einstein was born on March 14, 1879, in Ulm, Germany. \
             He developed the theory of special relativity in 1905. \
             Einstein received the Nobel Prize in Physics in 1921.",
        ),
        (
            "Well-hedged text",
            "Climate change might be linked to increased hurricane frequency. \
             Some researchers believe that ocean temperatures could affect storm intensity. \
             It is possible that sea levels will rise by several meters.",
        ),
        (
            "Overconfident suspicious claims",
            "The Great Wall of China is exactly 21,196.18 kilometers long. \
             It was built by precisely 3,247,862 workers over 47 years. \
             The wall can be clearly seen from the International Space Station.",
        ),
    ];

    for (title, text) in examples {
        println!("\n─── {title} ───");
        print_report(text, false);
    }

    // Consistency demo
    println!("\n─── Consistency check ───");
    print_consistency(
        &[
            "Einstein was born in 1879 in Ulm, Germany. He had 3 children.",
            "Einstein was born in 1879 in Munich, Germany. He had 3 children.",
            "Einstein was born in 1879 in Ulm, Germany. He had 5 children.",
        ],
        false,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        if atty::is(atty::Stream::Stdin) {
            print_usage();
            std::process::exit(1);
        }
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).unwrap();
        if input.trim().is_empty() {
            eprintln!("Error: empty input");
            std::process::exit(1);
        }
        print_report(input.trim(), false);
        return;
    }

    let mut json_mode = false;
    let mut consistency_mode = false;
    let mut text_args: Vec<String> = Vec::new();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--json" | "-j" => json_mode = true,
            "--consistency" | "-c" => consistency_mode = true,
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--demo" | "-d" => {
                run_demo();
                return;
            }
            _ => text_args.push(arg.clone()),
        }
    }

    if text_args.is_empty() {
        if !atty::is(atty::Stream::Stdin) {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                if consistency_mode {
                    // Try parsing as JSON array
                    if let Ok(responses) = serde_json::from_str::<Vec<String>>(trimmed) {
                        let refs: Vec<&str> = responses.iter().map(|s| s.as_str()).collect();
                        print_consistency(&refs, json_mode);
                    } else {
                        eprintln!(
                            "Error: --consistency with stdin expects a JSON array of strings"
                        );
                        std::process::exit(1);
                    }
                } else {
                    print_report(trimmed, json_mode);
                }
                return;
            }
        }
        print_usage();
        std::process::exit(1);
    }

    if consistency_mode {
        let refs: Vec<&str> = text_args.iter().map(|s| s.as_str()).collect();
        print_consistency(&refs, json_mode);
    } else {
        let text = text_args.join(" ");
        print_report(&text, json_mode);
    }
}
