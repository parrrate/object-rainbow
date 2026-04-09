use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Data, DeriveInput, Error, GenericParam, Generics, Index, parse_macro_input, parse_quote,
    parse_quote_spanned, spanned::Spanned,
};

#[proc_macro_derive(ToOutput)]
pub fn derive_to_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = bounds_to_output(input.generics);
    let to_output = gen_to_output(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ToOutput for #name #ty_generics #where_clause {
            fn to_output(&self, output: &mut dyn ::object_rainbow::Output) {
                #to_output
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_to_output(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(parse_quote!(::object_rainbow::ToOutput));
        }
    }
    generics
}

fn gen_to_output(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let to_output = fields.named.iter().map(|f| {
                    let i = f.ident.as_ref().unwrap();
                    quote_spanned! { f.ty.span() =>
                        self.#i.to_output(output)
                    }
                });
                quote! {
                    #(#to_output);*
                }
            }
            syn::Fields::Unnamed(fields) => {
                let to_output = fields.unnamed.iter().enumerate().map(|(i, f)| {
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

#[proc_macro_derive(Object)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_object(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let accept_points = gen_accept_points(&input.data);
    let parse = gen_parse(&input.data);
    let tags = gen_tags(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Object for #name #ty_generics #where_clause {
            fn accept_points(&self, visitor: &mut impl ::object_rainbow::PointVisitor) {
                #accept_points
            }

            fn parse(mut input: ::object_rainbow::Input) -> ::object_rainbow::Result<Self> {
                #parse
            }

            const TAGS: ::object_rainbow::Tags = #tags;
        }
    };
    TokenStream::from(output)
}

fn bounds_object(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            let last_at = data.fields.len().checked_sub(1).unwrap_or_default();
            for (i, f) in data.fields.iter().enumerate() {
                let last = i == last_at;
                let ty = &f.ty;
                let tr = if last {
                    quote!(::object_rainbow::Object)
                } else {
                    quote!(::object_rainbow::Inline)
                };
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: #tr
                    });
            }
        }
        Data::Enum(data) => {
            return Err(Error::new_spanned(
                data.enum_token,
                "`enum`s are not supported",
            ));
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "`union`s are not supported",
            ));
        }
    }
    Ok(generics)
}

fn gen_accept_points(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let accept_points = fields.named.iter().map(|f| {
                    let i = f.ident.as_ref().unwrap();
                    quote_spanned! { f.ty.span() =>
                        self.#i.accept_points(visitor)
                    }
                });
                quote! {
                    #(#accept_points);*
                }
            }
            syn::Fields::Unnamed(fields) => {
                let accept_points = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let i: Index = Index {
                        index: i.try_into().unwrap(),
                        span: f.span(),
                    };
                    quote_spanned! { f.ty.span() =>
                        self.#i.accept_points(visitor)
                    }
                });
                quote! {
                    #(#accept_points);*
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

fn gen_parse(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let last_at = fields.named.len().checked_sub(1).unwrap();
                let parse = fields.named.iter().enumerate().map(|(i, f)| {
                    let last = i == last_at;
                    let i = f.ident.as_ref().unwrap();
                    let method = if last {
                        quote!(parse)
                    } else {
                        quote!(parse_inline)
                    };
                    quote_spanned! { f.ty.span() =>
                        #i: input.#method()?
                    }
                });
                quote! {
                    Ok(Self { #(#parse),* })
                }
            }
            syn::Fields::Unnamed(fields) => {
                let last_at = fields.unnamed.len().checked_sub(1).unwrap();
                let parse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let last = i == last_at;
                    let method = if last {
                        quote!(parse)
                    } else {
                        quote!(parse_inline)
                    };
                    quote_spanned! { f.ty.span() =>
                        input.#method()?
                    }
                });
                quote! {
                    Ok(Self(#(#parse),*))
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

fn gen_tags(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let tags = data.fields.iter().map(|f| {
                let ty = &f.ty;
                quote! { &#ty::TAGS }
            });
            quote! {
                ::object_rainbow::Tags(&[], &[#(#tags),*])
            }
        }
        Data::Enum(data) => {
            Error::new_spanned(data.enum_token, "`enum`s are not supported").into_compile_error()
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").into_compile_error()
        }
    }
}
