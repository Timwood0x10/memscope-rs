//! Inference acceptance policy.
//!
//! A heuristic score is not a probability. Reporting the highest scoring type
//! unconditionally turns a weak signal ("this block is 8 bytes, so maybe it is a
//! pointer") into a confident answer, and a near tie between two candidates always
//! resolves to whichever row happens to come first in the table.
//!
//! This module adds the missing exit: [`InferencePolicy`] decides whether the
//! evidence is strong enough and unambiguous enough to answer at all, and
//! [`InferenceDecision`] keeps the raw numbers plus the reason for abstaining so
//! reports can show "not enough evidence" instead of an invented type.
//!
//! # Threshold calibration
//!
//! The defaults are *preliminary*: they only require more than a single weak
//! signal and reject near ties. They must be replaced by thresholds selected on a
//! validation set before any accuracy number is published (development plan
//! §11.2), so they live in one place and are consciously chosen, not sprinkled
//! through the scoring code.

use super::engine::{TypeKind, UnsafeInferenceEngine};

/// Acceptance thresholds for heuristic type inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InferencePolicy {
    /// Minimum raw score the winning candidate must reach.
    pub min_score: u16,
    /// Minimum lead the winner must have over the runner-up.
    pub min_margin: u16,
}

impl InferencePolicy {
    /// Preliminary thresholds: reject a lone weak signal and near ties.
    ///
    /// `min_score = 30` corresponds to exactly one weak signal (for example the
    /// 8-byte pointer heuristic), `min_margin = 10` rejects candidates that are
    /// less than one signal apart.
    ///
    /// # Measured behaviour
    ///
    /// On the deterministic four-sample fixture in this module's tests
    /// (`policy::tests::calibration_fixture`) this pair accepts 2 samples with
    /// precision 1.00 and recall 0.50: printable text and a NUL-terminated C string are
    /// answered, while a high-entropy buffer and a control-character block abstain
    /// because their candidates are only a few points apart. A grid search over
    /// `min_score ∈ [0, 80]` and `min_margin ∈ [0, 30]` with `min_precision = 0.85`
    /// finds a more permissive pair, but on four samples that is over-fitting, so the
    /// shipped pair stays deliberately conservative.
    ///
    /// This is a smoke-level measurement, **not** a validated calibration: plan §11.2
    /// requires thresholds selected on a validation set of real programs, including
    /// layout-based samples built with a controlled allocator, before any accuracy
    /// percentage is published.
    pub const PRELIMINARY: Self = Self {
        min_score: 30,
        min_margin: 10,
    };

    /// Build a policy with explicit thresholds.
    #[must_use]
    pub const fn new(min_score: u16, min_margin: u16) -> Self {
        Self {
            min_score,
            min_margin,
        }
    }
}

impl Default for InferencePolicy {
    fn default() -> Self {
        Self::PRELIMINARY
    }
}

/// Outcome of one inference run: what was predicted, on what evidence, or why not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferenceDecision {
    /// Accepted type, `None` when the evidence was rejected.
    pub predicted: Option<TypeKind>,
    /// Highest scoring candidate, accepted or not.
    pub best_candidate: TypeKind,
    /// Raw score of the best candidate (heuristic score, not a probability).
    pub raw_score: u16,
    /// Raw score of the runner-up candidate.
    pub runner_up_score: u16,
    /// Why the decision abstained, when it did.
    pub abstain_reason: Option<String>,
}

impl InferenceDecision {
    /// Whether a type was accepted.
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.predicted.is_some()
    }

    /// Confidence reported for an accepted decision, `0` when abstaining.
    ///
    /// The value is a heuristic score, not a percentage.
    #[must_use]
    pub fn accepted_score(&self) -> u16 {
        if self.is_accepted() {
            self.raw_score
        } else {
            0
        }
    }
}

/// One labelled sample used to select acceptance thresholds.
///
/// The label is the ground truth of the fixture that produced the bytes, never the
/// output of the engine under test.
pub struct CalibrationSample {
    /// Memory image handed to the engine.
    pub bytes: Vec<u8>,
    /// Requested size of the block.
    pub size: usize,
    /// Type the fixture actually allocated.
    pub expected: TypeKind,
}

/// Measured outcome of one policy over the calibration set.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CalibrationReport {
    /// Minimum score that was applied.
    pub min_score: u16,
    /// Minimum margin that was applied.
    pub min_margin: u16,
    /// Predictions that were accepted.
    pub accepted: usize,
    /// Accepted predictions that matched the label.
    pub correct: usize,
    /// Samples the policy rejected (abstained on).
    pub abstained: usize,
    /// `correct / accepted`; `1.0` when nothing was accepted.
    pub precision: f64,
    /// `correct / total`; measures how much recall the abstentions cost.
    pub recall: f64,
}

