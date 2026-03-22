import Lake
open Lake DSL

package «truthlens-proofs» where
  leanOptions := #[
    ⟨`autoImplicit, false⟩
  ]

@[default_target]
lean_lib «TruthLens» where
  srcDir := "."
  roots := #[`TruthLens.ScoreBounds, `TruthLens.Monotonicity, `TruthLens.Composition, `TruthLens.Trajectory, `TruthLens.Consistency, `TruthLens.Verification]
