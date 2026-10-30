//! Release handling for the async tracker.
//!
//! Kept in its own module so `async_tracker.rs` stays within the project line
//! budget; it is an `impl` block for the same type and shares its private state.

use super::*;

impl AsyncTracker {
    /// Track a deallocation associated with a task.
    ///
    /// Closes the *oldest* live record at the address: a late release must not
    /// destroy a newer binding created after the address was reused. Callers that know
    /// which binding they are releasing should use
    /// [`Self::track_deallocation_matching`].
    pub fn track_deallocation(&self, ptr: usize) {
        self.close_allocation(ptr, None, None);
    }

    /// Track a deallocation for a specific address binding.
    ///
    /// `size` and `generation` are optional filters; providing them lets a release that
    /// arrives late — after the address was reused — close the binding it actually
    /// refers to instead of the newest one.
    pub fn track_deallocation_matching(
        &self,
        ptr: usize,
        size: Option<usize>,
        generation: Option<u64>,
    ) {
        self.close_allocation(ptr, size, generation);
    }

    fn close_allocation(&self, ptr: usize, size: Option<usize>, generation: Option<u64>) {
        let closed = {
            let Ok(mut allocations) = self.allocations.lock() else {
                tracing::error!("Failed to acquire allocations lock during track_deallocation");
                return;
            };
            let Some(entries) = allocations.get_mut(&ptr) else {
                return;
            };

            // When the caller names a generation or size, only that binding may be
            // closed: falling back to "the oldest entry" would destroy a newer binding
            // (the exact bug this filter exists to prevent). Without a filter the
            // release closes the oldest binding, the best available guess.
            let index = match (generation, size) {
                (Some(generation), _) => entries.iter().position(|a| a.generation == generation),
                (None, Some(size)) => entries.iter().position(|a| a.size == size),
                (None, None) => Some(0),
            };
            let Some(index) = index else {
                tracing::debug!(
                    "unmatched async release: ptr={:#x}, size={:?}, generation={:?}",
                    ptr,
                    size,
                    generation
                );
                return;
            };

            let closed = entries.remove(index);
            if entries.is_empty() {
                allocations.remove(&ptr);
            }
            closed
        };

        if closed.task_id != 0 {
            if let Ok(mut profiles) = self.profiles.lock() {
                if let Some(profile) = profiles.get_mut(&closed.task_id) {
                    profile.record_deallocation(closed.size as u64);
                }
            } else {
                tracing::error!("Failed to acquire profiles lock during track_deallocation");
            }
        }

        if closed.size > 0 {
            if let Ok(mut stats) = self.stats.lock() {
                stats.active_memory = stats.active_memory.saturating_sub(closed.size);
                stats.total_deallocations += 1;
                stats.total_deallocated += closed.size as u64;
            } else {
                tracing::error!("Failed to acquire stats lock during track_deallocation");
            }
        }
    }
}
