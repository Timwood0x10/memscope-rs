//! Allocation life-cycle state and leak assessment.
//!
//! Two different facts about the same allocation are kept apart on purpose:
//!
//! - [`AllocationState`] is a **fact** about the physical allocation as far as the
//!   capture window observed it. It only records what happened inside the window
//!   (an allocation was seen, a release was seen, or observation simply stopped).
//! - [`LeakAssessment`] is a **conclusion** computed at a snapshot cut from the
//!   fact above plus whatever completeness information the capture layer provides.
//!
//! Keeping them separate prevents the historical bug where "no deallocation
//! timestamp" was silently treated as "leaked": a tracker drop, a session stop,
//! or a missing event never proves that the memory was leaked.
//!
//! # Accuracy contract
//!
//! * `Live` means "no release event was recorded for this allocation inside the
//!   window"; it does **not** prove the object is still reachable outside it.
//! * `Freed` is only produced from a real release observation (allocator hook or
//!   explicit instrumentation), never from a tracker/guard destruction.
//! * `Unknown` is used when observation ended before a release could be seen.
//! * A leak is only reported for `Suspected`/`Confirmed`; the default is
//!   [`LeakStatus::NotAssessed`], which reports no leak at all.

use serde::{Deserialize, Serialize};

/// Physical allocation state observed at a snapshot cut.
///
/// The variant names are serialized as `snake_case` strings so JSON consumers get
/// stable, readable values (`"live"`, `"freed"`, `"unknown"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AllocationState {
    /// The allocation was observed in the window and no release was recorded.
    #[default]
    Live,
    /// A release event for this allocation was recorded at `at_ns`.
    Freed {
        /// Session monotonic timestamp (nanoseconds) of the release observation.
        at_ns: u64,
    },
    /// Observation stopped before a release could be observed.
    Unknown {
        /// Human readable explanation of why no verdict is available.
        reason: String,
    },
}

impl AllocationState {
    /// Build the "observed, no release recorded" state.
    #[must_use]
    pub fn live() -> Self {
        Self::Live
    }

    /// Build a released state from the session monotonic timestamp.
    #[must_use]
    pub fn freed(at_ns: u64) -> Self {
        Self::Freed { at_ns }
    }

    /// Build an undetermined state with the reason observation ended.
    #[must_use]
    pub fn unknown(reason: impl Into<String>) -> Self {
        Self::Unknown {
            reason: reason.into(),
        }
    }

    /// Whether this allocation had no recorded release in the window.
    #[must_use]
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live)
    }

    /// Whether a release was actually observed for this allocation.
    #[must_use]
    pub fn is_freed(&self) -> bool {
        matches!(self, Self::Freed { .. })
    }

    /// Stable machine readable label used by DTOs and JSON envelopes.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Freed { .. } => "freed",
            Self::Unknown { .. } => "unknown",
        }
    }
}

/// How well a leak conclusion is supported by evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeakStatus {
    /// No leak rule was applied to this allocation yet.
    #[default]
    NotAssessed,
    /// Rules ran and found no evidence of a leak.
    NoEvidence,
    /// A rule produced a suspicion that still needs corroboration.
    Suspected,
    /// Independently explainable evidence proves the leak.
    Confirmed,
}

impl LeakStatus {
    /// Whether this status may be rendered as a leak in reports.
    ///
    /// `NotAssessed` and `NoEvidence` must never be displayed as leaks; doing so
    /// is what produced the historical false positives.
    #[must_use]
    pub fn is_reported_leak(self) -> bool {
        matches!(self, Self::Suspected | Self::Confirmed)
    }

    /// Stable machine readable label used by DTOs and JSON envelopes.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotAssessed => "not_assessed",
            Self::NoEvidence => "no_evidence",
            Self::Suspected => "suspected",
            Self::Confirmed => "confirmed",
        }
    }
}

/// Conclusion about whether a single allocation leaked.
///
/// The rule id and reason are only meaningful for `Suspected`/`Confirmed`; they
/// keep the conclusion traceable back to the rule that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LeakAssessment {
    /// Assessment status.
    pub status: LeakStatus,
    /// Identifier of the rule that produced the conclusion, if any.
    pub rule_id: Option<String>,
    /// Human readable justification, if any.
    pub reason: Option<String>,
}

impl LeakAssessment {
    /// Nothing was assessed yet — the conservative default.
    #[must_use]
    pub fn not_assessed() -> Self {
        Self {
            status: LeakStatus::NotAssessed,
            rule_id: None,
            reason: None,
        }
    }

    /// Rules ran and found no leak evidence.
    #[must_use]
    pub fn no_evidence() -> Self {
        Self {
            status: LeakStatus::NoEvidence,
            rule_id: None,
            reason: None,
        }
    }

