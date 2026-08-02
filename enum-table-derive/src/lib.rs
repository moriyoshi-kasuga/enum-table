use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::Result;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Enumerable)]
pub fn derive_enumerable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_enumerable_internal(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_enumerable_internal(input: DeriveInput) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Enumerable cannot be derived for generic enums",
        ));
    }

    if repr_has_align(&input.attrs)? {
        return Err(syn::Error::new_spanned(
            &input,
            "Enumerable cannot be derived for enums with `#[repr(align(N))]`: an alignment \
             larger than the discriminant's natural size introduces trailing padding bytes, \
             which would make this crate's byte-level comparisons read uninitialized memory",
        ));
    }

    let Data::Enum(data_enum) = input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "Enumerable can only be derived for enums",
        ));
    };

    let variant_idents = data_enum
        .variants
        .iter()
        .map(|v| {
            if !matches!(v.fields, syn::Fields::Unit) {
                return Err(syn::Error::new_spanned(
                    &v.fields,
                    "Enumerable can only be derived for unit variants",
                ));
            }
            Ok(&v.ident)
        })
        .collect::<Result<Vec<_>>>()?;

    let ident = &input.ident;
    let expanded = quote! {
        // SAFETY: `#variant_idents` lists every variant of `#ident` exactly
        // once, this enum has no fields (checked above) and therefore no
        // padding bytes, and `sort_variants` produces a `VARIANTS` array
        // sorted by the unsigned bit-pattern of each variant, as required by
        // `enum_table::Enumerable`'s safety contract.
        unsafe impl enum_table::Enumerable for #ident {
            const VARIANTS: &'static [#ident] = &unsafe {
                enum_table::__private::sort_variants([#(Self::#variant_idents),*])
            };

            fn variant_index(&self) -> usize {
                match *self {
                    #(
                        Self::#variant_idents => const {
                            // SAFETY: see the `unsafe impl` block above.
                            unsafe {
                                enum_table::__private::variant_index_of(&#ident::#variant_idents, <#ident as enum_table::Enumerable>::VARIANTS)
                            }
                        },
                    )*
                }
            }
        }
    };

    Ok(expanded)
}

/// Returns `true` if any `#[repr(...)]` attribute on `attrs` contains an `align(N)` item.
fn repr_has_align(attrs: &[syn::Attribute]) -> Result<bool> {
    for attr in attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }

        let mut has_align = false;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("align") {
                has_align = true;
            }
            // `align(N)` carries a parenthesized value that we must consume
            // ourselves, otherwise `parse_nested_meta` errors on the leftover
            // tokens; we only care whether `align` is present, not its value.
            if meta.input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in meta.input);
                let _ = content.parse::<proc_macro2::TokenStream>();
            }
            Ok(())
        })?;

        if has_align {
            return Ok(true);
        }
    }
    Ok(false)
}
