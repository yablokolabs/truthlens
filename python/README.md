# TruthLens Python

**AI Hallucination Detector — Formally Verified Trust Scoring for LLM Outputs**

Python bindings for [TruthLens](https://github.com/yablokolabs/truthlens), powered by PyO3.

## Install

```bash
pip install truthlens
```

## Usage

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
for c in result["contradictions"]:
    print(f"  Conflict: {c['claim_a']} vs {c['claim_b']}")

# Extract atomic claims
claims = extract_claims("Python was created in 1991. It is widely used.")
for c in claims:
    print(f"  [{c['specificity']:.2f}] {c['text']}")

# Extract named entities
entities = extract_entities("Marie Curie won the Nobel Prize in 1903.")
print(entities)  # ['1903', 'Marie Curie']
```

## License

Apache-2.0
