use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Generics, Ident, LitStr, parse::Parse, parse_macro_input,
    parse_quote, parse_quote_spanned, spanned::Spanned, token::Comma,
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
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::ToOutput
                        },
                    );
                }
            }
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

fn fields_to_output(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let let_self = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            let to_output = let_self.clone().zip(fields.named.iter()).map(|(i, f)| {
                quote_spanned! { f.ty.span() =>
                    #i.to_output(output)
                }
            });
            quote! {
                { #(#let_self),* } => {
                    #(#to_output);*
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let let_self = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| Ident::new(&format!("field{i}"), f.ty.span()));
            let to_output = let_self.clone().zip(fields.unnamed.iter()).map(|(i, f)| {
                quote_spanned! { f.ty.span() =>
                    #i.to_output(output)
                }
            });
            quote! {
                (#(#let_self),*) => {
                    #(#to_output);*
                }
            }
        }
        syn::Fields::Unit => quote! {
            => {}
        },
    }
}

fn gen_to_output(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let arm = fields_to_output(&data.fields);
            quote! {
                match self {
                    Self #arm
                }
            }
        }
        Data::Enum(data) => {
            let to_output = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_to_output(&v.fields);
                quote! { Self::#ident #arm }
            });
            quote! {
                let kind = ::object_rainbow::Enum::kind(self);
                let tag = ::object_rainbow::enumkind::EnumKind::to_tag(kind);
                tag.to_output(output);
                match self {
                    #(#to_output)*
                }
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
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
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Topological
                        },
                    );
                }
            }
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

fn fields_accept_points(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let let_self = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            let accept_points = let_self.clone().zip(fields.named.iter()).map(|(i, f)| {
                quote_spanned! { f.ty.span() =>
                    #i.accept_points(visitor)
                }
            });
            quote! {
                { #(#let_self),* } => {
                    #(#accept_points);*
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let let_self = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| Ident::new(&format!("field{i}"), f.ty.span()));
            let accept_points = let_self.clone().zip(fields.unnamed.iter()).map(|(i, f)| {
                quote_spanned! { f.ty.span() =>
                    #i.accept_points(visitor)
                }
            });
            quote! {
                (#(#let_self),*) => {
                    #(#accept_points);*
                }
            }
        }
        syn::Fields::Unit => quote! {
            => {}
        },
    }
}

fn gen_accept_points(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let arm = fields_accept_points(&data.fields);
            quote! {
                match self {
                    Self #arm
                }
            }
        }
        Data::Enum(data) => {
            let to_output = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_accept_points(&v.fields);
                quote! { Self::#ident #arm }
            });
            quote! {
                let kind = ::object_rainbow::Enum::kind(self);
                let tag = ::object_rainbow::enumkind::EnumKind::to_tag(kind);
                tag.accept_points(visitor);
                match self {
                    #(#to_output)*
                }
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

#[proc_macro_derive(Tagged, attributes(tags))]
pub fn derive_tagged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mut errors = Vec::new();
    let generics = match bounds_tagged(input.generics, &input.data, &mut errors) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let tags = gen_tags(&input.data, &input.attrs, &mut errors);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let errors = errors.into_iter().map(|e| e.into_compile_error());
    let output = quote! {
        #(#errors)*

        impl #impl_generics ::object_rainbow::Tagged for #name #ty_generics #where_clause {
            const TAGS: ::object_rainbow::Tags = #tags;
        }
    };
    TokenStream::from(output)
}

struct FieldTagArgs {
    skip: bool,
}

impl Parse for FieldTagArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut skip = false;
        while !input.is_empty() {
            let ident = input.parse::<Ident>()?;
            if ident.to_string().as_str() != "skip" {
                return Err(Error::new(ident.span(), "expected: skip"));
            }
            skip = true;
            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }
        Ok(Self { skip })
    }
}

fn bounds_tagged(
    mut generics: Generics,
    data: &Data,
    errors: &mut Vec<Error>,
) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let mut skip = false;
                for attr in &f.attrs {
                    match attr.parse_args::<FieldTagArgs>() {
                        Ok(args) => skip |= args.skip,
                        Err(e) => errors.push(e),
                    }
                }
                if !skip {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Tagged
                        },
                    );
                }
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