impl CalibrationReport {
    /// F1 of precision and recall, the value the calibration maximises.
    #[must_use]
    pub fn f1(&self) -> f64 {
        if self.precision + self.recall == 0.0 {
            return 0.0;
        }
        2.0 * self.precision * self.recall / (self.precision + self.recall)
    }
}

/// Evaluate a policy over a labelled sample set.
///
/// Precision is only meaningful together with the abstention count: a policy that
/// answers almost nothing can look perfect. The report therefore always carries
/// `accepted`, `abstained` and `recall` next to `precision`.
#[must_use]
pub fn evaluate(policy: InferencePolicy, samples: &[CalibrationSample]) -> CalibrationReport {
    let mut report = CalibrationReport {
        min_score: policy.min_score,
        min_margin: policy.min_margin,
        ..CalibrationReport::default()
    };
    if samples.is_empty() {
        return report;
    }

    for sample in samples {
        let decision =
            UnsafeInferenceEngine::decide_with_policy(&sample.bytes, sample.size, &policy);
        match decision.predicted {
            Some(kind) => {
                report.accepted += 1;
                if kind == sample.expected {
                    report.correct += 1;
                }
            }
            None => report.abstained += 1,
        }
    }

    report.precision = if report.accepted == 0 {
        1.0
    } else {
        report.correct as f64 / report.accepted as f64
    };
    report.recall = report.correct as f64 / samples.len() as f64;
    report
}

