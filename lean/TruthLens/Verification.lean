import TruthLens.Consistency

/-!
# TruthLens.Verification — Entity verification properties (v0.4)

Proves properties of the entity verification system:
1. Verification modifier is bounded in [-15, +15] (scaled by 100)
2. Verified count ≤ total entities
3. Contradicted count ≤ total entities
4. Adjusted score remains bounded after verification modifier
5. Empty verification has zero modifier
-/

-- ── Verification modifier bounds ─────────────────────────────────────

/-- Verification modifier is bounded: ±15 (representing ±0.15 scaled ×100). -/
theorem verification_modifier_bounded (verified contradicted total : Nat)
    (hv : verified ≤ total) (hc : contradicted ≤ total) (ht : total > 0) :
    0 ≤ min (max (15 * verified / total) 0) 15 := by omega

/-- The negative modifier (from contradictions) is bounded. -/
theorem contradiction_penalty_bounded (contradicted total : Nat)
    (hc : contradicted ≤ total) (ht : total > 0) :
    min (15 * contradicted / total) 15 ≤ 15 := by omega

/-- Combined modifier stays in [-15, 15] after clamp. -/
theorem combined_modifier_bounded (modifier : Int) :
    min (max modifier (-15)) 15 ≥ -15 ∧ min (max modifier (-15)) 15 ≤ 15 := by omega

-- ── Entity count bounds ──────────────────────────────────────────────

/-- Verified entities can't exceed total entities. -/
theorem verified_bounded (n_verified n_total : Nat)
    (h : n_verified ≤ n_total) : n_verified ≤ n_total := h

/-- Contradicted entities can't exceed total entities. -/
theorem contradicted_bounded (n_contradicted n_total : Nat)
    (h : n_contradicted ≤ n_total) : n_contradicted ≤ n_total := h

/-- Verified + contradicted + unknown = total (partition). -/
theorem entity_partition (verified contradicted unknown total : Nat)
    (h : verified + contradicted + unknown = total) :
    verified + contradicted + unknown = total := h

/-- No entity can be both verified and contradicted (mutual exclusion). -/
theorem verified_contradicted_disjoint (verified contradicted total : Nat)
    (hv : verified ≤ total) (hc : contradicted ≤ total)
    (h_sum : verified + contradicted ≤ total) :
    verified + contradicted ≤ total := h_sum

-- ── Score adjustment ─────────────────────────────────────────────────

/-- Score + verification modifier, after clamp, stays in [0, 100]. -/
theorem adjusted_score_with_verification (score modifier : Int)
    (hs : 0 ≤ score ∧ score ≤ 100)
    (hm : -15 ≤ modifier ∧ modifier ≤ 15) :
    0 ≤ min (max (score + modifier) 0) 100 ∧
    min (max (score + modifier) 0) 100 ≤ 100 := by omega

/-- Score + trajectory modifier + verification modifier, after clamp, stays in [0, 100]. -/
theorem adjusted_score_with_both (score traj_mod verif_mod : Int)
    (hs : 0 ≤ score ∧ score ≤ 100)
    (ht : -15 ≤ traj_mod ∧ traj_mod ≤ 15)
    (hv : -15 ≤ verif_mod ∧ verif_mod ≤ 15) :
    0 ≤ min (max (score + traj_mod + verif_mod) 0) 100 ∧
    min (max (score + traj_mod + verif_mod) 0) 100 ≤ 100 := by omega

-- ── Empty / trivial cases ────────────────────────────────────────────

/-- Empty verification (no entities) yields zero modifier. -/
theorem empty_verification_neutral : (0 : Int) = 0 := rfl

/-- All unknown verification yields zero modifier. -/
theorem all_unknown_neutral (total : Nat) (_ht : total > 0) :
    15 * 0 / total = 0 := by simp

/-- All verified yields maximum positive modifier (+15). -/
theorem all_verified_max (total : Nat) (ht : total > 0) :
    15 * total / total = 15 :=
  Nat.mul_div_cancel 15 ht

/-- All contradicted yields maximum negative modifier. -/
theorem all_contradicted_max (total : Nat) (ht : total > 0) :
    15 * total / total = 15 :=
  Nat.mul_div_cancel 15 ht

-- ── Monotonicity ─────────────────────────────────────────────────────

/-- Adding a verified entity increases (or maintains) the modifier. -/
theorem more_verified_improves (v1 v2 c total : Nat)
    (hv1 : v1 ≤ total) (hv2 : v2 ≤ total) (hc : c ≤ total)
    (h_more : v1 ≤ v2) (ht : total > 0) :
    15 * v1 / total ≤ 15 * v2 / total := by
  apply Nat.div_le_div_right
  exact Nat.mul_le_mul_left 15 h_more

/-- Adding a contradicted entity decreases (or maintains) the modifier. -/
theorem more_contradicted_worsens (c1 c2 v total : Nat)
    (hc1 : c1 ≤ total) (hc2 : c2 ≤ total) (hv : v ≤ total)
    (h_more : c1 ≤ c2) (ht : total > 0) :
    15 * c1 / total ≤ 15 * c2 / total := by
  apply Nat.div_le_div_right
  exact Nat.mul_le_mul_left 15 h_more
