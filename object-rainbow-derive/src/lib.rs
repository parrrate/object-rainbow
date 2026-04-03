//! `#[derive(...)]`s for [`object-rainbow`](<https://docs.rs/object-rainbow>).

use std::collections::BTreeSet;

use darling::FromMeta;
use proc_macro::TokenStream;
use quote::{ToTokens, quote, quote_spanned};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Field, FnArg, GenericParam, Generics, Ident,
    ImplItem, ItemTrait, LitStr, Path, TraitItem, Type, TypeGenerics, parse::Parse,
    parse_macro_input, parse_quote, parse_quote_spanned, spanned::Spanned, token::Comma,
};

use self::contains_generics::{GContext, type_contains_generics};

mod contains_generics;

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

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct RainbowArgs {
    #[darling(default)]
    remote: Option<Type>,
}

fn parse_for(name: &Ident, attrs: &[Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr_str(attr).as_deref() == Some("rainbow") {
            match attr.parse_args::<RainbowArgs>() {
                Ok(RainbowArgs { remote }) => {
                    if let Some(remote) = remote {
                        return remote.to_token_stream();
                    }
                }
                Err(e) => return e.into_compile_error(),
            }
        }
    }
    name.to_token_stream()
}

/// ```rust
/// use object_rainbow::{InlineOutput, ToOutput};
///
/// #[derive(ToOutput)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C> ToOutput for Three<A, B, C>
///     where
///         A: InlineOutput,
///         B: InlineOutput,
///         C: ToOutput,
///     {}
/// );
/// ```
#[proc_macro_derive(ToOutput, attributes(rainbow))]
pub fn derive_to_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_to_output(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let to_output = gen_to_output(&input.data);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::ToOutput for #target #ty_generics #where_clause {
            fn to_output(&self, output: &mut dyn ::object_rainbow::Output) {
                #to_output
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_to_output(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = &bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            let last_at = data.fields.len().saturating_sub(1);
            for (i, f) in data.fields.iter().enumerate() {
                let last = i == last_at;
                let ty = &f.ty;
                let tr = if last {
                    quote!(::object_rainbow::ToOutput)
                } else {
                    quote!(::object_rainbow::InlineOutput)
                };
                if !last || type_contains_generics(GContext { g, always: false }, ty) {
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
                let last_at = v.fields.len().saturating_sub(1);
                for (i, f) in v.fields.iter().enumerate() {
                    let last = i == last_at;
                    let ty = &f.ty;
                    let tr = if last {
                        quote!(::object_rainbow::ToOutput)
                    } else {
                        quote!(::object_rainbow::InlineOutput)
                    };
                    if !last || type_contains_generics(GContext { g, always: false }, ty) {
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

/// ```rust
/// use object_rainbow::{InlineOutput, ToOutput};
///
/// #[derive(ToOutput, InlineOutput)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C> InlineOutput for Three<A, B, C>
///     where
///         A: InlineOutput,
///         B: InlineOutput,
///         C: InlineOutput,
///     {}
/// );
/// ```
#[proc_macro_derive(InlineOutput)]
pub fn derive_inline_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = match bounds_inline_output(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::InlineOutput for #target #ty_generics #where_clause {}
    };
    TokenStream::from(output)
}

fn bounds_inline_output(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote_spanned! { ty.span() =>
                        #ty: ::object_rainbow::InlineOutput
                    });
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::InlineOutput
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

/// ```rust
/// use object_rainbow::ListHashes;
///
/// #[derive(ListHashes)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C> ListHashes for Three<A, B, C>
///     where
///         A: ListHashes,
///         B: ListHashes,
///         C: ListHashes,
///     {}
/// );
/// ```
#[proc_macro_derive(ListHashes, attributes(topology))]
pub fn derive_list_hashes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_list_hashes(input.generics, &input.data) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let list_hashes = gen_list_hashes(&input.data);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::ListHashes for #target #ty_generics #where_clause {
            fn list_hashes(&self, visitor: &mut impl FnMut(::object_rainbow::Hash)) {
                #list_hashes
            }
        }
    };
    TokenStream::from(output)
}

fn bounds_list_hashes(mut generics: Generics, data: &Data) -> syn::Result<Generics> {
    let g = &bounds_g(&generics);
    match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(GContext { g, always: false }, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::ListHashes
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(GContext { g, always: false }, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: ::object_rainbow::ListHashes
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

fn fields_list_hashes(fields: &syn::Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let let_self = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            let list_hashes = let_self.clone().zip(fields.named.iter()).map(|(i, f)| {
                quote_spanned! { f.ty.span() =>
                    #i.list_hashes(visitor)
                }
            });
            quote! {
                { #(#let_self),* } => {
                    #(#list_hashes);*
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let let_self = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| Ident::new(&format!("field{i}"), f.ty.span()));
            let list_hashes = let_self.clone().zip(fields.unnamed.iter()).map(|(i, f)| {
                quote_spanned! { f.ty.span() =>
                    #i.list_hashes(visitor)
                }
            });
            quote! {
                (#(#let_self),*) => {
                    #(#list_hashes);*
                }
            }
        }
        syn::Fields::Unit => quote! {
            => {}
        },
    }
}

fn gen_list_hashes(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let arm = fields_list_hashes(&data.fields);
            quote! {
                match self {
                    Self #arm
                }
            }
        }
        Data::Enum(data) => {
            let to_output = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_list_hashes(&v.fields);
                quote! { Self::#ident #arm }
            });
            quote! {
                let kind = ::object_rainbow::Enum::kind(self);
                let tag = ::object_rainbow::enumkind::EnumKind::to_tag(kind);
                tag.list_hashes(visitor);
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

/// ```rust
/// use object_rainbow::{ListHashes, Topological};
///
/// #[derive(ListHashes, Topological)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C> Topological for Three<A, B, C>
///     where
///         A: Topological,
///         B: Topological,
///         C: Topological,
///     {}
/// );
/// ```
#[proc_macro_derive(Topological, attributes(topology))]
pub fn derive_topological(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut defs = Vec::new();
    let generics =
        match bounds_topological(input.generics, &input.data, &input.attrs, &name, &mut defs) {
            Ok(g) => g,
            Err(e) => return e.into_compile_error().into(),
        };
    let traverse = gen_traverse(&input.data, &ty_generics);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        const _: () = {
            #(#defs)*

            #[automatically_derived]
            impl #impl_generics ::object_rainbow::Topological for #target #ty_generics
            #where_clause
            {
                fn traverse(&self, visitor: &mut impl ::object_rainbow::PointVisitor) {
                    #traverse
                }
            }
        };
    };
    TokenStream::from(output)
}

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ContainerTopologyArgs {
    #[darling(default)]
    recursive: bool,
    #[darling(default)]
    inline: bool,
}

fn parse_recursive_inline(attrs: &[Attribute]) -> syn::Result<(bool, bool)> {
    let mut r = false;
    let mut i = false;
    for attr in attrs {
        if attr_str(attr).as_deref() == Some("topology") {
            let ContainerTopologyArgs { recursive, inline } = attr.parse_args()?;
            if recursive {
                r = true;
            }
            if inline {
                i = true;
            }
        }
    }
    Ok((r, i))
}

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct FieldTopologyArgs {
    bound: Option<Path>,
    #[darling(default)]
    unchecked: bool,
    with: Option<Expr>,
    #[darling(default, rename = "unstable_mutual")]
    mutual: bool,
}

fn bounds_topological(
    mut generics: Generics,
    data: &Data,
    attrs: &[Attribute],
    name: &Ident,
    defs: &mut Vec<proc_macro2::TokenStream>,
) -> syn::Result<Generics> {
    let (recursive, inline) = parse_recursive_inline(attrs)?;
    let g = &bounds_g(&generics);
    let g_clone = generics.clone();
    let (impl_generics, ty_generics, where_clause) = g_clone.split_for_impl();
    let this = quote_spanned! { name.span() =>
        #name #ty_generics
    };
    let bound = if recursive {
        quote! { ::object_rainbow::Traversible }
    } else {
        quote! { ::object_rainbow::Topological }
    };
    match data {
        Data::Struct(data) => {
            'field: for f in data.fields.iter() {
                let ty = &f.ty;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("topology") {
                        let FieldTopologyArgs {
                            bound,
                            unchecked,
                            mutual,
                            ..
                        } = attr.parse_args()?;
                        if mutual {
                            let conditional =
                                format!("__ConditionalTopology_{}", f.ident.as_ref().unwrap());
                            let conditional = Ident::new(&conditional, f.span());
                            defs.push(quote! {
                                #[allow(non_camel_case_types)]
                                trait #conditional #impl_generics #where_clause {
                                    fn traverse(
                                        &self, visitor: &mut impl ::object_rainbow::PointVisitor,
                                    ) where #this: ::object_rainbow::Traversible;
                                }

                                impl #impl_generics #conditional #ty_generics for #ty
                                #where_clause
                                {
                                    fn traverse(
                                        &self, visitor: &mut impl ::object_rainbow::PointVisitor,
                                    ) where #this: ::object_rainbow::Traversible {
                                        ::object_rainbow::Topological::traverse(self, visitor)
                                    }
                                }
                            });
                            b = Some(parse_quote!(#conditional #ty_generics));
                        }
                        if unchecked {
                            continue 'field;
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                let bound = if let Some(bound) = b {
                    quote! { #bound }
                } else {
                    bound.clone()
                };
                if type_contains_generics(GContext { g, always: false }, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: #bound
                        },
                    );
                }
            }
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                'field: for (i, f) in v.fields.iter().enumerate() {
                    let ty = &f.ty;
                    let mut b = None;
                    for attr in &f.attrs {
                        if attr_str(attr).as_deref() == Some("topology") {
                            let FieldTopologyArgs {
                                bound,
                                unchecked,
                                mutual,
                                ..
                            } = attr.parse_args()?;
                            if mutual {
                                let conditional = format!("__ConditionalTopology_{i}");
                                let conditional = Ident::new(&conditional, f.span());
                                defs.push(quote! {
                                #[allow(non_camel_case_types)]
                                trait #conditional #impl_generics #where_clause {
                                    fn traverse(
                                        &self, visitor: &mut impl ::object_rainbow::PointVisitor,
                                    ) where #this: ::object_rainbow::Traversible;
                                }

                                impl #impl_generics #conditional #ty_generics for #ty
                                #where_clause
                                {
                                    fn traverse(
                                        &self, visitor: &mut impl ::object_rainbow::PointVisitor,
                                    ) where #this: ::object_rainbow::Traversible {
                                        ::object_rainbow::Topological::traverse(self, visitor)
                                    }
                                }
                            });
                            }
                            if unchecked {
                                continue 'field;
                            }
                            if let Some(bound) = bound {
                                b = Some(bound);
                            }
                        }
                    }
                    let bound = if let Some(bound) = b {
                        quote! { #bound }
                    } else {
                        bound.clone()
                    };
                    if type_contains_generics(GContext { g, always: false }, ty) {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                #ty: #bound
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
    let output_bound = if inline {
        quote! {
            ::object_rainbow::InlineOutput
        }
    } else {
        quote! {
            ::object_rainbow::ToOutput
        }
    };
    if recursive {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote_spanned! { name.span() =>
                Self: #output_bound + ::object_rainbow::Tagged
            });
    }
    Ok(generics)
}

fn fields_traverse(
    fields: &syn::Fields,
    ty_generics: &TypeGenerics<'_>,
) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let let_self = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
            let traverse = let_self.clone().zip(fields.named.iter()).map(|(i, f)| {
                let ty = &f.ty;
                let mut w = None;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("topology") {
                        let FieldTopologyArgs {
                            with,
                            bound,
                            mutual,
                            ..
                        } = match attr.parse_args() {
                            Ok(args) => args,
                            Err(e) => return e.into_compile_error(),
                        };
                        if mutual {
                            let conditional = format!("__ConditionalTopology_{i}");
                            let conditional = Ident::new(&conditional, f.span());
                            w = Some(parse_quote!(traverse));
                            b = Some(parse_quote!(#conditional #ty_generics));
                        }
                        if let Some(with) = with {
                            w = Some(with);
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                if let Some(with) = w {
                    if let Some(bound) = b {
                        quote_spanned! { f.ty.span() =>
                            <#ty as #bound>::#with(#i, visitor)
                        }
                    } else {
                        quote_spanned! { f.ty.span() =>
                            #with(#i, visitor)
                        }
                    }
                } else {
                    quote_spanned! { f.ty.span() =>
                        #i.traverse(visitor)
                    }
                }
            });
            quote! {
                { #(#let_self),* } => {
                    #(#traverse);*
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let let_self = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| Ident::new(&format!("field{i}"), f.ty.span()));
            let traverse = let_self.clone().zip(fields.unnamed.iter()).map(|(i, f)| {
                let ty = &f.ty;
                let mut w = None;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("topology") {
                        let FieldTopologyArgs {
                            with,
                            bound,
                            mutual,
                            ..
                        } = match attr.parse_args() {
                            Ok(args) => args,
                            Err(e) => return e.into_compile_error(),
                        };
                        if mutual {
                            let conditional = format!("__ConditionalTopology_{i}");
                            let conditional = Ident::new(&conditional, f.span());
                            w = Some(parse_quote!(traverse));
                            b = Some(parse_quote!(#conditional #ty_generics));
                        }
                        if let Some(with) = with {
                            w = Some(with);
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                if let Some(with) = w {
                    if let Some(bound) = b {
                        quote_spanned! { f.ty.span() =>
                            <#ty as #bound>::#with(#i, visitor)
                        }
                    } else {
                        quote_spanned! { f.ty.span() =>
                            #with(#i, visitor)
                        }
                    }
                } else {
                    quote_spanned! { f.ty.span() =>
                        #i.traverse(visitor)
                    }
                }
            });
            quote! {
                (#(#let_self),*) => {
                    #(#traverse);*
                }
            }
        }
        syn::Fields::Unit => quote! {
            => {}
        },
    }
}

fn gen_traverse(data: &Data, ty_generics: &TypeGenerics<'_>) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            let arm = fields_traverse(&data.fields, ty_generics);
            quote! {
                match self {
                    Self #arm
                }
            }
        }
        Data::Enum(data) => {
            let to_output = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_traverse(&v.fields, ty_generics);
                quote! { Self::#ident #arm }
            });
            quote! {
                let kind = ::object_rainbow::Enum::kind(self);
                let tag = ::object_rainbow::enumkind::EnumKind::to_tag(kind);
                tag.traverse(visitor);
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

/// ```rust
/// use object_rainbow::Tagged;
///
/// #[derive(Tagged)]
/// struct Three<A, B, C> {
///     a: A,
///     #[tags(skip)]
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C> Tagged for Three<A, B, C>
///     where
///         A: Tagged,
///         C: Tagged,
///     {}
/// );
/// ```
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
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        #(#errors)*

        #[automatically_derived]
        impl #impl_generics ::object_rainbow::Tagged for #target #ty_generics #where_clause {
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
    let g = &bounds_g(&generics);
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
                    if type_contains_generics(GContext { g, always: false }, ty) {
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
                        if type_contains_generics(GContext { g, always: false }, ty) {
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

/// ```rust
/// use object_rainbow::Size;
///
/// #[derive(Size)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C> Size for Three<A, B, C>
///     where
///         A: Size<Size = typenum::U2>,
///         B: Size<Size = typenum::U3>,
///         C: Size<Size = typenum::U7>,
///     {}
/// );
///
/// assert_eq!(Three::<[u8; 2], [u8; 3], [u8; 7]>::SIZE, 12);
/// ```
#[proc_macro_derive(Size)]
pub fn derive_size(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let size_arr = gen_size_arr(&input.data);
    let size = gen_size(&input.data);
    let (generics, is_enum) = match bounds_size(input.generics.clone(), &input.data, &size_arr) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let mut generics = input.generics;
    if is_enum {
        generics.params.push(parse_quote!(
            __Output: ::object_rainbow::typenum::Unsigned
        ));
    }
    let (impl_generics, _, _) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        const _: () = {
            use ::object_rainbow::typenum::tarr;

            #[automatically_derived]
            impl #impl_generics ::object_rainbow::Size for #target #ty_generics #where_clause {
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
) -> syn::Result<(Generics, bool)> {
    let g = &bounds_g(&generics);
    let is_enum = match data {
        Data::Struct(data) => {
            for f in data.fields.iter() {
                let ty = &f.ty;
                if type_contains_generics(GContext { g, always: false }, ty) {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            #ty: ::object_rainbow::Size
                        },
                    );
                }
            }
            generics.make_where_clause().predicates.push(parse_quote!(
                #size_arr: ::object_rainbow::typenum::FoldAdd<
                    Output: ::object_rainbow::typenum::Unsigned
                >
            ));
            false
        }
        Data::Enum(data) => {
            for v in data.variants.iter() {
                for f in v.fields.iter() {
                    let ty = &f.ty;
                    if type_contains_generics(GContext { g, always: false }, ty) {
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
            generics.make_where_clause().predicates.push(parse_quote!(
                #size_arr: ::object_rainbow::typenum::FoldAdd<Output = __Output>
            ));
            true
        }
        Data::Union(data) => {
            return Err(Error::new_spanned(
                data.union_token,
                "`union`s are not supported",
            ));
        }
    };
    Ok((generics, is_enum))
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
        quote! { tarr![#kind_size, ::object_rainbow::typenum::consts::U0, #(#size_arr),*] }
    } else {
        quote! { tarr![::object_rainbow::typenum::consts::U0, #(#size_arr),*] }
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

/// ```rust
/// use object_rainbow::{Parse, ParseInline, ParseInput};
///
/// #[derive(Parse)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C, I> Parse<I> for Three<A, B, C>
///     where
///         A: ParseInline<I>,
///         B: ParseInline<I>,
///         C: Parse<I>,
///         I: ParseInput,
///     {}
/// );
/// ```
#[proc_macro_derive(Parse, attributes(parse))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut defs = Vec::new();
    let generics = match bounds_parse(input.generics, &input.data, &input.attrs, &name, &mut defs) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (parse, enum_parse) = gen_parse(&input.data, &ty_generics);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let enum_parse = enum_parse.map(|enum_parse| {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::object_rainbow::enumkind::EnumParse<__I> for #target #ty_generics
            #where_clause
            {
                fn enum_parse(
                    kind: <Self as ::object_rainbow::Enum>::Kind, mut input: __I,
                ) -> ::object_rainbow::Result<Self> {
                    #enum_parse
                }
            }
        }
    });
    let output = quote! {
        const _: () = {
            #(#defs)*

            #[automatically_derived]
            impl #impl_generics ::object_rainbow::Parse<__I> for #target #ty_generics
            #where_clause
            {
                fn parse(mut input: __I) -> ::object_rainbow::Result<Self> {
                    #parse
                }
            }

            #enum_parse
        };
    };
    TokenStream::from(output)
}

#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ParseArgs {
    bound: Option<Type>,
    #[darling(default)]
    unchecked: bool,
    with: Option<Expr>,
    #[darling(default, rename = "unstable_mutual")]
    mutual: bool,
}

fn conditional_parse_name(f: &Field, inline: bool) -> Ident {
    let infix = if inline { "ParseInline" } else { "Parse" };
    let conditional = format!("__Conditional{infix}_{}", f.ident.as_ref().unwrap());
    Ident::new(&conditional, f.span())
}

fn conditional_parse_input(inline: bool) -> proc_macro2::TokenStream {
    if inline {
        quote!(&mut impl ::object_rainbow::PointInput<Extra = Self::E>)
    } else {
        quote!(impl ::object_rainbow::PointInput<Extra = Self::E>)
    }
}

fn conditional_parse_method(inline: bool) -> proc_macro2::TokenStream {
    if inline {
        quote!(parse_inline)
    } else {
        quote!(parse)
    }
}

fn bounds_parse(
    mut generics: Generics,
    data: &Data,
    attrs: &[Attribute],
    name: &Ident,
    defs: &mut Vec<proc_macro2::TokenStream>,
) -> syn::Result<Generics> {
    let g_clone = generics.clone();
    let (impl_generics, ty_generics, where_clause) = g_clone.split_for_impl();
    let this = quote_spanned! { name.span() =>
        #name #ty_generics
    };
    let (recursive, _) = parse_recursive_inline(attrs)?;
    let tr = |last| match (last, recursive) {
        (true, true) => {
            quote!(::object_rainbow::Parse<__I> + ::object_rainbow::Object<__I::Extra>)
        }
        (true, false) => quote!(::object_rainbow::Parse<__I>),
        (false, true) => {
            quote!(::object_rainbow::ParseInline<__I> + ::object_rainbow::Inline<__I::Extra>)
        }
        (false, false) => quote!(::object_rainbow::ParseInline<__I>),
    };
    match data {
        Data::Struct(data) => {
            let last_at = data.fields.len().saturating_sub(1);
            'field: for (i, f) in data.fields.iter().enumerate() {
                let last = i == last_at;
                let ty = &f.ty;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("parse") {
                        let ParseArgs {
                            bound,
                            unchecked,
                            mutual,
                            ..
                        } = attr.parse_args::<ParseArgs>()?;
                        if mutual {
                            let conditional = conditional_parse_name(f, !last);
                            let mut g_clone = g_clone.clone();
                            g_clone.params.push(parse_quote!(
                                __E: ::core::marker::Send + ::core::marker::Sync
                            ));
                            let (impl_generics_extra, _, _) = g_clone.split_for_impl();
                            let input_type = conditional_parse_input(!last);
                            let parse_method = conditional_parse_method(!last);
                            defs.push(quote! {
                                #[allow(non_camel_case_types)]
                                trait #conditional #impl_generics: ::object_rainbow::BoundPair
                                #where_clause
                                {
                                    fn parse(
                                        input: #input_type,
                                    ) -> ::object_rainbow::Result<Self::T>
                                    where #this: ::object_rainbow::Object<Self::E>;
                                }

                                impl #impl_generics_extra #conditional #ty_generics
                                    for (#ty, __E)
                                #where_clause
                                {
                                    fn parse(
                                        input: #input_type,
                                    ) -> ::object_rainbow::Result<Self::T>
                                    where #this: ::object_rainbow::Object<Self::E> {
                                        input.#parse_method::<Self::T>()
                                    }
                                }
                            });
                            b = Some(parse_quote!(#conditional #ty_generics));
                        }
                        if unchecked {
                            continue 'field;
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                if let Some(bound) = b {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            (#ty, __I::Extra): ::object_rainbow::BoundPair<
                                T = #ty, E = __I::Extra
                            > + #bound
                        },
                    );
                } else {
                    let tr = tr(last);
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
                let last_at = v.fields.len().saturating_sub(1);
                'field: for (i, f) in v.fields.iter().enumerate() {
                    let ty = &f.ty;
                    let mut b = None;
                    for attr in &f.attrs {
                        if attr_str(attr).as_deref() == Some("parse") {
                            let ParseArgs {
                                bound, unchecked, ..
                            } = attr.parse_args::<ParseArgs>()?;
                            if unchecked {
                                continue 'field;
                            }
                            if let Some(bound) = bound {
                                b = Some(bound);
                            }
                        }
                    }
                    if let Some(bound) = b {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                (#ty, __I::Extra): ::object_rainbow::BoundPair<
                                    T = #ty, E = __I::Extra
                                > + #bound
                            },
                        );
                    } else {
                        let last = i == last_at;
                        let tr = tr(last);
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
    generics.params.push(if recursive {
        parse_quote!(__I: ::object_rainbow::PointInput<
            Extra: ::core::marker::Send + ::core::marker::Sync + ::core::clone::Clone
        >)
    } else {
        parse_quote!(__I: ::object_rainbow::ParseInput)
    });
    Ok(generics)
}

fn gen_parse(
    data: &Data,
    ty_generics: &TypeGenerics<'_>,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    match data {
        Data::Struct(data) => {
            let arm = fields_parse(&data.fields, ty_generics);
            (quote! { Ok(Self #arm)}, None)
        }
        Data::Enum(data) => {
            let parse = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_parse(&v.fields, ty_generics);
                quote! {
                    <Self as ::object_rainbow::Enum>::Kind::#ident => Self::#ident #arm,
                }
            });
            (
                quote! {
                    ::object_rainbow::enumkind::EnumParse::parse_as_enum(input)
                },
                Some(quote! {
                    Ok(match kind {
                        #(#parse)*
                    })
                }),
            )
        }
        Data::Union(data) => (
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error(),
            None,
        ),
    }
}

fn fields_parse(fields: &syn::Fields, ty_generics: &TypeGenerics<'_>) -> proc_macro2::TokenStream {
    let last_at = fields.len().saturating_sub(1);
    match fields {
        syn::Fields::Named(fields) => {
            let parse = fields.named.iter().enumerate().map(|(i, f)| {
                let last = i == last_at;
                let ty = &f.ty;
                let mut w = None;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("parse") {
                        let ParseArgs {
                            with,
                            bound,
                            mutual,
                            ..
                        } = match attr.parse_args::<ParseArgs>() {
                            Ok(args) => args,
                            Err(e) => return e.into_compile_error(),
                        };
                        if mutual {
                            let conditional = format!(
                                "__Conditional{}_{}",
                                if last { "Parse" } else { "ParseInline" },
                                f.ident.as_ref().unwrap(),
                            );
                            let conditional = Ident::new(&conditional, f.span());
                            w = Some(parse_quote!(parse));
                            b = Some(parse_quote!(#conditional #ty_generics));
                        }
                        if let Some(with) = with {
                            w = Some(with);
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                let i = f.ident.as_ref().unwrap();
                if let Some(with) = w {
                    let arg = if last {
                        quote!(input)
                    } else {
                        quote!(&mut input)
                    };
                    if let Some(bound) = b {
                        quote_spanned! { f.ty.span() =>
                            #i: <(#ty, __I::Extra) as #bound>::#with(#arg)?
                        }
                    } else {
                        quote_spanned! { f.ty.span() =>
                            #i: #with(#arg)?
                        }
                    }
                } else {
                    let method = if last {
                        quote!(parse)
                    } else {
                        quote!(parse_inline)
                    };
                    quote_spanned! { f.ty.span() =>
                        #i: input.#method()?
                    }
                }
            });
            quote! { { #(#parse),* } }
        }
        syn::Fields::Unnamed(fields) => {
            let parse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let ty = &f.ty;
                let mut w = None;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("parse") {
                        let ParseArgs { with, bound, .. } = match attr.parse_args::<ParseArgs>() {
                            Ok(args) => args,
                            Err(e) => return e.into_compile_error(),
                        };
                        if let Some(with) = with {
                            w = Some(with);
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                let last = i == last_at;
                if let Some(with) = w {
                    let arg = if last {
                        quote!(input)
                    } else {
                        quote!(&mut input)
                    };
                    if let Some(bound) = b {
                        quote_spanned! { f.ty.span() =>
                            <(#ty, __I::Extra) as #bound>::#with(#arg)?
                        }
                    } else {
                        quote_spanned! { f.ty.span() =>
                            #with(#arg)?
                        }
                    }
                } else {
                    let method = if last {
                        quote!(parse)
                    } else {
                        quote!(parse_inline)
                    };
                    quote_spanned! { f.ty.span() =>
                        input.#method()?
                    }
                }
            });
            quote! { (#(#parse),*) }
        }
        syn::Fields::Unit => quote! {},
    }
}

/// ```rust
/// use object_rainbow::{Parse, ParseInline, ParseInput};
///
/// #[derive(Parse, ParseInline)]
/// struct Three<A, B, C> {
///     a: A,
///     b: B,
///     c: C,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<A, B, C, I> ParseInline<I> for Three<A, B, C>
///     where
///         A: ParseInline<I>,
///         B: ParseInline<I>,
///         C: ParseInline<I>,
///         I: ParseInput,
///     {}
/// );
/// ```
#[proc_macro_derive(ParseInline, attributes(parse))]
pub fn derive_parse_inline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = match bounds_parse_inline(input.generics, &input.data, &input.attrs) {
        Ok(g) => g,
        Err(e) => return e.into_compile_error().into(),
    };
    let (parse_inline, enum_parse_inline) = gen_parse_inline(&input.data);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let target = parse_for(&name, &input.attrs);
    let enum_parse_inline = enum_parse_inline.map(|enum_parse_inline| {
        quote! {
            #[automatically_derived]
            impl #impl_generics ::object_rainbow::enumkind::EnumParseInline<__I>
            for #target #ty_generics #where_clause {
                fn enum_parse_inline(
                    kind: <Self as ::object_rainbow::Enum>::Kind, input: &mut __I,
                ) -> ::object_rainbow::Result<Self> {
                    #enum_parse_inline
                }
            }
        }
    });
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::ParseInline<__I>
        for #target #ty_generics #where_clause {
            fn parse_inline(input: &mut __I) -> ::object_rainbow::Result<Self> {
                #parse_inline
            }
        }

        #enum_parse_inline
    };
    TokenStream::from(output)
}

fn bounds_parse_inline(
    mut generics: Generics,
    data: &Data,
    attrs: &[Attribute],
) -> syn::Result<Generics> {
    let (recursive, _) = parse_recursive_inline(attrs)?;
    let tr = if recursive {
        quote!(::object_rainbow::ParseInline<__I> + ::object_rainbow::Inline<__I::Extra>)
    } else {
        quote!(::object_rainbow::ParseInline<__I>)
    };
    match data {
        Data::Struct(data) => {
            'field: for f in data.fields.iter() {
                let ty = &f.ty;
                let mut b = None;
                for attr in &f.attrs {
                    if attr_str(attr).as_deref() == Some("parse") {
                        let ParseArgs {
                            bound, unchecked, ..
                        } = attr.parse_args::<ParseArgs>()?;
                        if unchecked {
                            continue 'field;
                        }
                        if let Some(bound) = bound {
                            b = Some(bound);
                        }
                    }
                }
                if let Some(bound) = b {
                    generics.make_where_clause().predicates.push(
                        parse_quote_spanned! { ty.span() =>
                            (#ty, __I::Extra): #bound
                        },
                    );
                } else {
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
                'field: for f in v.fields.iter() {
                    let ty = &f.ty;
                    let mut b = None;
                    for attr in &f.attrs {
                        if attr_str(attr).as_deref() == Some("parse") {
                            let ParseArgs {
                                bound, unchecked, ..
                            } = attr.parse_args::<ParseArgs>()?;
                            if unchecked {
                                continue 'field;
                            }
                            if let Some(bound) = bound {
                                b = Some(bound);
                            }
                        }
                    }
                    if let Some(bound) = b {
                        generics.make_where_clause().predicates.push(
                            parse_quote_spanned! { ty.span() =>
                                (#ty, __I::Extra): #bound
                            },
                        );
                    } else {
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
    generics.params.push(if recursive {
        parse_quote!(__I: ::object_rainbow::PointInput<
            Extra: ::core::marker::Send + ::core::marker::Sync
        >)
    } else {
        parse_quote!(__I: ::object_rainbow::ParseInput)
    });
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

fn gen_parse_inline(data: &Data) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    match data {
        Data::Struct(data) => {
            let arm = fields_parse_inline(&data.fields);
            (quote! { Ok(Self #arm) }, None)
        }
        Data::Enum(data) => {
            let parse_inline = data.variants.iter().map(|v| {
                let ident = &v.ident;
                let arm = fields_parse_inline(&v.fields);
                quote! {
                    <Self as ::object_rainbow::Enum>::Kind::#ident => Self::#ident #arm,
                }
            });
            (
                quote! {
                    ::object_rainbow::enumkind::EnumParseInline::parse_as_inline_enum(input)
                },
                Some(quote! {
                    Ok(match kind {
                        #(#parse_inline)*
                    })
                }),
            )
        }
        Data::Union(data) => (
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error(),
            None,
        ),
    }
}

/// ```rust
/// use object_rainbow::{Parse, ParseAsInline, ParseInline, ParseInput};
///
/// #[derive(ParseAsInline)]
/// struct Thing<T> {
///     inner: T,
/// }
///
/// object_rainbow::assert_impl!(
///     impl<T, I> Parse<I> for Thing<T>
///     where
///         Thing<T>: ParseInline<I>,
///         I: ParseInput,
///     {}
/// );
/// ```
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
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        #[automatically_derived]
        impl #impl_generics ::object_rainbow::Parse<__I> for #target #ty_generics #where_clause {
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

/// ```rust
/// use std::num::NonZero;
///
/// use object_rainbow::{Enum, MaybeHasNiche, ToOutput};
///
/// #[derive(Enum, ToOutput, MaybeHasNiche)]
/// enum WithDefault {
///     A(u8),
///     B(bool),
/// }
///
/// assert_eq!(Some(WithDefault::A(32)).vec(), [0, 32]);
/// assert_eq!(Some(WithDefault::B(true)).vec(), [1, 1]);
/// assert_eq!(None::<WithDefault>.vec(), [2, 0]);
///
/// #[derive(Enum, ToOutput, MaybeHasNiche)]
/// #[enumtag("NonZero<u8>")]
/// enum WithNz {
///     A(u8),
///     B(bool),
/// }
///
/// assert_eq!(None::<WithNz>.vec(), [0, 0]);
/// assert_eq!(Some(WithNz::A(32)).vec(), [1, 32]);
/// assert_eq!(Some(WithNz::B(true)).vec(), [2, 1]);
///
/// #[derive(Enum, ToOutput, MaybeHasNiche)]
/// #[enumtag("bool")]
/// enum WithBool {
///     A(u8),
///     B(bool),
/// }
///
/// assert_eq!(Some(WithBool::A(32)).vec(), [0, 32]);
/// assert_eq!(Some(WithBool::B(true)).vec(), [1, 1]);
/// assert_eq!(None::<WithBool>.vec(), [2, 0]);
///
/// #[derive(Enum, ToOutput, MaybeHasNiche)]
/// #[enumtag("u8")]
/// enum WithU8 {
///     A(u8),
///     B(bool),
/// }
///
/// assert_eq!(Some(WithU8::A(32)).vec(), [0, 32]);
/// assert_eq!(Some(WithU8::B(true)).vec(), [1, 1]);
/// assert_eq!(None::<WithU8>.vec(), [1, 2]);
///
/// #[derive(Enum, ToOutput, MaybeHasNiche)]
/// #[enumtag("u8")]
/// enum WithoutNiche {
///     A(u8),
///     B(u8),
/// }
///
/// assert_eq!(Some(WithoutNiche::A(32)).vec(), [0, 0, 32]);
/// assert_eq!(Some(WithoutNiche::B(1)).vec(), [0, 1, 1]);
/// assert_eq!(None::<WithoutNiche>.vec(), [1]);
/// ```
#[proc_macro_derive(Enum, attributes(enumtag))]
pub fn derive_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let generics = input.generics;
    let length = gen_length(&input.data);
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
    let enumtag = enumtag.unwrap_or_else(|| {
        parse_quote!(
            ::object_rainbow::partial_byte_tag::PartialByteTag<#length>
        )
    });
    let errors = errors.into_iter().map(|e| e.into_compile_error());
    let target = parse_for(&name, &input.attrs);
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
            impl #impl_generics ::object_rainbow::Enum for #target #ty_generics #where_clause {
                type Kind = __Kind;

                fn kind(&self) -> Self::Kind {
                    #kind
                }
            }
        };
    };
    TokenStream::from(output)
}

fn gen_length(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(data) => {
            Error::new_spanned(data.struct_token, "`struct`s are not supported").to_compile_error()
        }
        Data::Enum(data) => {
            let name = format!("U{}", data.variants.len());
            let ident = Ident::new(&name, data.variants.span());
            quote! { ::object_rainbow::typenum::#ident }
        }
        Data::Union(data) => {
            Error::new_spanned(data.union_token, "`union`s are not supported").to_compile_error()
        }
    }
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

/// ```rust
/// use object_rainbow::{MaybeHasNiche, Size};
///
/// #[derive(Size, MaybeHasNiche)]
/// struct WithHole(bool, u8);
///
/// #[derive(Size, MaybeHasNiche)]
/// struct NoHole(u8, u8);
///
/// assert_eq!(Option::<WithHole>::SIZE, 2);
/// assert_eq!(Option::<NoHole>::SIZE, 3);
/// ```
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
    let target = parse_for(&name, &input.attrs);
    let output = quote! {
        const _: () = {
            use ::object_rainbow::typenum::tarr;

            #[automatically_derived]
            impl #impl_generics ::object_rainbow::MaybeHasNiche
            for #target #ty_generics #where_clause {
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
        quote! {
            tarr![
                #kind_niche,
                ::object_rainbow::NoNiche<::object_rainbow::HackNiche<#variant>>, #(#mn_array),*
            ]
        }
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

#[proc_macro_attribute]
pub fn derive_for_mapped(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemTrait);
    let name = input.ident.clone();
    let generics = input.generics.clone();
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut generics = input.generics.clone();
    let sup = if args.is_empty() {
        let sup = input.supertraits.clone();
        quote!(#sup)
    } else {
        args.into()
    };
    generics.params.push(parse_quote! {
        __T: #name #ty_generics
    });
    generics.params.push(parse_quote! {
        __M: #sup
    });
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let field = quote!(1);
    let i = input
        .items
        .clone()
        .into_iter()
        .map(|i| match i {
            TraitItem::Const(i) => ImplItem::Const({
                let const_token = i.const_token;
                let ident = i.ident;
                let colon_token = i.colon_token;
                let ty = i.ty;
                let semi_token = i.semi_token;
                parse_quote! {
                    #const_token
                    #ident
                    #colon_token
                    #ty
                    =
                    <__T as #name #ty_generics>::#ident
                    #semi_token
                }
            }),
            TraitItem::Fn(i) => ImplItem::Fn({
                let mut sig = i.sig;
                let ident = sig.ident.clone();
                let args = sig
                    .inputs
                    .iter_mut()
                    .enumerate()
                    .map(|(n, i)| match i {
                        FnArg::Receiver(receiver) => {
                            let reference = receiver.reference.as_ref().map(|(and, _)| and);
                            let mutability = receiver.mutability.as_ref();
                            let ident = &receiver.self_token;
                            quote!(#reference #mutability #ident.#field)
                        }
                        FnArg::Typed(pat_type) => {
                            let ident = Ident::new(&format!("arg{n}"), pat_type.span());
                            *pat_type.pat = parse_quote!(#ident);
                            quote!(#ident)
                        }
                    })
                    .collect::<Vec<_>>();
                parse_quote! {
                    #sig
                    {
                        <__T as #name #ty_generics>::#ident(
                            #(#args),*
                        )
                    }
                }
            }),
            TraitItem::Type(i) => ImplItem::Type({
                let type_token = i.type_token;
                let ident = i.ident;
                let semi_token = i.semi_token;
                parse_quote! {
                    #type_token
                    #ident
                    =
                    <__T as #name #ty_generics>::#ident
                    #semi_token
                }
            }),
            _ => unimplemented!("unknown/unsupported item"),
        })
        .collect::<Vec<_>>();
    let derived = quote! {
        impl #impl_generics #name #ty_generics for ::object_rainbow::map_extra::MappedExtra<
            __T,
            __M,
        >
        #where_clause
        {
            #(#i)*
        }
    };
    let output = quote! {
        #input

        #derived
    };
    output.into()
}