    /// A rule raised a suspicion; the reason records why.
    #[must_use]
    pub fn suspected(rule_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            status: LeakStatus::Suspected,
            rule_id: Some(rule_id.into()),
            reason: Some(reason.into()),
        }
    }

    /// Proof exists; the reason records the evidence.
    #[must_use]
    pub fn confirmed(rule_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            status: LeakStatus::Confirmed,
            rule_id: Some(rule_id.into()),
            reason: Some(reason.into()),
        }
    }

    /// Whether this assessment may be rendered as a leak.
    #[must_use]
    pub fn is_reported_leak(&self) -> bool {
        self.status.is_reported_leak()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that a plain observation is reported as `Live` and never
    /// as a leak.
    /// Invariants: `Live` is not free, is not a reported leak, and serializes to
    /// the stable `"live"` label.
    #[test]
    fn live_state_is_not_a_leak() {
        let state = AllocationState::live();
        assert!(state.is_live(), "fresh observation must be Live");
        assert!(!state.is_freed(), "Live must not claim a release");
        assert_eq!(state.as_str(), "live");

        let assessment = LeakAssessment::not_assessed();
        assert!(
            !assessment.is_reported_leak(),
            "an unassessed allocation must never be reported as leaked"
        );
        assert_eq!(assessment.status.as_str(), "not_assessed");
    }

    /// Objective: Verify that only a real release observation produces `Freed`
    /// and preserves the observation timestamp.
    /// Invariants: `Freed` keeps `at_ns` untouched and is not live.
    #[test]
    fn freed_state_preserves_release_timestamp() {
        let state = AllocationState::freed(4_242);
        assert!(!state.is_live(), "a released allocation must not be Live");
        match state {
            AllocationState::Freed { at_ns } => {
                assert_eq!(at_ns, 4_242, "release timestamp must be preserved");
            }
            other => panic!("expected Freed state, got {other:?}"),
        }
        assert_eq!(state.as_str(), "freed");
    }

    /// Objective: Verify that missing observation yields `Unknown` with the
    /// supplied reason instead of a fabricated release.
    /// Invariants: The reason string is retained verbatim.
    #[test]
    fn unknown_state_keeps_reason() {
        let state = AllocationState::unknown("capture stopped before release");
        assert!(!state.is_live(), "unknown observations are not live");
        assert!(!state.is_freed(), "unknown observations are not freed");
        match &state {
            AllocationState::Unknown { reason } => assert_eq!(
                reason, "capture stopped before release",
                "unknown state must keep the diagnostic reason"
            ),
            other => panic!("expected Unknown state, got {other:?}"),
        }
        assert_eq!(state.as_str(), "unknown");
    }

    /// Objective: Verify the leak status gate that reports use.
    /// Invariants: Only Suspected/Confirmed pass the gate; NotAssessed/NoEvidence
    /// never do.
    #[test]
    fn only_suspected_and_confirmed_are_reported() {
        assert!(!LeakStatus::NotAssessed.is_reported_leak());
        assert!(!LeakStatus::NoEvidence.is_reported_leak());
        assert!(LeakStatus::Suspected.is_reported_leak());
        assert!(LeakStatus::Confirmed.is_reported_leak());
    }

    /// Objective: Verify that rule-backed assessments keep their provenance.
    /// Invariants: rule id and reason are present and the status is a reported
    /// leak for both `suspected` and `confirmed`.
    #[test]
    fn assessments_keep_rule_provenance() {
        let suspected = LeakAssessment::suspected("leak.unreleased_at_cut", "still live at cut");
        assert!(suspected.is_reported_leak());
        assert_eq!(suspected.rule_id.as_deref(), Some("leak.unreleased_at_cut"));
        assert_eq!(suspected.reason.as_deref(), Some("still live at cut"));

        let confirmed = LeakAssessment::confirmed("leak.proof", "independent evidence");
        assert!(confirmed.is_reported_leak());
        assert_eq!(confirmed.status, LeakStatus::Confirmed);

        let no_evidence = LeakAssessment::no_evidence();
        assert!(!no_evidence.is_reported_leak());
        assert!(
            no_evidence.rule_id.is_none(),
            "no-evidence assessments carry no rule id"
        );
    }

    /// Objective: Verify that the JSON contract for these types is stable for
    /// dashboard and envelope consumers.
    /// Invariants: Round-tripping through serde preserves the state and status.
    #[test]
    fn states_round_trip_through_json() {
        let state = AllocationState::freed(7);
        let json = serde_json::to_string(&state).expect("state must serialize");
        assert!(
            json.contains("freed"),
            "serialized state must use snake_case label, got {json}"
        );
        let back: AllocationState = serde_json::from_str(&json).expect("state must deserialize");
        assert_eq!(back, state, "round-trip must preserve the state");

        let status = LeakStatus::NoEvidence;
        let json = serde_json::to_string(&status).expect("status must serialize");
        assert_eq!(json, "\"no_evidence\"", "status label must be stable");

        let default_state = AllocationState::default();
        assert!(
            default_state.is_live(),
            "default allocation state must be Live, not freed"
        );
        assert_eq!(
            LeakStatus::default(),
            LeakStatus::NotAssessed,
            "default leak status must be NotAssessed"
        );
    }
}