struct StructTagArgs {
    tags: Vec<LitStr>,
}

impl Parse for StructTagArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tags = Vec::new();
        while !input.is_empty() {
            let tag = input.parse::<LitStr>()?;
            tags.push(tag);
            if !input.is_empty() {
                input.parse::<Comma>()?;
            }
        }
        Ok(Self { tags })
    }
}

fn gen_tags(data: &Data, attrs: &[Attribute], errors: &mut Vec<Error>) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let mut tags = Vec::new();
            for attr in attrs {
                match attr.parse_args::<StructTagArgs>() {
                    Ok(mut args) => tags.append(&mut args.tags),
                    Err(e) => errors.push(e),
                }
            }
            let nested = data
                .fields
                .iter()
                .filter_map(|f| {
                    let mut skip = false;
                    for attr in &f.attrs {
                        skip |= attr.parse_args::<FieldTagArgs>().ok()?.skip;
                    }
                    let ty = &f.ty;
                    (!skip).then_some(quote! { <#ty as ::object_rainbow::Tagged>::TAGS })
                })
                .collect::<Vec<_>>();
            if nested.len() == 1 && tags.is_empty() {
                let nested = nested.into_iter().next().unwrap();
                quote! {
                    #nested
                }
            } else {
                quote! {
                    ::object_rainbow::Tags(&[#(#tags),*], &[#(&#nested),*])
                }
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

#[proc_macro_derive(Object)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_object(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Object for #name #ty_generics #where_clause {}
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

#[proc_macro_derive(Inline)]
pub fn derive_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_inline(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Inline for #name #ty_generics #where_clause {}
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

#[proc_macro_derive(ReflessObject)]
pub fn derive_refless_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_refless_object(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ReflessObject for #name #ty_generics #where_clause {}
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
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ReflessInline for #name #ty_generics #where_clause {}
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
    let size_arr = gen_size_arr(&input.data);
    let size = gen_size(&input.data);
    let generics = match bounds_size(input.generics, &input.data, &size_arr) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let output = quote! {
        const _: () = {
            use ::typenum::tarr;

            impl #impl_generics ::object_rainbow::Size for #name #ty_generics #where_clause {
                const SIZE: usize = #size;

                type Size = <#size_arr as ::typenum::FoldAdd>::Output;
            }
        };
    };
    TokenStream::from(output)
}

fn bounds_size(
    mut generics: Generics,
    data: &Data,
    size_arr: &proc_macro2::TokenStream,
) -> syn::Result<Generics> {
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
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Size
                        },
                    );
                }
            }
            for v in data.variants.iter().skip(1) {
                let arr = fields_size_arr(&v.fields);
                generics.make_where_clause().predicates.push(parse_quote!(
                    #arr: ::typenum::FoldAdd<Output = <#size_arr as ::typenum::FoldAdd>::Output>
                ));
            }
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "`union`s are not supported",
            ));
        }
    }
    generics.make_where_clause().predicates.push(parse_quote!(
        #size_arr: ::typenum::FoldAdd<Output: ::typenum::Unsigned>
    ));
    Ok(generics)
}

