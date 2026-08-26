use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, ItemFn, Lit,
    LitBool, LitStr, Path, Token,
};

/// Derive `mlua::FromLua` and `codemod_recipe_lua::host_api::HostSpec` for a
/// simple Luau table spec.
///
/// Reads `#[serde(rename_all = "camelCase")]` and `#[serde(rename = "...")]`
/// attributes so the Lua key names match the JSON schema. The generated Luau
/// type name is always the Rust struct name.
#[proc_macro_derive(LuaSpec, attributes(serde))]
pub fn derive_lua_spec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "LuaSpec can only be derived for structs")
            .into_compile_error()
            .into();
    };
    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(
            &data.fields,
            "LuaSpec can only be derived for named-field structs",
        )
        .into_compile_error()
        .into();
    };

    let rename_all = serde_rename_all(&input.attrs);
    let mut field_names = Vec::new();
    let mut lua_keys = Vec::new();
    for field in &fields.named {
        let field_name = field.ident.as_ref().expect("named field").to_string();
        let rename = serde_rename(&field.attrs);
        let key = rename.unwrap_or_else(|| apply_rename_all(&field_name, rename_all.as_deref()));
        let ident = field.ident.clone().expect("named field");
        field_names.push(ident);
        lua_keys.push(key);
    }

    let from_lua_impl = quote! {
        impl ::mlua::FromLua for #name {
            fn from_lua(value: ::mlua::Value, _lua: &::mlua::Lua) -> ::mlua::Result<Self> {
                let table = match value {
                    ::mlua::Value::Table(t) => t,
                    other => {
                        return Err(::mlua::Error::FromLuaConversionError {
                            from: other.type_name(),
                            to: stringify!(#name).to_string(),
                            message: Some("expected table".to_string()),
                        });
                    }
                };
                Ok(Self {
                    #(#field_names: table.get(#lua_keys)?),*
                })
            }
        }
    };

    let host_spec_impl = quote! {
        impl ::codemod_recipe_lua::host_api::HostSpec for #name {
            const NAME: &'static str = stringify!(#name);
            fn schema() -> ::schemars::schema::RootSchema {
                ::schemars::schema_for!(#name)
            }
        }
    };

    quote! {
        #from_lua_impl
        #host_spec_impl
    }
    .into()
}

/// Mark a handler function as a `c:` method. Generates a `HostMethodSurface`
/// distributed-slice entry and, unless `recursive = true`, a `HostMethod`
/// runtime registration entry.
///
/// Example:
/// ```ignore
/// #[host_method(name = "insert", description = "...", spec = InsertSpec, return_type = Nil)]
/// fn do_insert(host: &SharedHost, _lua: &Lua, spec: InsertSpec) -> Result<Value, String> { ... }
/// ```
#[proc_macro_attribute]
pub fn host_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as HostMethodArgs);
    let item_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &item_fn.sig.ident;
    let surface_ident = Ident::new(
        &format!("__HOST_API_SURFACE_{}", fn_name),
        Span::call_site(),
    );
    let method_ident = Ident::new(&format!("__HOST_METHOD_{}", fn_name), Span::call_site());

    let name = &args.name;
    let description = &args.description;
    let return_type = &args.return_type;
    let accept_string = args.accept_string;

    let signature = match &args.kind {
        MethodKind::Spec(spec) => quote! {
            crate::host_api::registry::HostMethodSignature::Spec(
                crate::host_api::registry::HostSpecInfo {
                    name: <#spec as ::codemod_recipe_lua::host_api::HostSpec>::NAME,
                    schema: <#spec as ::codemod_recipe_lua::host_api::HostSpec>::schema,
                },
            )
        },
        MethodKind::StringId => quote! {
            crate::host_api::registry::HostMethodSignature::StringId
        },
        MethodKind::Assert => quote! {
            crate::host_api::registry::HostMethodSignature::Assert
        },
        MethodKind::Run => quote! {
            crate::host_api::registry::HostMethodSignature::Run
        },
    };

    let surface_static = quote! {
        #[allow(unsafe_code)]
        #[::linkme::distributed_slice(crate::host_api::registry::HOST_API_SURFACE)]
        #[allow(dead_code)]
        static #surface_ident: crate::host_api::registry::HostMethodSurface =
            crate::host_api::registry::HostMethodSurface {
                name: #name,
                description: #description,
                signature: #signature,
                return_type: crate::host_api::registry::HostReturnType::#return_type,
                accept_string: #accept_string,
            };
    };

    if args.recursive {
        // Only surface metadata; the runtime registration is wired manually
        // (e.g. `run` needs to call `build_ctx` recursively).
        return quote! {
            #item_fn
            #surface_static
        }
        .into();
    }

    let register_closure = match &args.kind {
        MethodKind::Spec(spec) => quote! {
            |lua, ctx, host| {
                crate::host_api::registry::set_fn(lua, ctx, #name, {
                    let host = ::std::rc::Rc::clone(host);
                    move |lua, (_this, spec): (::mlua::Value, #spec)| {
                        #fn_name(&host, lua, spec).map_err(crate::host_api::registry::rt)
                    }
                })
            }
        },
        MethodKind::StringId => quote! {
            |lua, ctx, host| {
                crate::host_api::registry::set_fn(lua, ctx, #name, {
                    let host = ::std::rc::Rc::clone(host);
                    move |lua, (_this, id): (::mlua::Value, ::std::string::String)| {
                        #fn_name(&host, lua, id).map_err(crate::host_api::registry::rt)
                    }
                })
            }
        },
        MethodKind::Assert => quote! {
            |lua, ctx, host| {
                crate::host_api::registry::set_fn(lua, ctx, #name, {
                    let host = ::std::rc::Rc::clone(host);
                    move |lua, (_this, cond, msg): (::mlua::Value, bool, ::std::option::Option<::std::string::String>)| {
                        #fn_name(&host, lua, cond, msg).map_err(crate::host_api::registry::rt)
                    }
                })
            }
        },
        MethodKind::Run => {
            return syn::Error::new_spanned(
                &item_fn,
                "run methods must use recursive = true; registration is handled manually in build_ctx",
            )
            .into_compile_error()
            .into();
        }
    };

    let method_static = quote! {
        #[allow(unsafe_code)]
        #[::linkme::distributed_slice(crate::host_api::registry::HOST_METHODS)]
        #[allow(dead_code)]
        static #method_ident: crate::host_api::registry::HostMethod =
            crate::host_api::registry::HostMethod {
                surface: &#surface_ident,
                register: #register_closure,
            };
    };

    quote! {
        #item_fn
        #surface_static
        #method_static
    }
    .into()
}

