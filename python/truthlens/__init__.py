"""TruthLens — AI Hallucination Detector.

Analyze AI-generated text for hallucination risk using linguistic signals,
claim decomposition, and formally verified trust scoring.

No API keys needed. No LLM calls required. Fast, local, formally verified.
"""

from truthlens._truthlens import (
    analyze,
    analyze_with_verification,
    check_consistency,
    extract_claims,
    extract_entities,
)

__version__ = "0.6.0"

__all__ = [
    "analyze",
    "analyze_with_verification",
    "check_consistency",
    "extract_claims",
    "extract_entities",
]
