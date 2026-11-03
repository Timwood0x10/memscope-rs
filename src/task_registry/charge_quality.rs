//! Charge evidence is captured with allocation events, never read from a later live cache.

use super::{ChargeOutcome, ChargeQuality, TaskIdRegistry};
use crate::event_store::{MemoryEvent, MemoryEventType};
use std::collections::BTreeMap;

impl TaskIdRegistry {
    /// Project window and per-task charge quality from one immutable event cut.
    ///
    /// Reallocations and annotations do not make another registry charge attempt. Legacy
    /// allocation events without an outcome remain readable, but cannot certify completeness.
    pub fn charge_quality_from_events(
        events: &[MemoryEvent],
    ) -> (ChargeQuality, BTreeMap<u64, ChargeQuality>) {
        let mut window = ChargeQuality::default();
        let mut tasks: BTreeMap<u64, ChargeQuality> = BTreeMap::new();
        for event in events
            .iter()
            .filter(|event| event.event_type == MemoryEventType::Allocate)
        {
            window.record(event.charge_outcome, event.size as u64);
            if let Some(task) = event.task_id {
                tasks
                    .entry(task)
                    .or_default()
                    .record(event.charge_outcome, event.size as u64);
            }
        }
        (window, tasks)
    }
}

impl ChargeQuality {
    fn record(&mut self, outcome: Option<ChargeOutcome>, bytes: u64) {
        match outcome {
            Some(ChargeOutcome::Charged) => {}
            Some(ChargeOutcome::Deferred) => {
                self.deferred_charges += 1;
                self.deferred_bytes += bytes;
            }
            Some(ChargeOutcome::Unattributed) => {
                self.unattributed_charges += 1;
                self.unattributed_bytes += bytes;
            }
            None => {
                self.unknown_charges += 1;
                self.unknown_bytes += bytes;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Preserve per-task gaps and unknown legacy evidence through serialization.
    /// Invariants: only allocations contribute charges; zero-byte attempts still count.
    #[test]
    fn charge_evidence_round_trips_without_inventing_completeness() {
        let mut events = Vec::new();
        for (task, size, outcome) in [
            (Some(1), 64, Some(ChargeOutcome::Charged)),
            (Some(2), 128, Some(ChargeOutcome::Deferred)),
            (Some(2), 0, Some(ChargeOutcome::Deferred)),
            (None, 32, Some(ChargeOutcome::Unattributed)),
            (Some(3), 16, None),
        ] {
            let mut event = MemoryEvent::allocate(events.len(), size, 1);
            event.task_id = task;
            event.charge_outcome = outcome;
            events.push(event);
        }
        // Other event kinds have no new charge attempt, even if an external producer
        // erroneously copied a charge field into one of them.
        let mut release = MemoryEvent::deallocate(1, 128, 1);
        release.charge_outcome = Some(ChargeOutcome::Deferred);
        events.push(release);
        events.push(MemoryEvent::reallocate_moved(0, 8, 64, 256, 1));
        let encoded = serde_json::to_string(&events).expect("serialize charge evidence");
        let restored: Vec<MemoryEvent> =
            serde_json::from_str(&encoded).expect("read captured events");
        let (window, tasks) = TaskIdRegistry::charge_quality_from_events(&restored);
        assert_eq!(
            (window.deferred_charges, window.deferred_bytes),
            (2, 128),
            "zero-byte attempts must remain visible"
        );
        assert_eq!(
            (window.unattributed_charges, window.unattributed_bytes),
            (1, 32),
            "unattributed bytes belong to the window"
        );
        assert_eq!(
            (window.unknown_charges, window.unknown_bytes),
            (1, 16),
            "legacy evidence cannot certify a charge"
        );
        assert!(
            tasks[&1].is_complete(),
            "known successful charge has no gap"
        );
        assert!(
            !tasks[&2].is_complete(),
            "deferred task remains a lower bound"
        );
        assert!(!tasks[&3].is_complete(), "legacy task remains unknown");
        assert_eq!(
            tasks.len(),
            3,
            "unattributed charges must not invent a task ID"
        );
        assert!(
            !window.is_complete(),
            "window must expose every kind of gap"
        );
    }
}
