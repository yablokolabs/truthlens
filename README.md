# TruthLens 🔍

**AI Hallucination Detector — Formally Verified Trust Scoring for LLM Outputs**

Analyze AI-generated text for hallucination risk. No API keys needed. No LLM calls required. Fast, local, and formally verified.

## Quick Start

```bash
cd rust
cargo run --release
```

```rust
use truthlens::analyze;

let report = analyze("Einstein was born in 1879 in Ulm, Germany.");
println!("Trust: {:.0}% — {}", report.score * 100.0, report.risk_level);
// Trust: 52% — HIGH
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

## Examples

### Factual text
```
"Albert Einstein was born on March 14, 1879, in Ulm, Germany."
→ 🔴 52% HIGH — specific verifiable claim, verify independently
```

### Well-hedged text
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
├── rust/                       # Core library + CLI
│   ├── src/
│   │   ├── lib.rs              # Public API: analyze()
│   │   ├── claim.rs            # Claim extraction + linguistic analysis
│   │   ├── scorer.rs           # Trust scoring + signal aggregation
│   │   └── main.rs             # CLI demo
│   └── Cargo.toml
├── lean/                       # Formal proofs
│   ├── TruthLens/
│   │   ├── ScoreBounds.lean    # Score ∈ [0, 1], weight sum, clamp
│   │   ├── Monotonicity.lean   # Better signals → better score
│   │   └── Composition.lean    # Passage aggregation properties
│   └── lakefile.lean
├── bridge/                     # Lean ↔ Rust mapping (coming)
└── README.md
```

## Build

```bash
# Rust
cd rust
cargo test       # 10 tests (9 unit + 1 doc)
cargo run         # demo with examples

# Lean
cd lean
lake build        # compile all proofs
```

## Roadmap

- [x] **v0.1** — Linguistic analysis (no LLM, fast)
- [ ] **v0.2** — Self-consistency checker (multi-sample, needs LLM)
- [ ] **v0.3** — Entity cross-reference (Wikidata lookup)
- [ ] **v0.4** — Python bindings (PyO3) → `pip install truthlens`
- [ ] **v0.5** — Browser extension (Chrome/Firefox)
- [ ] **v1.0** — API server + dashboard

## Why TruthLens?

Every existing hallucination detector either requires multiple LLM API calls (expensive, slow) or access to model logprobs (grey-box only). TruthLens works on **any AI output** with **zero API calls** — you paste text, you get a trust score. And the scoring properties are **formally proven** in Lean 4, which nobody else does.

## License

Apache-2.0
