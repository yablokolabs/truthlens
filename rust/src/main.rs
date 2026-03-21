use std::io::Read;
use truthlens::analyze;

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

fn print_usage() {
    eprintln!("TruthLens 🔍 — AI Hallucination Detector\n");
    eprintln!("Usage:");
    eprintln!("  truthlens \"text to analyze\"           Analyze text for hallucination risk");
    eprintln!("  truthlens --json \"text to analyze\"     Output as JSON");
    eprintln!("  echo \"text\" | truthlens                Read from stdin");
    eprintln!("  echo \"text\" | truthlens --json         Read from stdin, output JSON");
    eprintln!("  truthlens --demo                       Run demo examples");
    eprintln!("  truthlens --help                       Show this help");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  truthlens \"Einstein was born in 1879 in Ulm, Germany.\"");
    eprintln!("  truthlens --json \"Python 4.0 was released with quantum support.\"");
    eprintln!("  cat ai_response.txt | truthlens");
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
        (
            "Mixed factual + hallucinated",
            "Python was created by Guido van Rossum and first released in 1991. \
             Python 4.0 was released in December 2025 with native quantum computing support.",
        ),
    ];

    for (title, text) in examples {
        println!("\n─── {title} ───");
        print_report(text, false);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        // Check if stdin has data (piped input)
        if atty::is(atty::Stream::Stdin) {
            print_usage();
            std::process::exit(1);
        }

        // Read from stdin
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
    let mut text_args: Vec<&str> = Vec::new();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--json" | "-j" => json_mode = true,
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--demo" | "-d" => {
                run_demo();
                return;
            }
            _ => text_args.push(arg),
        }
    }

    if text_args.is_empty() {
        // JSON mode but no text — read from stdin
        if !atty::is(atty::Stream::Stdin) {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            if !input.trim().is_empty() {
                print_report(input.trim(), json_mode);
                return;
            }
        }
        print_usage();
        std::process::exit(1);
    }

    let text = text_args.join(" ");
    print_report(&text, json_mode);
}
