//! Formatting of the observed capture window, rather than process uptime.

pub(super) fn compute_system_uptime(
    allocations: &[crate::capture::types::AllocationInfo],
) -> String {
    if allocations.is_empty() {
        return "00:00:00".to_string();
    }
    let first = allocations
        .iter()
        .map(|a| a.timestamp_alloc)
        .min()
        .unwrap_or(0);
    let last = allocations
        .iter()
        .map(|a| a.timestamp_alloc)
        .max()
        .unwrap_or(0);
    format_duration(last.saturating_sub(first))
}

fn format_duration(duration_ns: u64) -> String {
    let total_secs = duration_ns / 1_000_000_000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else if total_secs >= 60 {
        format!("{}:{:02}", m, s)
    } else if total_secs > 0 {
        format!("{}:{:02}s", m, s)
    } else {
        // Zero-pad milliseconds: 22ms must display as 0.022s, not 0.22s.
        format!("0.{:03}s", duration_ns / 1_000_000)
    }
}

#[cfg(test)]
mod tests {
    use super::format_duration;

    /// Objective: Unit formatting cannot exaggerate short capture windows.
    /// Invariants: Milliseconds retain scale at leading-zero and second boundaries.
    #[test]
    fn duration_units_and_boundaries() {
        for (milliseconds, expected) in [
            (0, "0.000s"),
            (1, "0.001s"),
            (22, "0.022s"),
            (999, "0.999s"),
            (1000, "0:01s"),
            (60000, "1:00"),
            (3600000, "1:00:00"),
        ] {
            assert_eq!(
                format_duration(milliseconds * 1_000_000),
                expected,
                "format {milliseconds}ms without changing its unit scale"
            );
        }
    }
}
