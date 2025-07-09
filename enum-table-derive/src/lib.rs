use proc_macro2::TokenStream;
use quote::quote;
use syn::Data;
use syn::Result;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Enumable)]
pub fn derive_enumable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_enumable_internal(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_enumable_internal(input: DeriveInput) -> Result<TokenStream> {
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Enumable can only be derived for enums",
            ));
        }
    };

    let variants = variants
        .iter()
        .map(|v| {
            if !matches!(v.fields, syn::Fields::Unit) {
                return Err(syn::Error::new_spanned(
                    v,
                    "Enumable can only be derived for unit variants",
                ));
            }
            Ok(&v.ident)
        })
        .collect::<Result<Vec<_>>>()?;

    let ident = &input.ident;
    let expanded = quote! {
        impl enum_table::Enumable for #ident {
            const VARIANTS: &'static [#ident] = &enum_table::__private::sort_variants([#(Self::#variants),*]);
        }
    };

    Ok(expanded)
}
