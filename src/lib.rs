use proc_macro::TokenStream;
use syn::{
    parse_macro_input, FnArg, Item, ItemMod, Token, TraitItem, TraitItemMethod,
};

use quote::__private::Span;
use quote::{quote, ToTokens};

use proc_macro2::Ident;

#[inline]
fn gen_sub_trait_name(tname: &str, fname: &str) -> Ident {
    quote::format_ident!("__{}_{}__", tname, fname)
}

#[derive(Debug)]
struct Trait(pub syn::LitStr, pub syn::Ident, pub syn::LitStr);

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
    let item = parse_macro_input!(item as syn::DeriveInput);

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
    let trait_items_types = trait_items
        .iter()
        .filter_map(|it| match it {
            TraitItem::Type(ty) => {
                let ty_name = ty.ident.clone();
                let ty_name_snake =
                    quote::format_ident!("{}", change_case::snake_case(ty_name.to_string().as_str()));
                Some(quote! {
                    type #ty_name = crate::#ty_name_snake::#ty_name;
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let trait_item_funcs: Vec<TraitItemMethod> = trait_meta
        .items
        .iter()
        .filter_map(|it| {
            if let TraitItem::Method(it) = it {
                Some(it.clone())
            } else {
                None
            }
        })
        .collect();

    let sub_traits: Vec<_> = trait_item_funcs
        .iter()
        .map(|it| {
            let gen_trait_name = gen_sub_trait_name(
                trait_name.to_string().as_str(),
                it.sig.ident.to_string().as_str(),
            );

            let mut it = it.clone();
            if let syn::ReturnType::Type(_, ty) = &mut it.sig.output {
                if let syn::Type::Path(p) = ty.as_mut() {
                    if p.path.segments.first().unwrap().ident == "Self" {
                        let last = p.path.segments.last().unwrap().clone();
                        p.path.segments.clear();
                        p.path.segments.push(last);
                    }
                }
            }

            it.sig.inputs.iter_mut().for_each(|it| {
                if let syn::FnArg::Typed(t) = it {
                    if let syn::Type::Path(p) = t.ty.as_mut() {
                        if p.path.segments.first().unwrap().ident == "Self" {
                            let last = p.path.segments.last().unwrap().clone();
                            p.path.segments.clear();
                            p.path.segments.push(last);
                        }
                    }
                }
            });

            quote! {
                // trait #gen_trait_name {
                //     #(#trait_items_types)*
                //     #it
                // }
                // TODO: swap to upper one as soon as rustc support it "error[E0658]: associated type defaults are unstable"
                pub mod #gen_trait_name {
                    #(pub #trait_items_types)*
                    #[tonic::async_trait]
                    pub trait #gen_trait_name {
                        #it
                    }
                }
            }
        })
        .collect();

    let trait_mods = trait_meta
        .items
        .clone()
        .iter()
        .map(|it| match it {
            TraitItem::Method(method) => {
                let method_name = method.sig.ident.clone();
                return quote! {
                mod # method_name;
                };
            }
            TraitItem::Type(ty) => {
                let ty_name =
                    quote::format_ident!("{}", change_case::snake_case(ty.ident.clone().to_string().as_str()));
                return quote! {
                mod # ty_name;
                };
            }
            _ => {
                panic!("Expect Method or Type")
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
                let trait_name = gen_sub_trait_name(
                    trait_name.to_string().as_str(),
                    method_name.to_string().as_str(),
                );

                // let impl_receiver = method_args.clone().iter().nth(0).map(|it| match it {
                //     FnArg::Receiver(re) => {
                //         re.clone()
                //     }
                //     FnArg::Typed(_) => { panic!("expect receiver") }
                // }).unwrap();
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
                < dyn #trait_name::#trait_name >::#method_name(/* #impl_receiver */ self, # ( # impl_args) * )
                }
                }
            }
            syn::TraitItem::Type(ty) => {
                let ty_name = ty.ident.clone();
                let ty_name_snake =
                    quote::format_ident!("{}", change_case::snake_case(ty_name.to_string().as_str()));
                quote! {
                type # ty_name = crate::# ty_name_snake::#ty_name;
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
        // ___INJECTED BY auto_impl_trait: gen SUB TRAIT___
        #(#sub_traits)*
        // ___INJECTED BY auto_impl_trait: publish SUPER TRAIT___
        use ____CGQAQ__SUPER_TRAIT____::#trait_name;

        // ___INJECTED BY auto_impl_trait: ORIGINAL SOURCE___
        #item
        // ___INJECTED BY auto_impl_trait: impl block___
        // TODO: Check if async_trait needed
        #[tonic::async_trait]
        impl #trait_name for #item_name {
            #(#impl_decls)*
        }
        // ___INJECTED BY auto_impl_trait: END___
    })
        .into()
}
