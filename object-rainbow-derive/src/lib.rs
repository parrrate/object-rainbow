use std::collections::BTreeSet;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Error, Expr, GenericParam,
    Generics, Ident, LitStr, Path, Type, parse::Parse, parse_macro_input, parse_quote,
    parse_quote_spanned, spanned::Spanned, token::Comma,
};

fn expr_contains_generics(g: &BTreeSet<Ident>, expr: &Expr) -> bool {
    match expr {
        Expr::Path(expr) => path_contains_generics(g, &expr.path),
        _ => unimplemented!(),
    }
}

fn args_contains_generics(g: &BTreeSet<Ident>, args: &AngleBracketedGenericArguments) -> bool {
    args.args.iter().any(|arg| match arg {
        syn::GenericArgument::Type(ty) => type_contains_generics(g, ty),
        syn::GenericArgument::AssocType(ty) => {
            ty.generics
                .as_ref()
                .is_some_and(|args| args_contains_generics(g, args))
                || type_contains_generics(g, &ty.ty)
        }
        syn::GenericArgument::Const(expr) => expr_contains_generics(g, expr),
        syn::GenericArgument::AssocConst(expr) => {
            expr.generics
                .as_ref()
                .is_some_and(|args| args_contains_generics(g, args))
                || expr_contains_generics(g, &expr.value)
        }
        _ => false,
    })
}

fn path_contains_generics(g: &BTreeSet<Ident>, path: &Path) -> bool {
    path.segments.iter().any(|seg| match &seg.arguments {
        syn::PathArguments::None => seg.ident == "Self" && !g.is_empty() || g.contains(&seg.ident),
        syn::PathArguments::AngleBracketed(args) => {
            seg.ident == "Point" || args_contains_generics(g, args)
        }
        syn::PathArguments::Parenthesized(args) => {
            args.inputs.iter().any(|ty| type_contains_generics(g, ty))
                || match &args.output {
                    syn::ReturnType::Default => false,
                    syn::ReturnType::Type(_, ty) => type_contains_generics(g, ty),
                }
        }
    })
}

fn type_contains_generics(g: &BTreeSet<Ident>, ty: &Type) -> bool {
    match ty {
        Type::Array(ty) => type_contains_generics(g, &ty.elem),
        Type::BareFn(ty) => {
            ty.inputs.iter().any(|a| type_contains_generics(g, &a.ty))
                || match &ty.output {
                    syn::ReturnType::Default => false,
                    syn::ReturnType::Type(_, ty) => type_contains_generics(g, ty),
                }
        }
        Type::Group(ty) => type_contains_generics(g, &ty.elem),
        Type::Macro(_) => true,
        Type::Paren(ty) => type_contains_generics(g, &ty.elem),
        Type::Path(ty) => path_contains_generics(g, &ty.path),
        Type::Reference(ty) => type_contains_generics(g, &ty.elem),
        Type::Slice(ty) => type_contains_generics(g, &ty.elem),
        Type::Tuple(ty) => ty.elems.iter().any(|ty| type_contains_generics(g, ty)),
        _ => false,
    }
}

