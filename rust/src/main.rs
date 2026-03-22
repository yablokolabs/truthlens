use colored::Colorize;
use std::io::Read;
use truthlens::{analyze, analyze_with_verification, check_consistency};

fn print_report(text: &str, json_mode: bool, verify: bool) {
    let report = if verify {
        analyze_with_verification(text)
    } else {
        analyze(text)
    };

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }

    // Overall score
    let bar_len = (report.score * 30.0) as usize;
    let bar_filled = "█".repeat(bar_len);
    let bar_empty = "░".repeat(30 - bar_len);

    let score_pct = format!("{:.0}%", report.score * 100.0);
    let risk_str = format!("{}", report.risk_level);

    let (score_colored, risk_colored, bar_colored) = match report.risk_level {
        truthlens::RiskLevel::Low => (
            score_pct.green().bold(),
            risk_str.green().bold(),
            bar_filled.green(),
        ),
        truthlens::RiskLevel::Medium => (
            score_pct.yellow().bold(),
            risk_str.yellow().bold(),
            bar_filled.yellow(),
        ),
        truthlens::RiskLevel::High => (
            score_pct.red().bold(),
            risk_str.red().bold(),
            bar_filled.red(),
        ),
        truthlens::RiskLevel::Critical => (
            score_pct.red().bold(),
            risk_str.red().bold(),
            bar_filled.red(),
        ),
    };

    println!(
        "\n  Trust: {} [{}{}] {}\n",
        score_colored,
        bar_colored,
        bar_empty.dimmed(),
        risk_colored
    );
    println!("  {}", report.summary.dimmed());

    // Trajectory
    let traj_str = format!("{}", report.trajectory.pattern);
    let traj_colored = match report.trajectory.trust_modifier {
        m if m > 0.0 => traj_str.green(),
        m if m < 0.0 => traj_str.red(),
        _ => traj_str.white(),
    };
    println!(
        "\n  📈 Trajectory: {} (ζ≈{:.2}, modifier={:+.2})\n",
        traj_colored, report.trajectory.damping_estimate, report.trajectory.trust_modifier
    );

    // Per-claim breakdown
    for (i, claim) in report.claims.iter().enumerate() {
        let (icon, claim_score) = match claim.trust.risk_level {
            truthlens::RiskLevel::Low => {
                ("✅", format!("{:.0}%", claim.trust.score * 100.0).green())
            }
            truthlens::RiskLevel::Medium => {
                ("⚠️ ", format!("{:.0}%", claim.trust.score * 100.0).yellow())
            }
            truthlens::RiskLevel::High => {
                ("🔴", format!("{:.0}%", claim.trust.score * 100.0).red())
            }
            truthlens::RiskLevel::Critical => (
                "💀",
                format!("{:.0}%", claim.trust.score * 100.0).red().bold(),
            ),
        };
        let text = if claim.text.len() > 70 {
            format!("{}...", &claim.text[..67])
        } else {
            claim.text.clone()
        };
        println!(
            "  {icon} Claim {}: {} — \"{}\"",
            i + 1,
            claim_score,
            text.white()
        );
        println!("     {}", claim.trust.explanation.dimmed());

        // Display verification results if present
        if let Some(ref vr) = claim.verification {
            for m in &vr.matches {
                if let Some(ref qid) = m.wikidata_id {
                    let props = if m.verified_properties.is_empty() {
                        "exists".to_string()
                    } else {
                        m.verified_properties.join(", ")
                    };
                    println!(
                        "     {} Verified: {} ({}) — {} \u{2713}",
                        "\u{1f50d}".green(),
                        m.entity_name.green(),
                        qid.dimmed(),
                        props.green()
                    );
                } else {
                    println!(
                        "     {} Not verified: {} — no match found",
                        "\u{274c}".red(),
                        m.entity_name.red()
                    );
                }
            }
        }
    }
    println!();
}

