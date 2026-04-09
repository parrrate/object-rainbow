use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Data, DeriveInput, Error, GenericParam, Generics, Index, parse_macro_input, parse_quote,
    spanned::Spanned,
};

#[proc_macro_derive(ToOutput)]
pub fn derive_to_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_constraint_bounds(input.generics);
    let to_output = gen_to_output(input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ToOutput for #name #ty_generics #where_clause {
            fn to_output(&self, output: &mut dyn Output) {
                #to_output
            }
        }
    };
    TokenStream::from(output)
}

fn add_constraint_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(parse_quote!(::object_rainbow::ToOutput));
        }
    }
    generics
}

fn gen_to_output(data: Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match data.fields {
            syn::Fields::Named(fields) => {
                let to_output = fields.named.into_iter().map(|f| {
                    let i = f.ident.unwrap();
                    quote_spanned! { f.ty.span() =>
                        self.#i.to_output(output)
                    }
                });
                quote! {
                    #(#to_output);*
                }
            }
            syn::Fields::Unnamed(fields) => {
                let to_output = fields.unnamed.into_iter().enumerate().map(|(i, f)| {
                    let i: Index = Index {
                        index: i.try_into().unwrap(),
                        span: f.span(),
                    };
                    quote_spanned! { f.ty.span() =>
                        self.#i.to_output(output)
                    }
                });
                quote! {
                    #(#to_output);*
                }
            }
            syn::Fields::Unit => quote! {},
        },
        Data::Enum(data) => {
            Error::new_spanned(data.enum_token, "`enum`s are not supported").to_compile_error()
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}
