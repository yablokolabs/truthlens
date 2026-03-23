import pytest
from truthlens import (
    analyze,
    analyze_with_verification,
    check_consistency,
    extract_claims,
    extract_entities,
)


def test_analyze_basic():
    report = analyze("Einstein was born in 1879 in Ulm, Germany.")
    assert 0.0 <= report["score"] <= 1.0
    assert "risk_level" in report
    assert "claims" in report
    assert len(report["claims"]) >= 1


def test_analyze_empty():
    report = analyze("")
    assert report["total_claims"] == 0


def test_analyze_hedged():
    report = analyze("Climate change might be linked to hurricane frequency.")
    assert report["score"] > 0.0


def test_analyze_with_verification():
    report = analyze_with_verification("Albert Einstein was born in 1879.")
    assert 0.0 <= report["score"] <= 1.0
    has_verification = any(c.get("verification") is not None for c in report["claims"])
    assert has_verification


def test_check_consistency():
    report = check_consistency([
        "Einstein was born in 1879 in Ulm.",
        "Einstein was born in 1879 in Munich.",
    ])
    assert "consistency_score" in report
    assert 0.0 <= report["consistency_score"] <= 1.0


def test_extract_claims():
    claims = extract_claims("Einstein was born in 1879. He developed relativity.")
    assert isinstance(claims, list)
    assert len(claims) >= 1
    assert "text" in claims[0]


def test_extract_entities():
    entities = extract_entities("Albert Einstein was born in 1879 in Ulm.")
    assert isinstance(entities, list)
    assert "Albert Einstein" in entities
    assert "1879" in entities


def test_consistency_identical():
    report = check_consistency([
        "Einstein was born in 1879.",
        "Einstein was born in 1879.",
    ])
    assert report["consistency_score"] > 0.5


def test_analyze_returns_dict():
    result = analyze("The sky is blue.")
    assert isinstance(result, dict)


def test_types():
    report = analyze("Einstein published 300 papers.")
    assert isinstance(report["score"], float)
    assert isinstance(report["total_claims"], int)
    assert isinstance(report["claims"], list)