fn fields_size_arr(fields: &syn::Fields) -> proc_macro2::TokenStream {
    if fields.is_empty() {
        return quote! {
            tarr![::typenum::consts::U0]
        };
    }
    let size_arr = fields.iter().map(|f| {
        let ty = &f.ty;
        quote! { <#ty as ::object_rainbow::Size>::Size }
    });
    quote! {
        tarr![#(#size_arr),*]
    }
}

fn gen_size_arr(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => fields_size_arr(&data.fields),
        Data::Enum(data) => {
            if let Some(v) = data.variants.first() {
                fields_size_arr(&v.fields)
            } else {
                Error::new_spanned(data.enum_token, "empty `enum`s are not supported")
                    .into_compile_error()
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").into_compile_error()
        }
    }
}

fn fields_size(fields: &syn::Fields) -> proc_macro2::TokenStream {
    if fields.is_empty() {
        return quote! {0};
    }
    let size = fields.iter().map(|f| {
        let ty = &f.ty;
        quote! { <#ty as ::object_rainbow::Size>::SIZE }
    });
    quote! {
        #(#size)+*
    }
}

fn gen_size(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => fields_size(&data.fields),
        Data::Enum(data) => {
            if let Some(v) = data.variants.first() {
                fields_size(&v.fields)
            } else {
                Error::new_spanned(data.enum_token, "empty `enum`s are not supported")
                    .into_compile_error()
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").into_compile_error()
        }
    }
}

#[proc_macro_derive(Parse)]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_parse(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let parse = gen_parse(&input.data);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Parse<__I> for #name #ty_generics #where_clause {
            fn parse(mut input: __I) -> ::object_rainbow::Result<Self> {
                #parse
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_parse(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            let last_at = data.fields.len().checked_sub(1).unwrap_or_default();
            for (i, f) in data.fields.iter().enumerate() {
                let last = i == last_at;
                let ty = &f.ty;
                let tr = if last {
                    quote!(::object_rainbow::Parse<__I>)
                } else {
                    quote!(::object_rainbow::ParseInline<__I>)
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
            for v in data.variants.iter() {
                let last_at = v.fields.len().checked_sub(1).unwrap_or_default();
                for (i, f) in v.fields.iter().enumerate() {
                    let last = i == last_at;
                    let ty = &f.ty;
                    let tr = if last {
                        quote!(::object_rainbow::Parse<__I>)
                    } else {
                        quote!(::object_rainbow::ParseInline<__I>)
                    };
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: #tr
                        },
                    );
                }
            }
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "`union`s are not supported",
            ));
        }
    }
    generics
        .params
        .push(parse_quote!(__I: ::object_rainbow::ParseInput));
    Ok(generics)
}

fn gen_parse(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let arm = fields_parse(&data.fields);
            quote! { Ok(Self #arm)}
        }
        Data::Enum(data) => {
            let parse = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_parse(&v.fields);
                quote! {
                    <Self as ::object_rainbow::Enum>::Kind::#ident => Self::#ident #arm,
                }
            });
            quote! {
                Ok(match input.parse_inline()? {
                    #(#parse)*
                })
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

fn fields_parse(fields: &syn::Fields) -> proc_macro2::TokenStream {
    let last_at = fields.len().checked_sub(1).unwrap_or_default();
    match fields {
        syn::Fields::Named(fields) => {
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
            quote! { { #(#parse),* } }
        }
        syn::Fields::Unnamed(fields) => {
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
            quote! { (#(#parse),*) }
        }
        syn::Fields::Unit => quote! {},
    }
}

#[proc_macro_derive(ParseInline)]
pub fn derive_parse_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_parse_inline(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let parse_inline = gen_parse_inline(&input.data);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::ParseInline<__I> for #name #ty_generics #where_clause {
            fn parse_inline(input: &mut __I) -> ::object_rainbow::Result<Self> {
                #parse_inline
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_parse_inline(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::ParseInline<__I>
                    });
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::ParseInline<__I>
                        },
                    );
                }
            }
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "`union`s are not supported",
            ));
        }
    }
    generics
        .params
        .push(parse_quote!(__I: ::object_rainbow::ParseInput));
    Ok(generics)
}

