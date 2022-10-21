#![feature(proc_macro_span)]

use proc_macro::TokenStream;
use std::path::{Path, PathBuf};
use proc_macro2::Ident;
use syn::{
    parse_macro_input, FnArg, Item, ItemMod, Token, TraitItem, TraitItemMethod,
};

use quote::__private::Span;
use quote::{quote, ToTokens};

const PLACEHOLDER: &str = "__please_change_me__";

#[derive(Debug)]
struct Trait(pub syn::LitStr, pub syn::Ident, pub syn::LitStr);

#[inline]
fn get_path(file_name: &str) -> PathBuf {
    dbg!(Span::call_site().unwrap().source_file().path().with_file_name(format!("{}{}", file_name, ".rs")));
    Span::call_site().unwrap().source_file().path().with_file_name(format!("{}{}", file_name, ".rs"))
}

#[inline]
fn is_path_not_crate_root(path: PathBuf) -> bool {
    path.file_name().unwrap().to_str().unwrap() != "lib.rs" &&
        path.file_name().unwrap().to_str().unwrap() != "main.rs"
}

#[inline]
fn is_not_crate_root() -> bool {
    is_path_not_crate_root(Span::call_site().unwrap().source_file().path())
}

#[inline]
fn get_method_filename(method_name: &str) -> String {
    format!("__method_{}", change_case::snake_case(method_name))
}

#[inline]
fn get_type_filename(type_name: &str) -> String {
    format!("__type_{}", change_case::snake_case(type_name))
}

#[inline]
fn get_const_filename(const_name: &str) -> String {
    format!("__const_{}", change_case::snake_case(const_name))
}

#[inline]
fn ensure_interface(item: &TraitItem, struct_name: &syn::Ident) {
    let PLACEHOLDER_IDENT = Ident::new(PLACEHOLDER, Span::call_site());

    match item {
        TraitItem::Const(c) => {
            let mod_name = get_const_filename(c.ident.to_string().as_str());
            let mod_ident = c.ident.clone();
            let mut mod_type = c.ty.clone();
            if is_path_not_crate_root(get_path(&mod_name)) {
                if let syn::Type::Path(p) = &mut mod_type {
                    if p.path.segments.first().unwrap().ident.to_string() == "Self" {
                        p.path.segments.first_mut().unwrap().ident = Ident::new("super", Span::call_site());
                    }
                }
            } else {
                if let syn::Type::Path(p) = &mut mod_type {
                    if p.path.segments.first().unwrap().ident.to_string() == "Self" {
                        p.path.segments.first_mut().unwrap().ident = Ident::new("crate", Span::call_site());
                    }
                }
            }
            if !std::path::Path::exists(&*get_path(mod_name.as_str())) {
                std::fs::write(get_path(mod_name.as_str()), (quote! {
                    pub const #mod_ident: #mod_type = #PLACEHOLDER_IDENT;
                }).to_string()).expect(format!("write file {} failed", mod_name).as_str());
            }
        }
        TraitItem::Method(m) => {
            let mod_name = get_method_filename(m.sig.ident.to_string().as_str());
            let mod_ident = m.sig.ident.clone();
            let mod_args = m.sig.inputs.clone();
            let mod_return = m.sig.output.clone();
            let mod_asyncness = m.sig.asyncness;
            let mod_constness = m.sig.constness;
            let mod_unsafety = m.sig.unsafety;
            let mod_abi = m.sig.abi.clone();
            let mod_where_clause = m.sig.generics.where_clause.clone();
            let mod_fn = m.sig.fn_token;

            if !std::path::Path::exists(&*get_path(mod_name.as_str())) {
                mod_args.iter_mut().for_each(|it| match it{
                    FnArg::Receiver(_) => {}
                    FnArg::Typed(t) => {
                        if let syn::Type::Path(p) = &mut t.ty {
                            if p.path.segments.first().unwrap().ident.to_string() == "Self" {
                                p.path.segments.first_mut().unwrap().ident = Ident::new("super", Span::call_site());
                            }
                        }
                    }
                })

                std::fs::write(get_path(mod_name.as_str()), (quote! {
                    use super::#struct_name;

                    impl #struct_name {
                        pub #mod_abi #mod_unsafety #mod_constness #mod_asyncness #mod_fn #mod_ident(#mod_args) #mod_return #mod_where_clause {
                            unimplemented!(#PLACEHOLDER);
                        }
                    }
                }).to_string()).expect(format!("write file {} failed", mod_name).as_str());
            }
        }
        TraitItem::Type(t) => {
            let mod_name = get_type_filename(t.ident.to_string().as_str());
            let mod_ident = t.ident.clone();
            let mod_generics = t.generics.clone();
            let mod_where_clause = t.generics.where_clause.clone();
            let mod_type = t.type_token;

            if !std::path::Path::exists(&*get_path(mod_name.as_str())) {
                std::fs::write(get_path(mod_name.as_str()), (quote! {
                    pub #mod_type #mod_ident #mod_generics #mod_where_clause = #PLACEHOLDER_IDENT;
                }).to_string()).expect(format!("write file {} failed", mod_name).as_str());
            }
        }
        _ => {
            panic!("Only Const, Method, and Type are supported in the trait");
        }
    }
}

