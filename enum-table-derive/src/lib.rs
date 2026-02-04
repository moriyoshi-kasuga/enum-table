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
    let Data::Enum(data_enum) = input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "Enumable can only be derived for enums",
        ));
    };

    let variant_idents = data_enum
        .variants
        .iter()
        .map(|v| {
            if !matches!(v.fields, syn::Fields::Unit) {
                return Err(syn::Error::new_spanned(
                    &v.fields,
                    "Enumable can only be derived for unit variants",
                ));
            }
            Ok(&v.ident)
        })
        .collect::<Result<Vec<_>>>()?;

    let ident = &input.ident;
    let expanded = quote! {
        impl enum_table::Enumable for #ident {
            const VARIANTS: &'static [#ident] = &enum_table::__private::sort_variants([#(Self::#variant_idents),*]);

            fn variant_index(&self) -> usize {
                match *self {
                    #(
                        Self::#variant_idents => const { enum_table::__private::variant_index_of(&#ident::#variant_idents, <#ident as enum_table::Enumable>::VARIANTS) },
                    )*
                }
            }
        }
    };

    Ok(expanded)
}
