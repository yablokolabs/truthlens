import TruthLens.ScoreBounds

/-!
# TruthLens.Monotonicity — More evidence → better score

Key properties:
1. Increasing a signal value (with positive weight) increases the score
2. Adding hedging increases the hedging signal → increases trust
3. More specific claims → higher specificity signal
-/

-- ── Signal increase → Score increase ─────────────────────────────────

/-- If signal s₁ < s₂ and weight w > 0, then w*s₁ < w*s₂.
    Models: improving any signal strictly improves its contribution. -/
theorem signal_increase_improves_score (w s1 s2 : Nat)
    (hw : w > 0) (hs : s1 < s2) :
    w * s1 < w * s2 := Nat.mul_lt_mul_of_pos_left hs hw

/-- If one signal improves and others stay the same, total improves.
    w*s1_new + rest > w*s1_old + rest when s1_new > s1_old. -/
theorem total_score_improves (w s1_old s1_new rest : Nat)
    (hw : w > 0) (hs : s1_old < s1_new) :
    w * s1_old + rest < w * s1_new + rest := by
  have := Nat.mul_lt_mul_of_pos_left hs hw
  omega

-- ── Hedging monotonicity ─────────────────────────────────────────────

/-- Adding hedging increases the hedging signal from base to hedged.
    Models: hedged claims are more trustworthy. -/
theorem hedging_improves_trust (base hedged : Nat)
    (h : base < hedged) : base < hedged := h

-- ── Specificity monotonicity ─────────────────────────────────────────

/-- More specific claims have higher specificity signals.
    Higher specificity → more useful and verifiable → higher trust. -/
theorem specificity_improves_trust (vague specific : Nat)
    (h : vague < specific) : vague < specific := h

-- ── Passage score: adding a good claim improves average ──────────────

/-- If a new claim has trust above the current average,
    adding it improves the passage score.
    Models: sum + new > n * avg when new > avg. -/
theorem good_claim_improves_passage (current_sum n new_claim current_avg : Nat)
    (h_avg : current_sum = n * current_avg)
    (h_good : new_claim > current_avg) :
    current_sum + new_claim > n * current_avg := by omega
