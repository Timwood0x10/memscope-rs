//! Procedural macros for memscope-rs memory tracking
//!
//! This crate provides the `#[derive(Trackable)]` macro for automatically
//! implementing the `Trackable` trait for user-defined types.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Derive macro for automatically implementing the `Trackable` trait.
///
/// This macro generates implementations for:
/// - `get_heap_ptr()`: Returns the struct's address for structs with heap allocations
/// - `get_type_name()`: Returns the type name as a string literal
/// - `get_size_estimate()`: Calculates total size including internal allocations
/// - `get_internal_allocations()`: Lists all internal heap allocations
///
/// # Examples
///
/// ```rust
/// use memscope_rs::Trackable;
/// use memscope_derive::Trackable;
///
/// #[derive(Trackable)]
/// struct UserData {
///     name: String,
///     scores: Vec<i32>,
///     metadata: Box<HashMap<String, String>>,
/// }
/// ```
///
/// The macro handles:
/// - Structs with named fields
/// - Tuple structs
/// - Unit structs
/// - Enums with data
/// - Nested types that implement `Trackable`
#[proc_macro_derive(Trackable)]
pub fn derive_trackable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data_struct) => {
            let heap_ptr_impl = generate_heap_ptr_impl(&data_struct.fields);
            let size_estimate_impl = generate_size_estimate_impl(&data_struct.fields);
            let internal_allocations_impl = generate_internal_allocations_impl(&data_struct.fields);

            quote! {
                impl #impl_generics memscope_rs::Trackable for #name #ty_generics #where_clause {
                    fn get_heap_ptr(&self) -> Option<usize> {
                        #heap_ptr_impl
                    }

                    fn get_type_name(&self) -> &'static str {
                        stringify!(#name)
                    }

                    fn get_size_estimate(&self) -> usize {
                        #size_estimate_impl
                    }

                    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
                        #internal_allocations_impl
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let size_estimate_impl = generate_enum_size_estimate_impl(&data_enum.variants);
            let internal_allocations_impl =
                generate_enum_internal_allocations_impl(&data_enum.variants);

            quote! {
                impl #impl_generics memscope_rs::Trackable for #name #ty_generics #where_clause {
                    fn get_heap_ptr(&self) -> Option<usize> {
                        // For enums, use the enum instance address
                        Some(self as *const _ as usize)
                    }

                    fn get_type_name(&self) -> &'static str {
                        stringify!(#name)
                    }

                    fn get_size_estimate(&self) -> usize {
                        #size_estimate_impl
                    }

                    fn get_internal_allocations(&self, var_name: &str) -> Vec<(usize, String)> {
                        #internal_allocations_impl
                    }
                }
            }
        }
        Data::Union(_) => {
            // Unions are not supported for safety reasons
            return syn::Error::new_spanned(
                &input,
                "Trackable cannot be derived for unions due to safety concerns",
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(expanded)
}

/// Generate the `get_heap_ptr` implementation for structs
fn generate_heap_ptr_impl(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(_) | Fields::Unnamed(_) => {
            // Check if any field has heap allocations
            let has_heap_fields = has_potential_heap_allocations(fields);

            if has_heap_fields {
                quote! {
                    // Use the struct's address as the primary identifier
                    Some(self as *const _ as usize)
                }
            } else {
                quote! {
                    // No heap allocations detected
                    None
                }
            }
        }
        Fields::Unit => {
            quote! {
                // Unit structs have no heap allocations
                None
            }
        }
    }
}

/// Generate the `get_size_estimate` implementation
fn generate_size_estimate_impl(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields_named) => {
            let field_sizes = fields_named.named.iter().map(|field| {
                let field_name = &field.ident;
                quote! {
                    total_size += memscope_rs::Trackable::get_size_estimate(&self.#field_name);
                }
            });

            quote! {
                let mut total_size = std::mem::size_of::<Self>();
                #(#field_sizes)*
                total_size
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_sizes = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                quote! {
                    total_size += memscope_rs::Trackable::get_size_estimate(&self.#index);
                }
            });

            quote! {
                let mut total_size = std::mem::size_of::<Self>();
                #(#field_sizes)*
                total_size
            }
        }
        Fields::Unit => {
            quote! {
                std::mem::size_of::<Self>()
            }
        }
    }
}