// --- derive helpers ---

fn serde_rename_all(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut case = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                case = Some(lit.value());
            }
            Ok(())
        });
        if case.is_some() {
            return case;
        }
    }
    None
}

fn serde_rename(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        let mut rename = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value = meta.value()?;
                let lit: LitStr = value.parse()?;
                rename = Some(lit.value());
            }
            Ok(())
        });
        if rename.is_some() {
            return rename;
        }
    }
    None
}

fn apply_rename_all(name: &str, case: Option<&str>) -> String {
    match case {
        Some("camelCase") => snake_to_camel(name),
        Some("PascalCase") => pascal_case(name),
        Some("kebab-case") => name.replace('_', "-"),
        Some("SCREAMING_SNAKE_CASE") => name.to_ascii_uppercase(),
        _ => name.to_string(),
    }
}

fn snake_to_camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for ch in s.chars() {
        if ch == '_' {
            upper = true;
        } else if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn pascal_case(s: &str) -> String {
    let camel = snake_to_camel(s);
    let mut chars = camel.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

// --- attribute helpers ---

struct HostMethodArgs {
    name: String,
    description: String,
    return_type: Ident,
    kind: MethodKind,
    recursive: bool,
    accept_string: bool,
}

enum MethodKind {
    Spec(Path),
    StringId,
    Assert,
    Run,
}

impl Parse for HostMethodArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<String> = None;
        let mut description: Option<String> = None;
        let mut return_type: Option<Ident> = None;
        let mut spec: Option<Path> = None;
        let mut string_id = false;
        let mut assert = false;
        let mut run = false;
        let mut recursive = false;
        let mut accept_string = false;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;

            if ident == "name" {
                input.parse::<Token![=]>()?;
                name = Some(parse_str_lit(input)?);
            } else if ident == "description" {
                input.parse::<Token![=]>()?;
                description = Some(parse_str_lit(input)?);
            } else if ident == "return_type" {
                input.parse::<Token![=]>()?;
                return_type = Some(input.parse()?);
            } else if ident == "spec" {
                input.parse::<Token![=]>()?;
                spec = Some(input.parse()?);
            } else if ident == "string_id" {
                string_id = true;
            } else if ident == "assert" {
                assert = true;
            } else if ident == "run" {
                run = true;
            } else if ident == "recursive" {
                input.parse::<Token![=]>()?;
                recursive = parse_bool_lit(input)?;
            } else if ident == "accept_string" {
                input.parse::<Token![=]>()?;
                accept_string = parse_bool_lit(input)?;
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unknown host_method argument",
                ));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let kind = if let Some(path) = spec {
            if string_id || assert || run {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "spec is mutually exclusive with string_id, assert, and run",
                ));
            }
            MethodKind::Spec(path)
        } else if string_id {
            if assert || run {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "string_id is mutually exclusive with assert and run",
                ));
            }
            MethodKind::StringId
        } else if assert {
            if run {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "assert is mutually exclusive with run",
                ));
            }
            MethodKind::Assert
        } else if run {
            MethodKind::Run
        } else {
            return Err(syn::Error::new(
                Span::call_site(),
                "host_method requires one of: spec, string_id, assert, run",
            ));
        };

        let name =
            name.ok_or_else(|| syn::Error::new(Span::call_site(), "host_method requires name"))?;
        let description = description.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "host_method requires description")
        })?;
        let return_type = return_type.ok_or_else(|| {
            syn::Error::new(Span::call_site(), "host_method requires return_type")
        })?;

        Ok(HostMethodArgs {
            name,
            description,
            return_type,
            kind,
            recursive,
            accept_string,
        })
    }
}

fn parse_str_lit(input: ParseStream) -> syn::Result<String> {
    let lit: Lit = input.parse()?;
    match lit {
        Lit::Str(s) => Ok(s.value()),
        other => Err(syn::Error::new_spanned(other, "expected string literal")),
    }
}

fn parse_bool_lit(input: ParseStream) -> syn::Result<bool> {
    let expr: Expr = input.parse()?;
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(LitBool { value, .. }),
            ..
        }) => Ok(value),
        other => Err(syn::Error::new_spanned(other, "expected boolean literal")),
    }
}