fn fields_parse_inline(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let parse = fields.named.iter().map(|f| {
                let i = f.ident.as_ref().unwrap();
                quote_spanned! { f.ty.span() =>
                    #i: input.parse_inline()?
                }
            });
            quote! { { #(#parse),* } }
        }
        syn::Fields::Unnamed(fields) => {
            let parse = fields.unnamed.iter().map(|f| {
                quote_spanned! { f.ty.span() =>
                    input.parse_inline()?
                }
            });
            quote! { (#(#parse),*) }
        }
        syn::Fields::Unit => quote! {},
    }
}

fn gen_parse_inline(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let arm = fields_parse_inline(&data.fields);
            quote! { Ok(Self #arm) }
        }
        Data::Enum(data) => {
            let parse_inline = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_parse_inline(&v.fields);
                quote! {
                    <Self as ::object_rainbow::Enum>::Kind::#ident => Self::#ident #arm,
                }
            });
            quote! {
                Ok(match input.parse_inline()? {
                    #(#parse_inline)*
                })
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

#[proc_macro_derive(ParseAsInline)]
pub fn derive_parse_as_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_parse_as_inline(input.generics, &name) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        impl #impl_generics ::object_rainbow::Parse<__I> for #name #ty_generics #where_clause {
            fn parse(input: __I) -> ::object_rainbow::Result<Self> {
                ::object_rainbow::ParseInline::<__I>::parse_as_inline(input)
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_parse_as_inline(mut generics: Generics, name: &Ident) -> syn::Result<Generics> {
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote_spanned! { name.span() =>
            Self: ::object_rainbow::ParseInline::<__I>
        });
    generics
        .params
        .push(parse_quote!(__I: ::object_rainbow::ParseInput));
    Ok(generics)
}

#[proc_macro_derive(Enum)]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = input.generics;
    let variants = gen_variants(&input.data);
    let variant_count = gen_variant_count(&input.data);
    let to_tag = gen_to_tag(&input.data);
    let from_tag = gen_from_tag(&input.data);
    let kind = gen_kind(&input.data);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        const _: () = {
            use ::object_rainbow::enumkind::EnumKind;

            #[derive(Clone, Copy, ::object_rainbow::ParseAsInline)]
            enum __Kind {
                #variants
            }

            impl ::object_rainbow::enumkind::EnumKind for __Kind {
                type Tag = ::object_rainbow::enumkind::EnumTag<
                    ::object_rainbow::numeric::Le<u8>,
                    #variant_count,
                >;

                fn to_tag(self) -> Self::Tag {
                    #to_tag
                }

                fn from_tag(tag: Self::Tag) -> Self {
                    #from_tag
                }
            }

            impl<I: ::object_rainbow::ParseInput> ::object_rainbow::ParseInline<I> for __Kind {
                fn parse_inline(input: &mut I) -> ::object_rainbow::Result<Self> {
                    Ok(::object_rainbow::enumkind::EnumKind::from_tag(input.parse_inline()?))
                }
            }

            impl #impl_generics ::object_rainbow::Enum for #name #ty_generics #where_clause {
                type Kind = __Kind;

                fn kind(&self) -> Self::Kind {
                    #kind
                }
            }
        };
    };
    TokenStream::from(output)
}

fn gen_variants(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            Error::new_spanned(data.struct_token, "`struct`s are not supported").to_compile_error()
        }
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| &v.ident);
            quote! { #(#variants),* }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

fn gen_variant_count(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            Error::new_spanned(data.struct_token, "`struct`s are not supported").to_compile_error()
        }
        Data::Enum(data) => {
            let variant_count = data.variants.len();
            quote! { #variant_count }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

fn gen_to_tag(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            Error::new_spanned(data.struct_token, "`struct`s are not supported").to_compile_error()
        }
        Data::Enum(data) => {
            let to_tag = data.variants.iter().enumerate().map(|(i, v)| {
                let ident = &v.ident;
                quote_spanned! { ident.span() =>
                    Self::#ident => ::object_rainbow::enumkind::EnumTag::from_const::<#i>(),
                }
            });
            quote! {
                match self {
                    #(#to_tag)*
                }
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

fn gen_from_tag(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            Error::new_spanned(data.struct_token, "`struct`s are not supported").to_compile_error()
        }
        Data::Enum(data) => {
            let from_tag = data.variants.iter().enumerate().map(|(i, v)| {
                let ident = &v.ident;
                quote_spanned! { ident.span() =>
                    #i => Self::#ident,
                }
            });
            quote! {
                match (*tag).try_into().unwrap() {
                    #(#from_tag)*
                    _ => unreachable!(),
                }
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}

fn gen_kind(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            Error::new_spanned(data.struct_token, "`struct`s are not supported").to_compile_error()
        }
        Data::Enum(data) => {
            let variants = data.variants.iter().map(|v| {
                let ident = &v.ident;
                quote_spanned! { ident.span() =>
                    Self::#ident {..} => __Kind::#ident,
                }
            });
            quote! {
                match self {
                    #(#variants)*
                }
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
}