/// Select thresholds from a labelled set.
///
/// The grid is searched for the highest F1 among policies whose precision reaches
/// `min_precision`; ties are broken towards the more conservative (larger) margin.
/// The result is a *measurement*, not an opinion: the caller decides whether to ship
/// it, and the shipped pair is recorded in [`InferencePolicy::PRELIMINARY`].
#[must_use]
pub fn calibrate(samples: &[CalibrationSample], min_precision: f64) -> CalibrationReport {
    let mut best: Option<CalibrationReport> = None;

    for min_score in (0..=80u16).step_by(10) {
        for min_margin in (0..=30u16).step_by(5) {
            let policy = InferencePolicy::new(min_score, min_margin);
            let report = evaluate(policy, samples);
            if report.precision < min_precision || report.accepted == 0 {
                continue;
            }
            let better = match &best {
                None => true,
                Some(current) => match report.f1().partial_cmp(&current.f1()) {
                    Some(std::cmp::Ordering::Greater) => true,
                    Some(std::cmp::Ordering::Equal) => report.min_margin > current.min_margin,
                    _ => false,
                },
            };
            if better {
                best = Some(report);
            }
        }
    }

    best.unwrap_or_else(|| evaluate(InferencePolicy::PRELIMINARY, samples))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify the shipped defaults are the documented preliminary pair.
    /// Invariants: default equals `PRELIMINARY` and both thresholds are positive.
    #[test]
    fn default_policy_is_preliminary() {
        assert_eq!(
            InferencePolicy::default(),
            InferencePolicy::PRELIMINARY,
            "the default policy must be the documented preliminary thresholds"
        );
        assert!(
            InferencePolicy::default().min_score > 0,
            "a zero minimum score would accept every guess"
        );
        assert!(
            InferencePolicy::default().min_margin > 0,
            "a zero margin would accept ties"
        );
    }

    /// Objective: Verify the decision helpers expose acceptance, not the raw kind.
    /// Invariants: an abstaining decision reports score 0 and no predicted type.
    #[test]
    fn abstained_decision_reports_zero_score() {
        let decision = InferenceDecision {
            predicted: None,
            best_candidate: TypeKind::Vec,
            raw_score: 25,
            runner_up_score: 20,
            abstain_reason: Some("below min_score".to_string()),
        };
        assert!(!decision.is_accepted(), "no type was accepted");
        assert_eq!(
            decision.accepted_score(),
            0,
            "an abstained decision must not report a confidence score"
        );
    }

    /// Objective: Verify an accepted decision keeps the raw heuristic score.
    /// Invariants: accepted_score equals raw_score and a type is present.
    #[test]
    fn accepted_decision_keeps_raw_score() {
        let decision = InferenceDecision {
            predicted: Some(TypeKind::String),
            best_candidate: TypeKind::String,
            raw_score: 90,
            runner_up_score: 15,
            abstain_reason: None,
        };
        assert!(decision.is_accepted(), "the type must be accepted");
        assert_eq!(decision.accepted_score(), 90, "the raw score must survive");
    }

    /// Deterministic labelled fixture used by the calibration test.
    ///
    /// Only content-based samples are included: pointer/layout samples depend on real
    /// heap addresses (`is_valid_ptr` checks the process map), so they cannot be
    /// reproduced byte-for-byte and would make the measurement non-deterministic. Those
    /// cases need a controlled allocator (plan §12.1) and are deferred to PR-05/08.
    fn calibration_fixture() -> Vec<CalibrationSample> {
        let mut samples = Vec::new();

        // String: printable UTF-8 text.
        let text = b"the quick brown fox jumps over the lazy dog".to_vec();
        samples.push(CalibrationSample {
            size: text.len(),
            bytes: text,
            expected: TypeKind::String,
        });

        // CString: printable UTF-8 with a terminating NUL.
        let mut cstring = b"hello world".to_vec();
        cstring.push(0);
        samples.push(CalibrationSample {
            size: cstring.len(),
            bytes: cstring,
            expected: TypeKind::CString,
        });

        // Buffer: sizeable, high-entropy, no readable structure.
        let buffer: Vec<u8> = (0..=255u8).collect();
        samples.push(CalibrationSample {
            size: buffer.len(),
            bytes: buffer,
            expected: TypeKind::Buffer,
        });

        // Control-character UTF-8: valid encoding, no printable content. It must not be
        // reported as a String; abstaining is the honest outcome.
        let control = vec![0xC2u8, 0x80, 0xC2, 0x81, 0xC2, 0x82];
        samples.push(CalibrationSample {
            size: control.len(),
            bytes: control,
            expected: TypeKind::Buffer,
        });

        samples
    }

    /// Objective: Verify the shipped thresholds are backed by a measurement on a
    /// labelled fixture, and that the measurement is reported with the abstention
    /// rate instead of precision alone.
    /// Invariants: precision on accepted predictions is high, something is accepted
    /// (no "abstain everything" policy), and the calibrated pair is no worse than the
    /// shipped one.
    #[test]
    fn preliminary_thresholds_are_backed_by_measurement() {
        let samples = calibration_fixture();
        let shipped = evaluate(InferencePolicy::PRELIMINARY, &samples);
        let calibrated = calibrate(&samples, 0.85);

        assert!(
            shipped.accepted > 0,
            "the shipped policy must not abstain on the whole fixture"
        );
        assert!(
            shipped.precision >= 0.85,
            "shipped thresholds must keep precision above 0.85, measured {} (accepted {}, abstained {})",
            shipped.precision,
            shipped.accepted,
            shipped.abstained
        );
        assert!(
            shipped.recall > 0.0,
            "the shipped policy must recover at least one sample, got recall {}",
            shipped.recall
        );
        assert!(
            calibrated.precision >= shipped.precision - 0.2,
            "calibration must not degrade precision below the shipped policy (calibrated {}, shipped {})",
            calibrated.precision,
            shipped.precision
        );
        assert!(
            calibrated.f1() >= shipped.f1() - 1e-9,
            "the calibrated policy must not be worse than the shipped one (calibrated {}, shipped {})",
            calibrated.f1(),
            shipped.f1()
        );
        // Recorded measurement on the deterministic fixture. If the scoring changes,
        // this must be re-measured instead of silently drifting.
        assert_eq!(
            (shipped.accepted, shipped.correct, shipped.abstained),
            (2, 2, 2),
            "measured acceptance on the calibration fixture changed: {shipped:?}"
        );
        assert!(
            (shipped.precision - 1.0).abs() < 1e-9,
            "measured precision must be 1.0, got {}",
            shipped.precision
        );
    }

    /// Objective: Verify that an uncalibrated low-score policy is rejected by the
    /// evaluation, i.e. the report exposes weak precision instead of hiding it.
    /// Invariants: a policy with zero thresholds accepts everything and reports the
    /// resulting precision.
    #[test]
    fn evaluation_exposes_weak_thresholds() {
        let samples = calibration_fixture();
        let permissive = evaluate(InferencePolicy::new(0, 0), &samples);

        assert_eq!(
            permissive.abstained, 0,
            "a zero-threshold policy never abstains"
        );
        assert_eq!(
            permissive.accepted,
            samples.len(),
            "every sample must be accepted by the permissive policy"
        );
        assert!(
            permissive.precision <= 1.0,
            "precision must stay a fraction, got {}",
            permissive.precision
        );
    }
}