fn print_consistency(responses: &[&str], json_mode: bool) {
    let report = check_consistency(responses);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return;
    }

    let score_pct = format!("{:.0}%", report.consistency_score * 100.0);
    let bar_len = (report.consistency_score * 30.0) as usize;
    let bar_filled = "█".repeat(bar_len);
    let bar_empty = "░".repeat(30 - bar_len);

    let (score_colored, bar_colored) = if report.consistency_score >= 0.7 {
        (score_pct.green().bold(), bar_filled.green())
    } else if report.consistency_score >= 0.5 {
        (score_pct.yellow().bold(), bar_filled.yellow())
    } else {
        (score_pct.red().bold(), bar_filled.red())
    };

    println!(
        "\n  Consistency: {} [{}{}]",
        score_colored,
        bar_colored,
        bar_empty.dimmed()
    );
    println!(
        "  {} responses, {} total claims\n",
        report.n_responses.to_string().bold(),
        report.total_claims.to_string().bold()
    );

    if !report.contradictions.is_empty() {
        println!("  {} Contradictions:", "❌".red());
        for c in &report.contradictions {
            println!(
                "     Response {} vs {}: {}",
                (c.response_a + 1).to_string().bold(),
                (c.response_b + 1).to_string().bold(),
                c.conflict.red()
            );
        }
        println!();
    }

    if !report.consistent_claims.is_empty() {
        println!("  {} Consistent claims:", "✅".green());
        for c in &report.consistent_claims {
            println!(
                "     {}/{} agree: {}",
                c.agreement_count.to_string().green(),
                report.n_responses,
                c.text.white()
            );
        }
        println!();
    }

    if !report.unique_claims.is_empty() {
        println!("  {} Unique to one response (verify these):", "🔍".yellow());
        for u in &report.unique_claims {
            println!(
                "     Response {}: {}",
                (u.response_idx + 1).to_string().yellow(),
                u.text.yellow()
            );
        }
        println!();
    }
}

fn print_usage() {
    eprintln!("{}", "TruthLens 🔍 — AI Hallucination Detector\n".bold());
    eprintln!("{}", "Usage:".underline());
    eprintln!(
        "  {} \"text to analyze\"                    Analyze text",
        "truthlens".green()
    );
    eprintln!(
        "  {} --json \"text\"                        JSON output",
        "truthlens".green()
    );
    eprintln!(
        "  echo \"text\" | {}                        Read from stdin",
        "truthlens".green()
    );
    eprintln!();
    eprintln!(
        "  {} --verify \"text\"                      Verify entities via Wikidata",
        "truthlens".green()
    );
    eprintln!();
    eprintln!(
        "  {} --consistency \"resp1\" \"resp2\" ...     Compare responses",
        "truthlens".green()
    );
    eprintln!(
        "  {} --consistency --json \"r1\" \"r2\"       Consistency as JSON",
        "truthlens".green()
    );
    eprintln!();
    eprintln!(
        "  {} --demo                               Run examples",
        "truthlens".green()
    );
    eprintln!(
        "  {} --help                               Show this help",
        "truthlens".green()
    );
    eprintln!();
    eprintln!("{}", "Examples:".underline());
    eprintln!("  truthlens \"Einstein was born in 1879 in Ulm.\"",);
    eprintln!("  truthlens --verify \"Einstein was born in 1879 in Ulm.\"",);
    eprintln!("  truthlens --consistency \\",);
    eprintln!("    \"Einstein was born in 1879 in Ulm.\" \\",);
    eprintln!("    \"Einstein was born in 1879 in Munich.\"",);
}

fn run_demo() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗".bold()
    );
    println!(
        "{}",
        "║  TruthLens 🔍 — AI Hallucination Detector               ║".bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝".bold()
    );

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
        println!("\n─── {} ───", title.bold());
        print_report(text, false, false);
    }

    println!("\n─── {} ───", "Consistency check".bold());
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
            eprintln!("{}", "Error: empty input".red());
            std::process::exit(1);
        }
        print_report(input.trim(), false, false);
        return;
    }

    let mut json_mode = false;
    let mut consistency_mode = false;
    let mut verify_mode = false;
    let mut text_args: Vec<String> = Vec::new();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--json" | "-j" => json_mode = true,
            "--consistency" | "-c" => consistency_mode = true,
            "--verify" | "-v" => verify_mode = true,
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

    // Check if verify was requested but feature not compiled
    if verify_mode && !cfg!(feature = "verify") {
        eprintln!(
            "{}",
            "Error: --verify requires the 'verify' feature. Reinstall with:".red()
        );
        eprintln!("  {}", "cargo install truthlens --features verify".yellow());
        std::process::exit(1);
    }

    if text_args.is_empty() {
        if !atty::is(atty::Stream::Stdin) {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                if consistency_mode {
                    if let Ok(responses) = serde_json::from_str::<Vec<String>>(trimmed) {
                        let refs: Vec<&str> = responses.iter().map(|s| s.as_str()).collect();
                        print_consistency(&refs, json_mode);
                    } else {
                        eprintln!(
                            "{}",
                            "Error: --consistency with stdin expects a JSON array of strings".red()
                        );
                        std::process::exit(1);
                    }
                } else {
                    print_report(trimmed, json_mode, verify_mode);
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
        print_report(&text, json_mode, verify_mode);
    }
}
