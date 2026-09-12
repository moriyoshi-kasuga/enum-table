#[proc_macro_derive(Enumerable)]
pub fn derive_enumerable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_enumerable_internal(syn::parse_macro_input!(input as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_enumerable_internal(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let syn::Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input,
            "Enumerable can only be derived for enums",
        ));
    };

    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Enumerable cannot be derived for generic enums",
        ));
    }

    if repr_align(&input.attrs)?.is_some() {
        return Err(syn::Error::new_spanned(
            &input,
            "Enumerable cannot be derived for enums with `#[repr(align(N))]`: alignment \
             padding could be read as uninitialized memory by this crate's byte-level \
             comparisons",
        ));
    }

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
        .collect::<syn::Result<Vec<_>>>()?;

    let ident = &input.ident;
    let expanded = quote::quote! {
        // SAFETY: `#variant_idents` covers every variant of `#ident` exactly once;
        // unit variants and the absence of `#[repr(align(N))]` (checked above) rule
        // out padding; and `sort_variants` sorts `VARIANTS` by unsigned bit-pattern.
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

fn repr_align(attrs: &[syn::Attribute]) -> syn::Result<Option<usize>> {
    let mut align = None::<usize>;
    for attr in attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("align") {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let lit: syn::LitInt = content.parse()?;
                    let n: usize = lit.base10_parse()?;
                    align = Some(n);
                }
                Ok(())
            })?;
        }
    }
    Ok(align)
}
