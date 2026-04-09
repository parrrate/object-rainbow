use std::collections::BTreeSet;

use syn::{AngleBracketedGenericArguments, Expr, Ident, Path, Type};

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
        syn::PathArguments::None => !g.is_empty() || g.contains(&seg.ident),
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

pub fn type_contains_generics(g: &BTreeSet<Ident>, ty: &Type) -> bool {
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