fn bounds_g(generics: &Generics) -> BTreeSet<Ident> {
    generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Lifetime(_) => None,
            GenericParam::Type(param) => Some(&param.ident),
            GenericParam::Const(param) => Some(&param.ident),
        })
        .cloned()
        .collect()
}

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
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::ToOutput for #name #ty_generics #where_clause {
            fn to_output(&self, output: &mut dyn ::object_rainbow::Output) {
                #to_output
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_to_output(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::ToOutput
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::ToOutput
                            },
                        );
                    }
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
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_topological(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let accept_points = gen_accept_points(&input.data);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::Topological<__E> for #name #ty_generics #where_clause {
            fn accept_points(&self, visitor: &mut impl ::object_rainbow::PointVisitor<__E>) {
                #accept_points
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_topological(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Topological<__E>
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::Topological<__E>
                            },
                        );
                    }
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
    generics.params.push(parse_quote!(__E: 'static));
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

        #[automatically_derived]
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
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let mut skip = false;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("tags") {
                        match attr.parse_args::<FieldTagArgs>() {
                            Ok(args) => skip |= args.skip,
                            Err(e) => errors.push(e),
                        }
                    }
                }
                if !skip {
                    let ty = &f.ty;
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::Tagged
                            },
                        );
                    }
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let mut skip = false;
                    for attr in &f.attrs {
                        if attr_str(attr).as_deref() == Some("tags") {
                            match attr.parse_args::<FieldTagArgs>() {
                                Ok(args) => skip |= args.skip,
                                Err(e) => errors.push(e),
                            }
                        }
                    }
                    if !skip {
                        let ty = &f.ty;
                        if type_contains_generics(&g, ty) {
                            generics.make_where_clause().predicates.push(
                                parse_quote_spanned! { ty.span() =>
                                    #ty: ::object_rainbow::Tagged
                                },
                            );
                        }
                    }
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

fn fields_tags(fields: &syn::Fields) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .filter_map(|f| {
            let mut skip = false;
            for attr in &f.attrs {
                if attr_str(attr).as_deref() == Some("tags") {
                    skip |= attr.parse_args::<FieldTagArgs>().ok()?.skip;
                }
            }
            let ty = &f.ty;
            (!skip).then_some(quote! { <#ty as ::object_rainbow::Tagged>::TAGS })
        })
        .collect()
}

fn gen_tags(data: &Data, attrs: &[Attribute], errors: &mut Vec<Error>) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let mut tags = Vec::new();
            for attr in attrs {
                if attr_str(attr).as_deref() == Some("tags") {
                    match attr.parse_args::<StructTagArgs>() {
                        Ok(mut args) => tags.append(&mut args.tags),
                        Err(e) => errors.push(e),
                    }
                }
            }
            let nested = fields_tags(&data.fields);
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
            let mut tags = Vec::new();
            for attr in attrs {
                if attr_str(attr).as_deref() == Some("tags") {
                    match attr.parse_args::<StructTagArgs>() {
                        Ok(mut args) => tags.append(&mut args.tags),
                        Err(e) => errors.push(e),
                    }
                }
            }
            let mut nested: Vec<_> = data
                .variants
                .iter()
                .flat_map(|v| fields_tags(&v.fields))
                .collect();
            let kind_tags = quote! {
                <
                    <
                        <
                            Self
                            as
                            ::object_rainbow::Enum
                        >::Kind
                        as
                        ::object_rainbow::enumkind::EnumKind
                    >::Tag
                    as  ::object_rainbow::Tagged
                >::TAGS
            };
            nested.insert(0, kind_tags);
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
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").into_compile_error()
        }
    }
}

#[proc_macro_derive(Object)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_object(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::Object<__E> for #name #ty_generics #where_clause {}
    };
    TokenStream::from(output)
}

fn bounds_object(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            let last_at = data.fields.len().checked_sub(1).unwrap_or_default();
            for (i, f) in data.fields.iter().enumerate() {
                let last = i == last_at;
                let ty = &f.ty;
                let tr = if last {
                    quote!(::object_rainbow::Object<__E>)
                } else {
                    quote!(::object_rainbow::Inline<__E>)
                };
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: #tr
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                let last_at = v.fields.len().checked_sub(1).unwrap_or_default();
                for (i, f) in v.fields.iter().enumerate() {
                    let last = i == last_at;
                    let ty = &f.ty;
                    let tr = if last {
                        quote!(::object_rainbow::Object<__E>)
                    } else {
                        quote!(::object_rainbow::Inline<__E>)
                    };
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: #tr
                            },
                        );
                    }
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
    generics.params.push(parse_quote!(__E: 'static));
    Ok(generics)
}

#[proc_macro_derive(Inline)]
pub fn derive_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_inline(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::Inline<__E> for #name #ty_generics #where_clause {}
    };
    TokenStream::from(output)
}