impl syn::parse::Parse for Trait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "expected a path to a trait",
            ));
        } else {
            let path = input.parse::<syn::LitStr>().expect(
                r#"
                first argument must be a string literal
                for example: `#[auto_impl_trait("./src/rect_trait.rs", Rect)]`
                "#,
            );
            input.parse::<Token![,]>().expect(r#"missing comma"#);
            let ident = input.parse::<syn::Ident>().expect(
                r#"
                second argument must be an identifier
                for example: `#[auto_impl_trait("./src/rect_trait.rs", React)]`
                "#,
            );

            let mut prefix = syn::LitStr::new("", Span::call_site());
            if input.parse::<Token![,]>().is_ok() {
                prefix = input.parse::<syn::LitStr>().expect(
                    r#"
                    third argument must be prefix string
                    for example: `#[auto_impl_trait("./src/rect_trait.rs", React, "runtime")]
                    "#,
                );
            }

            return Ok(Trait(path, ident, prefix));
        }
    }
}

///
/// This crate is for internal use only.
///
/// ```ignore
/// #[auto_impl_trait("./src/rect_trait.rs", Rect, "runtime")]
/// #[doc = "Test this will keep after expand"]
/// #[derive(Debug)]
/// struct Square {
///     side: i32,
/// }
/// ```
///
/// will expand to
///
/// ```ignore
/// mod item;
/// mod area;
/// mod perimeter;
/// mod scale;
/// mod ____CGQAQ__SUPER_TRAIT____ {
///     pub mod runtime {
///         use std::ops::{Add, Sub, Mul, Div};
///
///         pub trait Rect {
///             type Item: Add + Sub + Mul + Div;
///             fn area(&self) -> Self::Item;
///             fn perimeter(&self) -> Self::Item;
///             fn scale(&mut self, scale: Self::Item);
///         }
///     }
///
///     pub use runtime::Rect;
/// }
/// pub mod __Rect_area__ {
///     pub type Item = crate::item::Item;
///
///     pub trait __Rect_area__ { fn area(&self) -> Item; }
/// }
/// pub mod __Rect_perimeter__ {
///     pub type Item = crate::item::Item;
///
///     pub trait __Rect_perimeter__ { fn perimeter(&self) -> Item; }
/// }
/// pub mod __Rect_scale__ {
///     pub type Item = crate::item::Item;
///
///     pub trait __Rect_scale__ { fn scale(&mut self, scale: Item); }
/// }
/// use ____CGQAQ__SUPER_TRAIT____::Rect;
/// #[doc = "Test this will keep after expand"]
/// #[derive(Debug)]
/// struct Square {
///     side: i32,
/// }
/// impl Rect for Square {
///     type Item = crate::item::Item;
///     fn area(&self) -> Self::Item { <dyn __Rect_area__::__Rect_area__>::area(self) }
///     fn perimeter(&self) -> Self::Item { <dyn __Rect_perimeter__::__Rect_perimeter__>::perimeter(self) }
///     fn scale(&mut self, scale: Self::Item) { <dyn __Rect_scale__::__Rect_scale__>::scale(self, scale) }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn auto_impl_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let attrs = parse_macro_input!(attr as Trait);
    let item = parse_macro_input!(item as syn::ItemStruct);

    let item_name = item.ident.clone();

    let trait_location = attrs.0.value();
    let trait_name = attrs.1;
    let prefix = attrs.2;
    let mut prefix_stack: Vec<&str> = Vec::new();

    let trait_content =
        std::fs::read_to_string(trait_location).expect("Something went wrong reading the file");

    let file = syn::parse_str::<syn::File>(trait_content.as_str()).unwrap();

    let trait_meta = if prefix.value().is_empty() {
        file.items
            .iter()
            .find_map(|it| match it {
                Item::Trait(it) => {
                    if it.ident == trait_name {
                        Some(it)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .cloned()
            .expect(format!("expect trait {} in file", trait_name).as_str())
    } else {
        let mut current: Option<ItemMod> = None;
        for part in prefix.value().split("::") {
            prefix_stack.push(part);
            if let Some(cur) = current.clone() {
                if let Some(content) = cur.content {
                    current = Some(
                        content
                            .1
                            .iter()
                            .find_map(|it| match it {
                                Item::Mod(m) => {
                                    if m.ident == part {
                                        Some(m.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            })
                            .expect(format!("Mod {} is empty", prefix_stack.join("::")).as_str()),
                    )
                } else {
                    panic!("Mod {} is empty", prefix_stack.join("::"));
                }
            } else {
                current = Some(
                    file.items
                        .iter()
                        .find_map(|it| match it {
                            Item::Mod(m) => {
                                if m.ident == part {
                                    Some(m.clone())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        })
                        .expect(
                            format!("file must contains mod {}", prefix_stack.join("::")).as_str(),
                        ),
                )
            }
        }

        if let Some(current) = current {
            if let Some(content) = current.content {
                content
                    .1
                    .iter()
                    .find_map(|it| match it {
                        Item::Trait(t) => {
                            if t.ident == trait_name {
                                Some(t.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .expect(
                        format!(
                            "trait \"{}\" is not in prefix \"{}\"",
                            trait_name.clone().to_string(),
                            prefix.value()
                        )
                            .as_str(),
                    )
            } else {
                panic!(
                    "trait \"{}\" is not in prefix \"{}\"",
                    trait_name.clone().to_string(),
                    prefix.value()
                )
            }
        } else {
            panic!(
                "file must contains trait {}::{}",
                prefix.value(),
                trait_name.clone().to_string()
            )
        }
    };

    let trait_items = trait_meta.items.clone();

    dbg!(&trait_items);

    // TODO(CGQAQ): make this behind flag
    trait_items.iter().for_each(|it| ensure_interface(it, &item_name));

    let trait_mods = trait_meta
        .items
        .clone()
        .iter()
        .map(|it| match it {
            TraitItem::Method(method) => {
                let method_name = quote::format_ident!("{}", get_method_filename(method.sig.ident.to_string().as_str()));
                return quote! {
                    mod #method_name;
                };
            }
            TraitItem::Type(ty) => {
                let ty_name =
                    quote::format_ident!("{}", get_type_filename(ty.ident.to_string().as_str()));
                return quote! {
                    mod #ty_name;
                };
            }
            TraitItem::Const(c) => {
                let const_name =
                    quote::format_ident!("{}", get_const_filename(c.ident.to_string().as_str()));
                return quote! {
                    mod #const_name;
                };
            }
            _ => {
                panic!("Expect Method, Type or Const");
            }
        })
        .collect::<Vec<_>>();

    let impl_decls = trait_items
        .iter()
        .map(|item| match item {
            syn::TraitItem::Method(method) => {
                let method_name = method.sig.ident.clone();
                let method_return_type = method.sig.output.clone();
                let method_args = method.sig.inputs.clone();

                let impl_args = method_args
                    .clone()
                    .iter()
                    .filter_map(|it| match it {
                        FnArg::Receiver(_) => None,
                        FnArg::Typed(ty) => Some(ty.pat.clone().into_token_stream()),
                    })
                    .collect::<Vec<_>>();

                quote! {
                    fn #method_name( # method_args) # method_return_type {
                        self.#method_name(#(#impl_args)*)
                    }
                }
            }
            syn::TraitItem::Type(ty) => {
                let ty_name = ty.ident.clone();
                let ty_name_snake =
                    quote::format_ident!("{}", get_type_filename(ty.ident.to_string().as_str()));
                if is_not_crate_root() {
                    quote! {
                        type #ty_name = super::#ty_name_snake::#ty_name;
                    }
                } else {
                    quote! {
                        type #ty_name = crate::#ty_name_snake::#ty_name;
                    }
                }
            }
            syn::TraitItem::Const(c) => {
                let const_name = c.ident.clone();
                let mut const_ty = c.ty.clone();
                let const_name_snake =
                    quote::format_ident!("{}", get_const_filename(c.ident.to_string().as_str()));

                if let syn::Type::Path(ref mut p) = const_ty {
                    if p.path.segments.first().expect("expect path segment").ident.to_string() == "Self" {
                        let type_name_snake =
                            quote::format_ident!("{}", get_type_filename(p.path.segments.last().unwrap().ident.to_string().as_str()));
                        p.path.segments.first_mut().unwrap().ident = type_name_snake.clone();
                    }
                }

                if is_not_crate_root() {
                    quote! {
                        const #const_name: super::#const_ty = super::#const_name_snake::#const_name;
                    }
                } else {
                    quote! {
                        const #const_name: crate::#const_ty = crate::#const_name_snake::#const_name;
                    }
                }
            }
            _ => {
                panic!("Expect Method or Type")
            }
        })
        .collect::<Vec<_>>();

    let prefix_parsed: proc_macro2::TokenStream = prefix.parse().unwrap();
    (quote! {
        // ___INJECTED BY auto_impl_trait: BEGIN___
        #(#trait_mods)*
        // ___INJECTED BY auto_impl_trait: SUPER TRAIT ___
        mod ____CGQAQ__SUPER_TRAIT____ {
            #file
            // ___INJECTED BY auto_impl_trait: 3rd argument___
            pub use #prefix_parsed::#trait_name;
        }
        // ___INJECTED BY auto_impl_trait: publish SUPER TRAIT___
        use ____CGQAQ__SUPER_TRAIT____::#trait_name;

        // ___INJECTED BY auto_impl_trait: ORIGINAL SOURCE___
        #item
        // ___INJECTED BY auto_impl_trait: impl block___
        #[tonic::async_trait]
        impl #trait_name for #item_name {
            #(#impl_decls)*
        }
        // ___INJECTED BY auto_impl_trait: END___
    })
        .into()
}
