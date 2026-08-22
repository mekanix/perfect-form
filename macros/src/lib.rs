use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    FnArg, ItemFn, LitStr, Pat, PatType, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Attribute syntax:
///
/// ```ignore
/// #[permissions(any("admin", "editor"), all("verified"))]
/// #[route(GET "/admin")]
/// async fn admin(cx: &Cx) -> Result<auth::Page> { ... }
/// ```
///
/// Requirements are evaluated in order. A user satisfies `any(...)` when they
/// hold at least one of the listed roles, and `all(...)` when they hold every
/// listed role. Admin users bypass all role checks.
///
/// With no requirements the attribute only requires a logged-in user.
///
/// The attribute must be placed *outside* `#[route]` / `#[page]` so it runs
/// first and the route macro sees the guarded function body.
#[proc_macro_attribute]
pub fn permissions(attr: TokenStream, item: TokenStream) -> TokenStream {
    let permissions = match syn::parse::<Permissions>(attr) {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut func = match syn::parse::<ItemFn>(item) {
        Ok(value) => value,
        Err(err) => return err.to_compile_error().into(),
    };

    let cx_ident = match find_cx_param(&func) {
        Some(ident) => ident,
        None => {
            return syn::Error::new_spanned(
                &func.sig,
                "#[permissions] requires a `cx: &Cx` parameter",
            )
            .to_compile_error()
            .into();
        }
    };

    let requirements = permissions.requirements.iter().map(|req| match req {
        Requirement::Any(roles) => {
            let roles = roles.iter();
            quote! { crate::auth::RoleRequirement::Any(&[#(#roles),*]) }
        }
        Requirement::All(roles) => {
            let roles = roles.iter();
            quote! { crate::auth::RoleRequirement::All(&[#(#roles),*]) }
        }
    });

    let body = &func.block;
    let stmts = &body.stmts;

    func.block = syn::parse2(quote! {{
        {
            let __cfg: &crate::config::Config = topcoat::context::app_context(#cx_ident);
            let __db: &sea_orm::DatabaseConnection = topcoat::context::app_context(#cx_ident);
            match crate::auth::require_roles(#cx_ident, __db, __cfg, &[#(#requirements),*]).await? {
                crate::auth::AuthOutcome::User(_) => {}
                crate::auth::AuthOutcome::LoginRedirect => {
                    return Ok(crate::auth::Page::Redirect(topcoat::router::error::see_other("/login")));
                }
                crate::auth::AuthOutcome::ForbiddenRedirect => {
                    return Ok(crate::auth::Page::Redirect(topcoat::router::error::see_other("/")));
                }
            }
        }
        #(#stmts)*
    }})
    .expect("generated a valid function body");

    func.into_token_stream().into()
}

enum Requirement {
    Any(Punctuated<LitStr, syn::Token![,]>),
    All(Punctuated<LitStr, syn::Token![,]>),
}

struct Permissions {
    requirements: Vec<Requirement>,
}

impl Parse for Permissions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut requirements = Vec::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            let roles: Punctuated<LitStr, syn::Token![,]> = Punctuated::parse_terminated(&content)?;

            if roles.is_empty() {
                return Err(syn::Error::new_spanned(
                    &ident,
                    "role list must not be empty",
                ));
            }

            if ident == "any" {
                requirements.push(Requirement::Any(roles));
            } else if ident == "all" {
                requirements.push(Requirement::All(roles));
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "expected `any(...)` or `all(...)`",
                ));
            }

            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(Permissions { requirements })
    }
}

fn find_cx_param(func: &ItemFn) -> Option<syn::Ident> {
    func.sig.inputs.iter().find_map(|arg| {
        let FnArg::Typed(PatType { pat, ty, .. }) = arg else {
            return None;
        };
        if !is_cx_type(ty) {
            return None;
        }
        let Pat::Ident(ident) = &**pat else {
            return None;
        };
        Some(ident.ident.clone())
    })
}

fn is_cx_type(ty: &Type) -> bool {
    let Type::Reference(reference) = ty else {
        return false;
    };
    if reference.mutability.is_some() {
        return false;
    }
    let Type::Path(path) = &*reference.elem else {
        return false;
    };
    path.qself.is_none()
        && path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Cx")
}
