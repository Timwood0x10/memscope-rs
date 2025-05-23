// src/procmacros.rs

// This function is called by the `track_var!` macro.
// It needs to be `pub` so it's accessible from `main.rs` (if called directly there)
// and from the macro expansion (which effectively happens in the context of `main.rs`).
use crate::types::associate_variable_with_ptr;

/// Trait for types whose heap allocations can be tracked.
pub trait Trackable {
    /// Returns the raw pointer to the primary heap-allocated data, if applicable.
    /// Returns `None` if the object is not on the heap or not in a trackable state (e.g., empty Vec).
    fn get_trackable_raw_ptr(&self) -> Option<usize>;

    /// Returns a string representation of the type.
    fn get_type_name_str(&self) -> String;
}

impl<T> Trackable for Box<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        Some(self.as_ref() as *const T as usize)
    }
    fn get_type_name_str(&self) -> String {
        std::any::type_name::<Box<T>>().to_string()
    }
}

impl<T> Trackable for Vec<T> {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.capacity() == 0 {
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }

    fn get_type_name_str(&self) -> String {
        std::any::type_name::<Vec<T>>().to_string()
    }

}

impl Trackable for String {
    fn get_trackable_raw_ptr(&self) -> Option<usize> {
        if self.capacity() == 0 {
            None
        } else {
            Some(self.as_ptr() as usize)
        }
    }

    fn get_type_name_str(&self) -> String {
        std::any::type_name::<String>().to_string()
    }

}

/// Internal helper function called by the `track_var!` macro expansion.
#[doc(hidden)]
pub fn __internal_associate_var_with_alloc<T: Trackable + ?Sized>(variable: &T, var_name: &str) {
    if let Some(ptr_val) = variable.get_trackable_raw_ptr() {
        if ptr_val != 0 {
            let type_name_str = variable.get_type_name_str();
            // Now we need to call a version of associate_variable_with_ptr that also takes type_name_str
            // Or, call the function from types.rs if that's the intended one.
            // Let's assume `associate_variable_with_ptr` in `main.rs` is modified or a new function is made.
            // For now, we'll stick to the existing `associate_variable_with_ptr` from `main.rs`
            // Call the updated function in main.rs, which now accepts type_name_str.
            let _ = associate_variable_with_ptr(ptr_val, var_name, &type_name_str);
        }
    }
}
