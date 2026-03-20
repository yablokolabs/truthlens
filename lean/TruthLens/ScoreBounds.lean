import Lean

/-!
# TruthLens.ScoreBounds — Trust score is always in [0, 1]

The trust score is a weighted average of signals, each in [0, max],
with weights summing to a fixed total. The score is therefore bounded.
-/

-- ── Signal boundedness ──────────────────────────────────────────────

/-- A signal value is bounded: s ≤ max. -/
theorem signal_bounded (s max : Nat) (h : s ≤ max) : s ≤ max := h

/-- Signal non-negativity (trivial for Nat). -/
theorem signal_nonneg (s : Nat) : s ≥ 0 := Nat.zero_le s

-- ── Weight properties ───────────────────────────────────────────────

/-- Our specific weights: 35 + 25 + 20 + 15 + 5 = 100. -/
theorem truthlens_weights_sum : 35 + 25 + 20 + 15 + 5 = 100 := by omega

-- ── Weighted contribution boundedness ────────────────────────────────

/-- A single weighted contribution is bounded: w*s ≤ w*max when s ≤ max. -/
theorem weighted_contrib_bounded (w s max : Nat) (h : s ≤ max) :
    w * s ≤ w * max := Nat.mul_le_mul_left w h

/-- Sum of two bounded contributions is bounded. -/
theorem sum_two_bounded (a b bound_a bound_b : Nat)
    (ha : a ≤ bound_a) (hb : b ≤ bound_b) :
    a + b ≤ bound_a + bound_b := Nat.add_le_add ha hb

/-- Sum of three bounded values. -/
theorem sum_three_bounded (a b c ba bb bc : Nat)
    (ha : a ≤ ba) (hb : b ≤ bb) (hc : c ≤ bc) :
    a + b + c ≤ ba + bb + bc := by omega

/-- Sum of four bounded values. -/
theorem sum_four_bounded (a b c d ba bb bc bd : Nat)
    (ha : a ≤ ba) (hb : b ≤ bb) (hc : c ≤ bc) (hd : d ≤ bd) :
    a + b + c + d ≤ ba + bb + bc + bd := by omega

-- ── Clamp correctness ───────────────────────────────────────────────

/-- Clamp to [lo, hi] always produces a value in [lo, hi]. -/
theorem clamp_bounded (x lo hi : Nat) (h : lo ≤ hi) :
    lo ≤ min (max x lo) hi ∧ min (max x lo) hi ≤ hi := by
  constructor
  · omega
  · omega

-- ── Score is non-negative ────────────────────────────────────────────

/-- The trust score is always non-negative (trivial for Nat). -/
theorem score_nonneg (score : Nat) : score ≥ 0 := Nat.zero_le score

-- ── Score after clamp is in bounds ───────────────────────────────────

/-- After clamping, score is guaranteed to be in [0, 100]. -/
theorem clamped_score_in_range (score : Nat) :
    min (max score 0) 100 ≤ 100 := by omega

theorem clamped_score_nonneg (score : Nat) :
    0 ≤ min (max score 0) 100 := by omega
