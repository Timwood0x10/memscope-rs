// rust_execution_visualizer_macros/src/lib.rs

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// Attribute macro to track a variable's heap allocation.
///
/// Usage:
/// ```rust
/// #[track_var]
/// let my_box = Box::new(42);
/// ```
/// This will be expanded to roughly:
/// ```rust
/// let my_box = Box::new(42);
/// crate::__internal_associate_var_with_alloc(&my_box, "my_box");
/// ```
use syn::{parse_macro_input, Ident};

#[proc_macro]
pub fn track_var(item: TokenStream) -> TokenStream {
    // Input is expected to be the variable identifier
    let var_ident = parse_macro_input!(item as Ident);
    let var_name_str = var_ident.to_string();

    // To get the type, we'd ideally inspect the variable's declaration or use type inference hints.
    // Proc macros operate on syntax, making direct type resolution complex without compiler internals.
    // A common approach is to require the type to be passed to the macro, or use a helper trait.
    // For this iteration, we'll create a placeholder for the type name.
    // A more advanced version might use `std::any::type_name_of_val` if the macro could be structured
    // to have access to the value at runtime, or require manual type annotation in the macro call.

    // Placeholder for type_name. In a real scenario, this would need a robust way to be determined.
    // let type_name_str = "UnknownType"; // Simplified placeholder

    // The `track_var!(my_variable)` macro will now rely on the `Trackable` trait
    // implemented in `procmacros.rs` to get the pointer and the `__internal_associate_var_with_alloc`
    // function in `procmacros.rs` (which should be moved or aliased to `crate::...` if it's in `types.rs`)
    // to perform the association. The type name will be passed from the `Trackable` impl or a new mechanism.

    // The original __internal_associate_var_with_alloc in procmacros.rs takes &T and var_name.
    // The one in types.rs takes ptr, size, var_name, type_name_str, fn_context_override.
    // We need to ensure we are calling the correct one or refactor.
    // Assuming the one in `procmacros.rs` is the intended target for the simple `track_var!(ident)` form.

    quote! {
        {
            let var = #var_ident;
            if let Some(ptr) = crate::Trackable::get_trackable_raw_ptr(&var) {
                let tracker = crate::get_global_tracker();
                if let Err(e) = tracker.associate_var(ptr, #var_name_str.to_string()) {
                    tracing::error!("Failed to track variable: {}", e);
                }
            }
            var
        }
    }.into()
}


// Helper to parse `var_name: Type` (not used in current track_var, but useful for other macros)
struct TrackVarInput {
    _var_ident: Ident,
    _colon_token: syn::Token![:],
    _var_type: syn::Type,
}

impl syn::parse::Parse for TrackVarInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(TrackVarInput {
            _var_ident: input.parse()?,
            _colon_token: input.parse()?,
            _var_type: input.parse()?,
        })
    }
}
