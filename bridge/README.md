# Bridge

This directory connects the formal Lean proofs in `lean/TruthLens/` with the runtime Rust implementation in `rust/src/`.

The goal of the bridge is not FFI or code generation. Instead, it provides a clear mapping between:
- **proved properties** (Lean)
- **runtime modules** (Rust)
- **behavioral responsibilities** (what each implementation is expected to satisfy)

## Purpose

TruthLens has two layers:

1. **Lean proofs** — establish score bounds, monotonicity, composition rules, trajectory properties, consistency behavior, and verification modifier bounds.
2. **Rust implementation** — executes the actual scoring, analysis, consistency checking, and verification logic used by the CLI and published libraries.

The bridge documents how those two layers correspond.

## Contents

- `mapping.md` — theorem-family → implementation-module mapping

## Scope

This bridge currently provides:
- documentation of proof/runtime alignment
- a maintainable contract for future contributors
- a place to expand into stronger CI validation later

It does **not yet** provide:
- automatic proof extraction into Rust
- generated tests from Lean theorems
- machine-checked equivalence between Lean definitions and Rust code

Those can be added in future versions if needed.
