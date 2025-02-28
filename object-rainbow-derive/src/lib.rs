use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Data, DeriveInput, Error, Generics, Index, parse_macro_input, parse_quote_spanned,
    spanned::Spanned,
};

#[proc_macro_derive(ToOutput)]
pub fn derive_to_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_to_output(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
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

fn bounds_to_output(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::ToOutput
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

fn bounds_topological(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::Topological
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

#[proc_macro_derive(Topological)]
pub fn derive_topological(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_topological(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let accept_points = gen_accept_points(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Topological for #name #ty_generics #where_clause {
            fn accept_points(&self, visitor: &mut impl ::object_rainbow::PointVisitor) {
                #accept_points
            }
        }
    };
    TokenStream::from(output)
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

#[proc_macro_derive(Object)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_object(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let parse = gen_parse(&input.data);
    let tags = gen_tags(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Object for #name #ty_generics #where_clause {
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
            syn::Fields::Unit => quote! {
                Ok(Self)
            },
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
                quote! { &<#ty as ::object_rainbow::Object>::TAGS }
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

#[proc_macro_derive(Inline)]
pub fn derive_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_inline(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let parse_inline = gen_parse_inline(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Inline for #name #ty_generics #where_clause {
            fn parse_inline(input: &mut ::object_rainbow::Input) -> ::object_rainbow::Result<Self> {
                #parse_inline
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_inline(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::Inline
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

fn gen_parse_inline(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                let parse = fields.named.iter().map(|f| {
                    let i = f.ident.as_ref().unwrap();
                    quote_spanned! { f.ty.span() =>
                        #i: input.parse_inline()?
                    }
                });
                quote! {
                    Ok(Self { #(#parse),* })
                }
            }
            syn::Fields::Unnamed(fields) => {
                let parse = fields.unnamed.iter().map(|f| {
                    quote_spanned! { f.ty.span() =>
                        input.parse_inline()?
                    }
                });
                quote! {
                    Ok(Self(#(#parse),*))
                }
            }
            syn::Fields::Unit => quote! {
                Ok(Self)
            },
        },
        Data::Enum(data) => {
            Error::new_spanned(data.enum_token, "`enum`s are not supported").to_compile_error()
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

#[proc_macro_derive(ReflessObject)]
pub fn derive_refless_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_refless_object(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let parse = gen_parse(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ReflessObject for #name #ty_generics #where_clause {
            fn parse(mut input: ::object_rainbow::ReflessInput) -> ::object_rainbow::Result<Self> {
                #parse
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_refless_object(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            let last_at = data.fields.len().checked_sub(1).unwrap_or_default();
            for (i, f) in data.fields.iter().enumerate() {
                let last = i == last_at;
                let ty = &f.ty;
                let tr = if last {
                    quote!(::object_rainbow::ReflessObject)
                } else {
                    quote!(::object_rainbow::ReflessInline)
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

#[proc_macro_derive(ReflessInline)]
pub fn derive_refless_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_refless_inline(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let parse_inline = gen_parse_inline(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ReflessInline for #name #ty_generics #where_clause {
            fn parse_inline(input: &mut ::object_rainbow::ReflessInput) -> ::object_rainbow::Result<Self> {
                #parse_inline
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_refless_inline(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::ReflessInline
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

#[proc_macro_derive(Size)]
pub fn derive_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_size(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let size = gen_size(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Size for #name #ty_generics #where_clause {
            const SIZE: usize = #size;
        }
    };
    TokenStream::from(output)
}

fn bounds_size(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::Size
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

fn gen_size(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            if data.fields.is_empty() {
                return quote! {0};
            }
            let size = data.fields.iter().map(|f| {
                let ty = &f.ty;
                quote! { <#ty as ::object_rainbow::Size>::SIZE }
            });
            quote! {
                #(#size)+*
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
