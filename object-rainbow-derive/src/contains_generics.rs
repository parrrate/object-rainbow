use std::collections::BTreeSet;

use syn::{AngleBracketedGenericArguments, Expr, Ident, Path, Type};

#[derive(Clone, Copy)]
pub struct GContext<'a> {
    pub g: &'a BTreeSet<Ident>,
    pub points: bool,
}

fn expr_contains_generics(cx: GContext, expr: &Expr) -> bool {
    match expr {
        Expr::Path(expr) => path_contains_generics(cx, &expr.path),
        _ => unimplemented!(),
    }
}

fn args_contains_generics(cx: GContext, args: &AngleBracketedGenericArguments) -> bool {
    args.args.iter().any(|arg| match arg {
        syn::GenericArgument::Type(ty) => type_contains_generics(cx, ty),
        syn::GenericArgument::AssocType(ty) => {
            ty.generics
                .as_ref()
                .is_some_and(|args| args_contains_generics(cx, args))
                || type_contains_generics(cx, &ty.ty)
        }
        syn::GenericArgument::Const(expr) => expr_contains_generics(cx, expr),
        syn::GenericArgument::AssocConst(expr) => {
            expr.generics
                .as_ref()
                .is_some_and(|args| args_contains_generics(cx, args))
                || expr_contains_generics(cx, &expr.value)
        }
        _ => false,
    })
}

fn path_contains_generics(cx: GContext, path: &Path) -> bool {
    path.segments.iter().any(|seg| match &seg.arguments {
        syn::PathArguments::None => !cx.g.is_empty() || cx.g.contains(&seg.ident),
        syn::PathArguments::AngleBracketed(args) => {
            (cx.points && seg.ident == "Point") || args_contains_generics(cx, args)
        }
        syn::PathArguments::Parenthesized(args) => {
            args.inputs.iter().any(|ty| type_contains_generics(cx, ty))
                || match &args.output {
                    syn::ReturnType::Default => false,
                    syn::ReturnType::Type(_, ty) => type_contains_generics(cx, ty),
                }
        }
    })
}

pub fn type_contains_generics(cx: GContext, ty: &Type) -> bool {
    match ty {
        Type::Array(ty) => type_contains_generics(cx, &ty.elem),
        Type::BareFn(ty) => {
            ty.inputs.iter().any(|a| type_contains_generics(cx, &a.ty))
                || match &ty.output {
                    syn::ReturnType::Default => false,
                    syn::ReturnType::Type(_, ty) => type_contains_generics(cx, ty),
                }
        }
        Type::Group(ty) => type_contains_generics(cx, &ty.elem),
        Type::Macro(_) => true,
        Type::Paren(ty) => type_contains_generics(cx, &ty.elem),
        Type::Path(ty) => path_contains_generics(cx, &ty.path),
        Type::Reference(ty) => type_contains_generics(cx, &ty.elem),
        Type::Slice(ty) => type_contains_generics(cx, &ty.elem),
        Type::Tuple(ty) => ty.elems.iter().any(|ty| type_contains_generics(cx, ty)),
        _ => false,
    }
}