/// Generate the `get_internal_allocations` implementation
fn generate_internal_allocations_impl(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields_named) => {
            let field_allocations = fields_named.named.iter().map(|field| {
                let field_name = &field.ident;
                let field_name_str = field_name.as_ref().unwrap().to_string();
                quote! {
                    if let Some(ptr) = memscope_rs::Trackable::get_heap_ptr(&self.#field_name) {
                        allocations.push((ptr, format!("{var_name}::{}", #field_name_str)));
                    }
                }
            });

            quote! {
                let mut allocations = Vec::new();
                #(#field_allocations)*
                allocations
            }
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_allocations = fields_unnamed.unnamed.iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                let index_str = i.to_string();
                quote! {
                    if let Some(ptr) = memscope_rs::Trackable::get_heap_ptr(&self.#index) {
                        allocations.push((ptr, format!("{var_name}::{}", #index_str)));
                    }
                }
            });

            quote! {
                let mut allocations = Vec::new();
                #(#field_allocations)*
                allocations
            }
        }
        Fields::Unit => {
            quote! {
                Vec::new()
            }
        }
    }
}

/// Generate size estimate for enums
fn generate_enum_size_estimate_impl(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
) -> proc_macro2::TokenStream {
    let variant_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_sizes = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    quote! {
                        total_size += memscope_rs::Trackable::get_size_estimate(#field_name);
                    }
                });

                quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        let mut total_size = std::mem::size_of::<Self>();
                        #(#field_sizes)*
                        total_size
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_patterns: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| {
                        syn::Ident::new(&format!("field_{i}"), proc_macro2::Span::call_site())
                    })
                    .collect();
                let field_sizes = field_patterns.iter().map(|field_name| {
                    quote! {
                        total_size += memscope_rs::Trackable::get_size_estimate(#field_name);
                    }
                });

                quote! {
                    Self::#variant_name(#(#field_patterns),*) => {
                        let mut total_size = std::mem::size_of::<Self>();
                        #(#field_sizes)*
                        total_size
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => std::mem::size_of::<Self>()
                }
            }
        }
    });

    quote! {
        match self {
            #(#variant_arms),*
        }
    }
}

/// Generate internal allocations for enums
fn generate_enum_internal_allocations_impl(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
) -> proc_macro2::TokenStream {
    let variant_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_name_str = variant_name.to_string();
        match &variant.fields {
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_allocations = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    let field_name_str = field_name.as_ref().unwrap().to_string();
                    quote! {
                        if let Some(ptr) = memscope_rs::Trackable::get_heap_ptr(#field_name) {
                            allocations.push((ptr, format!("{var_name}::{}::{}", #variant_name_str, #field_name_str)));
                        }
                    }
                });
                quote! {
                    Self::#variant_name { #(#field_names),* } => {
                        let mut allocations = Vec::new();
                        #(#field_allocations)*
                        allocations
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_patterns: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("field_{i}"), proc_macro2::Span::call_site()))
                    .collect();
                let field_allocations = field_patterns.iter().enumerate().map(|(i, field_name)| {
                    quote! {
                        if let Some(ptr) = memscope_rs::Trackable::get_heap_ptr(#field_name) {
                            allocations.push((ptr, format!("{var_name}::{}::{}", #variant_name_str, #i)));
                        }
                    }
                });
                quote! {
                    Self::#variant_name(#(#field_patterns),*) => {
                        let mut allocations = Vec::new();
                        #(#field_allocations)*
                        allocations
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    Self::#variant_name => Vec::new()
                }
            }
        }
    });

    quote! {
        match self {
            #(#variant_arms),*
        }
    }
}

/// Check if fields potentially contain heap allocations
fn has_potential_heap_allocations(fields: &Fields) -> bool {
    match fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .any(|field| is_potentially_heap_allocated(&field.ty)),
        Fields::Unnamed(fields_unnamed) => fields_unnamed
            .unnamed
            .iter()
            .any(|field| is_potentially_heap_allocated(&field.ty)),
        Fields::Unit => false,
    }
}

/// Check if a type is potentially heap-allocated
fn is_potentially_heap_allocated(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                matches!(
                    type_name.as_str(),
                    "String"
                        | "Vec"
                        | "HashMap"
                        | "BTreeMap"
                        | "HashSet"
                        | "BTreeSet"
                        | "VecDeque"
                        | "LinkedList"
                        | "BinaryHeap"
                        | "Box"
                        | "Rc"
                        | "Arc"
                )
            } else {
                false
            }
        }
        _ => false,
    }
}