fn bounds_inline(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Inline<__E>
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::Inline<__E>
                            },
                        );
                    }
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
    generics.params.push(parse_quote!(__E: 'static));
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
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::ReflessObject for #name #ty_generics #where_clause {}
    };
    TokenStream::from(output)
}

fn bounds_refless_object(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = bounds_g(&generics);
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
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: #tr
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                let last_at = v.fields.len().checked_sub(1).unwrap_or_default();
                for (i, f) in v.fields.iter().enumerate() {
                    let last = i == last_at;
                    let ty = &f.ty;
                    let tr = if last {
                        quote!(::object_rainbow::ReflessObject)
                    } else {
                        quote!(::object_rainbow::ReflessInline)
                    };
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: #tr
                            },
                        );
                    }
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
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::ReflessInline for #name #ty_generics #where_clause {}
    };
    TokenStream::from(output)
}

fn bounds_refless_inline(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::ReflessInline
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::ReflessInline
                            },
                        );
                    }
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

#[proc_macro_derive(Size)]
pub fn derive_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let size_arr = gen_size_arr(&input.data);
    let size = gen_size(&input.data);
    let generics = match bounds_size(input.generics.clone(), &input.data, &size_arr) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let mut generics = input.generics;
    generics.params.push(parse_quote!(
        __Output: ::object_rainbow::typenum::Unsigned
    ));
    let (impl_generics, _, _) = generics.split_for_impl();
    let output = quote! {
        const _: () = {
            use ::object_rainbow::typenum::tarr;

            #[automatically_derived]
            impl #impl_generics ::object_rainbow::Size for #name #ty_generics #where_clause {
                const SIZE: usize = #size;

                type Size = <#size_arr as ::object_rainbow::typenum::FoldAdd>::Output;
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
    let g = bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(&g, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Size
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(&g, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::Size
                            },
                        );
                    }
                }
            }
            for v in data.variants.iter().skip(1) {
                let arr = fields_size_arr(&v.fields, true);
                generics.make_where_clause().predicates.push(parse_quote!(
                    #arr: ::object_rainbow::typenum::FoldAdd<Output = __Output>
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
        #size_arr: ::object_rainbow::typenum::FoldAdd<Output = __Output>
    ));
    Ok(generics)
}

