# TruthLens 🔍

[![Crates.io](https://img.shields.io/crates/v/truthlens.svg)](https://crates.io/crates/truthlens)
[![Docs.rs](https://docs.rs/truthlens/badge.svg)](https://docs.rs/truthlens)

**AI Hallucination Detector — Formally Verified Trust Scoring for LLM Outputs**

Analyze AI-generated text for hallucination risk. No API keys needed. No LLM calls required. Fast, local, formally verified, and color-coded terminal output.

**Published package:** <https://crates.io/crates/truthlens>
**API docs:** <https://docs.rs/truthlens>

## MCP Integration (v0.6)

TruthLens includes a local stdio MCP server for editor/agent integrations.

Available tools:
- `analyze_text` — analyze raw text in-context
- `analyze_file` — analyze a local UTF-8 text file

Run locally:

```bash
cd rust
cargo run --bin mcp_server
```

This enables local claim checking for AI-generated text without API keys or cloud calls.

## Quick Start

### Install as CLI

```bash
cargo install truthlens
```

### Usage

```bash
# Analyze text directly
truthlens "Einstein invented the telephone in 1876."
#  Trust: 49% [██████████████░░░░░░░░░░░░░░░░] HIGH
#  🔴 Claim 1: 49% — specific verifiable claim — verify independently

# JSON output (for scripts/API integration)
truthlens --json "Python 4.0 has quantum computing support."

# Pipe from file or other commands
cat ai_response.txt | truthlens

# Pipe from clipboard (macOS)
pbpaste | truthlens

# Analyze ChatGPT/Claude output saved to file
curl -s "https://api.example.com/chat" | truthlens --json

# Compare multiple AI responses for contradictions
truthlens --consistency "response 1" "response 2" "response 3"

# Run built-in demo examples
truthlens --demo
```

### Use as a Rust library

```rust
use truthlens::analyze;

let report = analyze("Einstein was born in 1879 in Ulm, Germany.");
println!("Trust: {:.0}% — {}", report.score * 100.0, report.risk_level);
// Trust: 52% — HIGH

// Access per-claim breakdown
for claim in &report.claims {
    println!("  {} — {}", claim.text, claim.trust.risk_level);
}

// Access trajectory analysis
println!("Pattern: {}", report.trajectory.pattern);
println!("Damping: ζ≈{:.2}", report.trajectory.damping_estimate);

// JSON serialization
let json = serde_json::to_string_pretty(&report).unwrap();
```

### Multi-response consistency check (v0.3)

Paste N responses to the same prompt — TruthLens detects contradictions between them.

```rust
use truthlens::check_consistency;

let report = check_consistency(&[
    "Einstein was born in 1879 in Ulm, Germany.",
    "Einstein was born in 1879 in Munich, Germany.",  // ← contradiction
    "Einstein was born in 1879 in Ulm, Germany.",
]);

println!("Consistency: {:.0}%", report.consistency_score * 100.0);
// Consistency: 75%

// Contradictions detected
for c in &report.contradictions {
    println!("⚠️  {} vs {} — {}", c.claim_a, c.claim_b, c.conflict);
}
// ⚠️  "Ulm, Germany" vs "Munich, Germany"

// Claims unique to one response (potential hallucination)
for u in &report.unique_claims {
    println!("🔍 Unique to response {}: {}", u.response_idx, u.text);
}
```

```bash
# CLI: compare multiple responses as separate arguments
truthlens --consistency \
  "Einstein was born in 1879 in Ulm, Germany." \
  "Einstein was born in 1879 in Munich, Germany." \
  "Einstein was born in 1879 in Ulm, Germany."
#  Consistency: 70% [█████████████████████░░░░░░░░░]
#  ❌ Contradictions:
#     Response 1 vs 2: "Ulm, Germany" vs "Munich, Germany"
#  ✅ Consistent claims:
#     3/3 agree: einstein was born in: 1879

# JSON output
truthlens --consistency --json "resp1" "resp2" "resp3"

# Pipe JSON array from stdin
echo '["Python was created in 1991.", "Python was created in 1989."]' \
  | truthlens --consistency
```

### Use as a Python library (v0.6)

```bash
pip install truthlens
```

```python
from truthlens import analyze, check_consistency, extract_claims, extract_entities

# Analyze text for hallucination risk
report = analyze("Einstein was born in 1879 in Ulm, Germany.")
print(f"Trust: {report['score']:.0%} — {report['risk_level']}")

# Per-claim breakdown
for claim in report["claims"]:
    print(f"  {claim['text']} — {claim['trust']['risk_level']}")

# Multi-response consistency check
result = check_consistency([
    "Einstein was born in 1879 in Ulm.",
    "Einstein was born in 1879 in Munich.",
])
print(f"Consistency: {result['consistency_score']:.0%}")

# Extract atomic claims
claims = extract_claims("Python was created in 1991. It is widely used.")

# Extract named entities
entities = extract_entities("Marie Curie won the Nobel Prize in 1903.")
print(entities)  # ['1903', 'Marie Curie']
```

### Install via Snap (v0.6)

```bash
# Install from Snap Store (Ubuntu/Linux)
sudo snap install truthlens

# Analyze text
truthlens "Einstein invented the telephone in 1876."

# JSON output
truthlens --json "Python was created in 1991."

# Compare multiple AI responses
truthlens --consistency \
  "Einstein was born in Ulm." \
  "Einstein was born in Munich."

# Entity verification (requires network)
truthlens --verify "Marie Curie won the Nobel Prize in 1903."

# Run demo examples
truthlens --demo

# Show help
truthlens --help
```

### Entity verification (v0.4)

Cross-reference named entities (people, places, dates) against Wikidata to boost or reduce trust scores.

```bash
# Install with verification support
cargo install truthlens --features verify

# Verify entities in a claim
truthlens --verify "Albert Einstein was born in 1879 in Ulm, Germany."
#  Trust: 67% [████████████████████░░░░░░░░░░] MEDIUM
#  🔍 Verified: Albert Einstein (Q937) — birth year: 1879, birthplace: Ulm ✓

# Combine with JSON output
truthlens --verify --json "Marie Curie won the Nobel Prize in 1903."
```

> **Note:** The `--verify` flag requires the `verify` feature (adds the `ureq` HTTP dependency).
> Without `--features verify`, TruthLens works fully offline with no network dependencies.

```toml
# Cargo.toml
[dependencies]
truthlens = "0.6"

# With entity verification
# truthlens = { version = "0.5", features = ["verify"] }
```

## What It Does

TruthLens decomposes AI text into atomic claims and scores each for hallucination risk using linguistic signals — **no LLM calls, no API keys, no external dependencies**.

```
Input:  "Python 4.0 was released in December 2025 with native quantum computing support."

Output: 🔴 Trust: 49% [HIGH]
        → specific verifiable claim — verify independently
        → overconfident language without hedging
```

## How It Works

### 1. Claim Extraction
Text → atomic sentences → each is an independent claim to evaluate.

### 2. Signal Analysis (per claim)

| Signal | What It Measures | Weight |
|--------|-----------------|--------|
| **Confidence** | Overconfident language without hedging (hallucination red flag) | 35% |
| **Hedging** | Uncertainty markers ("might", "possibly") — correlates with lower hallucination | 25% |
| **Specificity** | How concrete/verifiable the claim is (numbers, names, dates) | 20% |
| **Verifiability** | Whether the claim contains fact-checkable entities | 15% |
| **Consistency** | Multi-sample agreement (optional, requires LLM) | 5% |

### 3. Trust Score
Signals are aggregated into a single trust score in **[0.0, 1.0]**:

| Score | Risk Level | Meaning |
|-------|-----------|---------|
| 0.75–1.0 | ✅ LOW | Likely factual or appropriately hedged |
| 0.55–0.74 | ⚠️ MEDIUM | Some uncertain claims, verify key facts |
| 0.35–0.54 | 🔴 HIGH | Multiple suspicious claims, verify everything |
| 0.0–0.34 | 💀 CRITICAL | Likely contains hallucinations |

### 4. Passage Scoring
Passage score = 70% average + 30% worst claim. One bad claim drags down the whole passage.

## Key Design Decisions

- **No LLM required** — linguistic analysis only. Fast (microseconds), private (local), free.
- **Hedging = good** — unlike most "confidence detectors", we score hedged claims HIGHER. A model that says "might" is better calibrated than one that states falsehoods with certainty.
- **Specificity is double-edged** — specific claims are more useful but also more damaging if wrong. We flag them for independent verification.
- **Formally verified** — Lean 4 proofs guarantee score bounds, monotonicity, and composition properties.

## What's Proven (Lean 4)

### Score Bounds
- `signal_nonneg` — all signals ≥ 0
- `weighted_contrib_bounded` — w·s ≤ w·max when s ≤ max
- `clamped_score_in_range` — final score ∈ [0, 100] after clamp
- `truthlens_weights_sum` — weights sum to 100%

### Monotonicity
- `signal_increase_improves_score` — improving a signal improves the score
- `total_score_improves` — better signal + same rest = better total
- `good_claim_improves_passage` — adding a good claim raises the average

### Composition
- `passage_score_bounded` — 70%·avg + 30%·min ≤ 100%·max
- `passage_at_least_worst` — passage score ≥ 30% of worst claim
- `score_order_independent` — claim order doesn't affect passage score
- `score_deterministic` — same inputs → same output (functional purity)

### Trajectory (v0.2)
- `adjusted_score_bounded` — score + modifier stays bounded after clamp
- `transitions_bounded` — direction changes ≤ n_claims − 2
- `damping_positive` — damping estimate is always positive (stable system)
- `penalty_still_nonneg` — score after penalty ≥ 0 after clamp

### Consistency (v0.3)
- `consistency_bounded` — consistency score ∈ [0, 100] after clamp
- `contradictions_bounded` — contradiction count ≤ comparison pairs
- `agreement_ratio_valid` — agreement ≤ total responses
- `agreeing_response_improves` — adding agreement increases count
- `contradiction_symmetric` — if A contradicts B, B contradicts A
- `unique_bounded` — unique claims ≤ total claims

### Verification (v0.4)
- `verification_modifier_bounded` — modifier ∈ [0, 15] (scaled) after clamp
- `combined_modifier_bounded` — combined modifier ∈ [-15, +15]
- `adjusted_score_with_verification` — score + verification modifier stays in [0, 100]
- `adjusted_score_with_both` — score + trajectory + verification modifier stays in [0, 100]
- `entity_partition` — verified + contradicted + unknown = total
- `verified_contradicted_disjoint` — verified + contradicted ≤ total
- `empty_verification_neutral` — no entities → zero modifier
- `all_verified_max` — all verified → maximum positive modifier
- `all_contradicted_max` — all contradicted → maximum negative modifier
- `more_verified_improves` — adding verified entity increases modifier (monotonic)
- `more_contradicted_worsens` — adding contradicted entity decreases modifier (monotonic)

## Examples

### Factual text
```
"Albert Einstein was born on March 14, 1879, in Ulm, Germany."
→ 🔴 52% HIGH — specific verifiable claim, verify independently
```

### Well-hedged passage (✅ LOW risk)
```
"Climate change might be linked to increased hurricane frequency.
 Some researchers believe ocean temperatures could affect storm intensity.
 It is possible that sea levels will rise over the next century."
→ ✅ 60% LOW — Trajectory: FLAT LOW (consistently cautious), trust bonus +10%
```

### Single hedged claim
```
"Climate change might be linked to increased hurricane frequency."
→ ⚠️ 65% MEDIUM — appropriately hedged
```

### Overconfident hallucination
```
"The Great Wall is exactly 21,196.18 kilometers long."
→ 🔴 52% HIGH — overconfident without hedging; highly specific
```

### Vague filler
```
"Various factors contribute to the situation."
→ 🔴 40% HIGH — vague claim with low specificity
```

## JSON Output

```json
{
  "score": 0.49,
  "risk_level": "High",
  "summary": "1 claims analyzed. 1 high-risk claims detected.",
  "claims": [
    {
      "text": "Einstein invented the telephone in 1876.",
      "trust": {
        "score": 0.49,
        "signals": {
          "confidence": 0.5,
          "specificity": 0.3,
          "hedging": 0.5,
          "verifiability": 0.7,
          "consistency": null
        },
        "risk_level": "High"
      }
    }
  ]
}
```

## Repository Structure

```
truthlens/
├── .github/                    # Automation and release workflows
│   └── workflows/
│       ├── pypi-publish.yml
│       ├── python-ci.yml
│       ├── release.yml
│       └── rust-ci.yml
├── rust/                       # Core library + CLI
│   ├── src/
│   │   ├── lib.rs              # Public API: analyze(), check_consistency(), extract_*()
│   │   ├── claim.rs            # Claim extraction + linguistic analysis
│   │   ├── scorer.rs           # Trust scoring + signal aggregation
│   │   ├── trajectory.rs       # Confidence trajectory analysis (v0.2)
│   │   ├── consistency.rs      # Multi-response consistency checker (v0.3)
│   │   ├── entity.rs           # Entity cross-reference with Wikidata (v0.4)
│   │   ├── mcp.rs              # MCP request handling, tool dispatch, and tests (v0.6)
│   │   └── main.rs             # CLI: analyze, --consistency, --verify, --demo
│   ├── bin/
│   │   └── mcp_server.rs       # Local stdio MCP server (v0.6)
│   ├── tests/
│   │   └── integration.rs      # End-to-end integration tests
│   └── Cargo.toml
├── python/                     # Python bindings (v0.6)
│   ├── src/lib.rs              # PyO3 wrapper
│   ├── truthlens/              # Python package
│   │   ├── __init__.py         # Re-exports + docstrings
│   │   ├── __init__.pyi        # Type stubs (PEP 561)
│   │   └── py.typed            # PEP 561 marker
│   ├── tests/
│   │   └── test_truthlens.py   # Python test suite
│   ├── Cargo.toml              # cdylib crate
│   └── pyproject.toml          # maturin build config
├── lean/                       # Formal proofs
│   ├── TruthLens/
│   │   ├── ScoreBounds.lean    # Score ∈ [0, 1], weight sum, clamp
│   │   ├── Monotonicity.lean   # Better signals → better score
│   │   ├── Composition.lean    # Passage aggregation properties
│   │   ├── Trajectory.lean     # Trajectory modifier bounds + correctness
│   │   ├── Consistency.lean    # Contradiction bounds, agreement, symmetry
│   │   └── Verification.lean   # Entity verification modifier bounds (v0.4)
│   └── lakefile.lean
├── snap/                       # Snap package config (v0.6)
│   ├── gui/
│   │   └── truthlens.png       # Snap store icon
│   └── snapcraft.yaml
├── bridge/                     # Lean ↔ Rust proof/runtime mapping
└── README.md
```

## Build

```bash
# Rust (default — no network dependencies)
cd rust
cargo test                    # unit + doc tests
cargo test --features verify  # includes entity verification tests

# Python bindings
cd python
pip install maturin pytest
maturin develop               # build + install locally
pytest tests/ -v               # run Python tests

# Lean
cd lean
lake build        # 6 proof modules, zero sorry
```

## Roadmap

- [x] **v0.1** — Linguistic analysis: claim extraction, hedging detection, specificity scoring
- [x] **v0.2** — Confidence trajectory: detects oscillating, flat, or convergent confidence patterns using second-order dynamical system modeling
- [x] **v0.3** — Multi-response consistency, CLI (`cargo install truthlens`), colored output
- [x] **v0.4** — Entity cross-reference: verify extracted entities against Wikidata SPARQL (optional `verify` feature flag)
- [x] **v0.5** — Python bindings (PyO3) → `pip install truthlens`, Snap package
- [x] **v0.6** — Claude Code / MCP integration: local stdio MCP server, `analyze_text` + `analyze_file` tools, auto-checks AI text claims in-context
- [ ] **v0.7** — VS Code extension: analyze selection/file, inline diagnostics for docs/comments/markdown, status bar trust score
- [ ] **v0.8** — CI/CD integration: GitHub Action, fail builds on low trust score, policy thresholds (`--min-score`)
- [ ] **v0.9** — Browser extension: highlight claims in ChatGPT/Claude UI with inline trust indicators
- [ ] **v1.0** — TruthLens Platform: unified trust layer across CLI, VS Code, MCP, and CI pipelines with policy enforcement and fully local execution
- [ ] **v2.0** — Enterprise Trust System: policy engine, dashboard, audit & compliance reporting, enterprise API, team governance

### Design Principles (all versions)
- **Zero API calls by default** — every version works offline, locally, for free
- **Formally verified** — Lean 4 proofs for all scoring properties
- **Hedging = trustworthy** — a model that says "might" is more honest than one stating falsehoods with certainty
- **Fast** — microsecond analysis, no model inference required

## Why TruthLens?

Every existing hallucination detector either requires multiple LLM API calls (expensive, slow) or access to model logprobs (grey-box only). TruthLens works on **any AI output** with **zero API calls** — you paste text, you get a trust score. And the scoring properties are **formally proven** in Lean 4, which nobody else does.

## License

Apache-2.0
