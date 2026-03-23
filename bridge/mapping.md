# Lean ↔ Rust Mapping

This document maps the major Lean proof modules to the Rust implementation modules that realize the corresponding behavior.

## 1. Score Bounds

**Lean:** `lean/TruthLens/ScoreBounds.lean`

**Rust:**
- `rust/src/scorer.rs`
- `rust/src/lib.rs`

**Covers:**
- signal/value bounds
- weighted score aggregation
- final clamping into valid score range

**Runtime responsibility:**
Rust scoring functions must keep all exposed trust scores within valid bounds and preserve the weighting model documented by the proofs.

---

## 2. Monotonicity

**Lean:** `lean/TruthLens/Monotonicity.lean`

**Rust:**
- `rust/src/scorer.rs`

**Covers:**
- improving signals should not worsen the score
- better component values should improve or preserve total outcome

**Runtime responsibility:**
Any scoring refactor in Rust should preserve monotonic relationships between signal improvements and aggregate trust score.

---

## 3. Composition

**Lean:** `lean/TruthLens/Composition.lean`

**Rust:**
- `rust/src/scorer.rs`
- `rust/src/lib.rs`

**Covers:**
- passage-level aggregation
- average + worst-claim penalty composition
- order independence / determinism

**Runtime responsibility:**
Passage scoring in Rust should remain deterministic, bounded, and insensitive to claim ordering except where explicitly modeled.

---

## 4. Trajectory

**Lean:** `lean/TruthLens/Trajectory.lean`

**Rust:**
- `rust/src/trajectory.rs`
- `rust/src/lib.rs`

**Covers:**
- trajectory modifier bounds
- damping positivity
- transition/count constraints

**Runtime responsibility:**
Trajectory analysis should preserve the bounded modifier behavior and stable interpretation proven in Lean.

---

## 5. Consistency

**Lean:** `lean/TruthLens/Consistency.lean`

**Rust:**
- `rust/src/consistency.rs`
- `rust/src/lib.rs`

**Covers:**
- contradiction counting bounds
- consistency score limits
- symmetry of contradiction relationships
- agreement/uniqueness constraints

**Runtime responsibility:**
Rust consistency checking should keep contradiction logic symmetric and maintain bounded consistency/uniqueness calculations.

---

## 6. Verification

**Lean:** `lean/TruthLens/Verification.lean`

**Rust:**
- `rust/src/entity.rs`
- `rust/src/lib.rs`
- optional `verify` feature paths in the Rust crate

**Covers:**
- verification modifier bounds
- combination of verification with base score / trajectory modifier
- partitioning of verified / contradicted / unknown entities
- monotonic effects of more verified vs more contradicted entities

**Runtime responsibility:**
Entity verification logic and score adjustment paths must preserve the bounded modifier semantics proven in Lean.

---

## 7. Claim Extraction and Parsing

**Lean:** no dedicated proof module yet

**Rust:**
- `rust/src/claim.rs`

**Notes:**
Claim extraction and heuristic parsing currently live only in Rust. These are runtime heuristics and are not yet formally modeled in Lean.

---

## 8. CLI / Packaging Layer

**Lean:** none

**Rust / Python / Snap:**
- `rust/src/main.rs`
- `python/src/lib.rs`
- `snap/snapcraft.yaml`

**Notes:**
Packaging, bindings, CLI UX, and distribution concerns are implementation-only and intentionally outside the Lean proof scope.

---

## Change Management Guidance

When modifying the project:

- If a Rust scoring rule changes, check whether the corresponding Lean proof module should change too.
- If a Lean proof expands the contract, confirm the mapped Rust module still satisfies that contract.
- If a new major feature adds score-affecting logic, it should be added to this mapping.

## Future Extensions

Potential future improvements for the bridge:
- machine-readable theorem ↔ module index
- CI checks for mapping completeness
- generated conformance tests
- proof-linked documentation for released APIs