fn fields_size_arr(fields: &syn::Fields, as_enum: bool) -> proc_macro2::TokenStream {
    let kind_size = quote! {
        <
            <
                <
                    Self
                    as
                    ::object_rainbow::Enum
                >::Kind
                as
                ::object_rainbow::enumkind::EnumKind
            >::Tag
            as  ::object_rainbow::Size
        >::Size
    };
    if fields.is_empty() {
        return if as_enum {
            quote! { tarr![#kind_size, ::object_rainbow::typenum::consts::U0] }
        } else {
            quote! { tarr![::object_rainbow::typenum::consts::U0] }
        };
    }
    let size_arr = fields.iter().map(|f| {
        let ty = &f.ty;
        quote! { <#ty as ::object_rainbow::Size>::Size }
    });
    if as_enum {
        quote! { tarr![#kind_size, #(#size_arr),*] }
    } else {
        quote! { tarr![#(#size_arr),*] }
    }
}

fn gen_size_arr(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => fields_size_arr(&data.fields, false),
        Data::Enum(data) => {
            if let Some(v) = data.variants.first() {
                fields_size_arr(&v.fields, true)
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
                let size = fields_size(&v.fields);
                let kind_size = quote! {
                    <
                        <
                            <
                                Self
                                as
                                ::object_rainbow::Enum
                            >::Kind
                            as
                            ::object_rainbow::enumkind::EnumKind
                        >::Tag
                        as  ::object_rainbow::Size
                    >::SIZE
                };
                quote! { #kind_size + #size }
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
        #[automatically_derived]
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
        #[automatically_derived]
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
        #[automatically_derived]
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

fn parse_path(attr: &Attribute) -> syn::Result<Type> {
    attr.parse_args::<LitStr>()?.parse()
}

fn attr_str(attr: &Attribute) -> Option<String> {
    Some(attr.path().get_ident()?.to_string())
}

#[proc_macro_derive(Enum, attributes(enumtag))]
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
    let mut errors = Vec::new();
    let mut enumtag = None;
    for attr in &input.attrs {
        if attr_str(attr).as_deref() == Some("enumtag") {
            match parse_path(attr) {
                Ok(path) => {
                    if enumtag.is_some() {
                        errors.push(Error::new_spanned(path, "duplicate tag"));
                    } else {
                        enumtag = Some(path);
                    }
                }
                Err(e) => errors.push(e),
            }
        }
    }
    let enumtag = enumtag
        .unwrap_or_else(|| parse_quote!(::object_rainbow::numeric::Le<::core::num::NonZero<u8>>));
    let errors = errors.into_iter().map(|e| e.into_compile_error());
    let output = quote! {
        const _: () = {
            #(#errors)*

            use ::object_rainbow::enumkind::EnumKind;

            #[derive(Clone, Copy, ::object_rainbow::ParseAsInline)]
            pub enum __Kind {
                #variants
            }

            #[automatically_derived]
            impl ::object_rainbow::enumkind::EnumKind for __Kind {
                type Tag = ::object_rainbow::enumkind::EnumTag<
                    #enumtag,
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

            #[automatically_derived]
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
                match tag.to_usize() {
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

#[proc_macro_derive(MaybeHasNiche)]
pub fn derive_maybe_has_niche(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let mn_array = gen_mn_array(&input.data);
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let generics = match bounds_maybe_has_niche(input.generics.clone(), &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let output = quote! {
        const _: () = {
            use ::object_rainbow::typenum::tarr;

            #[automatically_derived]
            impl #impl_generics ::object_rainbow::MaybeHasNiche for #name #ty_generics #where_clause {
                type MnArray = #mn_array;
            }
        };
    };
    TokenStream::from(output)
}

fn bounds_maybe_has_niche(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::MaybeHasNiche<
                            MnArray: ::object_rainbow::MnArray<
                                MaybeNiche: ::object_rainbow::MaybeNiche
                            >
                        >
                    });
            }
        }
        Data::Enum(data) => {
            generics.params.push(parse_quote!(
                __N: ::object_rainbow::typenum::Unsigned
            ));
            for (i, v) in data.variants.iter().enumerate() {
                let mn_array = fields_mn_array(&v.fields, Some(i));
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { v.span() =>
                        #mn_array: ::object_rainbow::MnArray<
                            MaybeNiche: ::object_rainbow::NicheOr<N = __N>
                        >
                    });
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::MaybeHasNiche<
                                MnArray: ::object_rainbow::MnArray<
                                    MaybeNiche: ::object_rainbow::MaybeNiche
                                >
                            >
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

fn fields_mn_array(fields: &syn::Fields, variant: Option<usize>) -> proc_macro2::TokenStream {
    let mn_array = fields.iter().map(|f| {
        let ty = &f.ty;
        quote! {
            <
                <
                    #ty
                    as
                    ::object_rainbow::MaybeHasNiche
                >::MnArray
                as
                ::object_rainbow::MnArray
            >::MaybeNiche
        }
    });
    if let Some(variant) = variant {
        let kind_niche = quote! {
            ::object_rainbow::AutoEnumNiche<Self, #variant>
        };
        quote! { tarr![#kind_niche, ::object_rainbow::NoNiche<::object_rainbow::HackNiche<#variant>>, #(#mn_array),*] }
    } else {
        quote! { tarr![#(#mn_array),*] }
    }
}

fn gen_mn_array(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => fields_mn_array(&data.fields, None),
        Data::Enum(data) => {
            let mn_array = data.variants.iter().enumerate().map(|(i, v)| {
                let mn_array = fields_mn_array(&v.fields, Some(i));
                quote! { <#mn_array as ::object_rainbow::MnArray>::MaybeNiche }
            });
            quote! {
                ::object_rainbow::NicheFoldOrArray<tarr![#(#mn_array),*]>
            }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").into_compile_error()
        }
    }
}
