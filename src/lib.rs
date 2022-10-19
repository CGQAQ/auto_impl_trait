extern crate core;

use proc_macro::{ TokenStream};
use syn::{FnArg, Item, parse_macro_input, Token, TraitItem };

use quote::{quote, ToTokens};
use quote::__private::Span;

use change_case::{pascal_case, snake_case};

#[derive(Debug)]
struct Trait(pub syn::LitStr, pub syn::Ident);

impl syn::parse::Parse for Trait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(Span::call_site(), "expected a path to a trait"));
        } else {
            let path = input.parse::<syn::LitStr>().expect(r#"
                first argument must be a string literal
                for example: `#[auto_impl_trait("./src/rect_trait.rs", Rect)]`
                "#);
            input.parse::<Token![,]>().expect(r#"missing comma"#);
            let ident = input.parse::<syn::Ident>().expect(r#"
                second argument must be an identifier
                for example: `#[auto_impl_trait("./src/rect_trait.rs", React)]`
                "#);
            return Ok(Trait(path, ident))
        }
    }
}


///
/// This crate is for internal use only.
///
/// ```ignore
/// #[auto_impl_trait("./src/rect_trait.rs", Rect)]
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
///     use std::ops::{Add, Sub, Mul, Div};
///
///     pub trait Rect {
///         type Item: Add + Sub + Mul + Div;
///         fn area(&self) -> Self::Item;
///         fn perimeter(&self) -> Self::Item;
///         fn scale(&mut self, scale: Self::Item);
///     }
/// }
/// use ____CGQAQ__SUPER_TRAIT____::Rect;
/// #[doc = "Test this will keep after expand"]
/// #[derive(Debug)]
/// struct Square {
///     side: i32,
/// }
/// impl Rect for Square {
///     type Item = crate::item::Item;
///     fn area(&self) -> Self::Item { <dyn crate::area::Area>::area(self) }
///     fn perimeter(&self) -> Self::Item { <dyn crate::perimeter::Perimeter>::perimeter(self) }
///     fn scale(&mut self, scale: Self::Item) { <dyn crate::scale::Scale>::scale(self, scale) }
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

    let trait_content = std::fs::read_to_string(trait_location)
        .expect("Something went wrong reading the file");

    let file = syn::parse_str::<syn::File>(trait_content.as_str()).unwrap();

    let trait_meta =  file.items.iter().find_map(|it| match it {
        Item::Trait(it) => if it.ident == trait_name {
            Some(it)
        } else {
            None
        },
        _ => None
    }).expect(format!("expect trait {} in file", trait_name).as_str());

    let trait_functions = trait_meta.items.clone();
    let trait_mods = trait_meta
        .items
        .clone()
        .iter()
        .map(|it| {
            match it {
                TraitItem::Method(method) => {
                    let method_name = method.sig.ident.clone();
                    return quote! {
                        mod #method_name;
                    };
                }
                TraitItem::Type(ty) => {
                    let ty_name = quote::format_ident!("{}", snake_case(ty.ident.clone().to_string().as_str()));
                    return quote! {
                        mod #ty_name;
                    }
                }
                _ => { panic!("Expect Method or Type") }
            }
        })
        .collect::<Vec<_>>();

    let impl_decls = trait_functions
        .iter()
        .map(|item| match item {
            syn::TraitItem::Method(method) => {
                let method_name = method.sig.ident.clone();
                let method_return_type = method.sig.output.clone();
                let method_args = method.sig.inputs.clone();
                let trait_name = quote::format_ident!("{}", pascal_case(&method_name.to_string()));

                // let impl_receiver = method_args.clone().iter().nth(0).map(|it| match it {
                //     FnArg::Receiver(re) => {
                //         re.clone()
                //     }
                //     FnArg::Typed(_) => { panic!("expect receiver") }
                // }).unwrap();
                let impl_args = method_args.clone().iter().filter_map(|it| match it {
                    FnArg::Receiver(_) => {
                        None
                    }
                    FnArg::Typed(ty) => {Some(ty.pat.clone().into_token_stream())}
                }).collect::<Vec<_>>();


                quote! {
                    fn #method_name(#method_args) #method_return_type {
                        <dyn crate::#method_name::#trait_name>::#method_name(/* #impl_receiver */ self, #(#impl_args)*)
                    }
                }
            }
            syn::TraitItem::Type(ty) => {
                let ty_name = ty.ident.clone();
                let ty_name_snake = quote::format_ident!("{}", snake_case(ty_name.to_string().as_str()));
                quote! {
                    type #ty_name = crate::#ty_name_snake::#ty_name;
                }
            }
            _ => { panic!("Expect Method or Type") }
        })
        .collect::<Vec<_>>();

    (quote! {
        #(#trait_mods)*
        mod ____CGQAQ__SUPER_TRAIT____ {
            #file
        }
        use ____CGQAQ__SUPER_TRAIT____::#trait_name;

        #item
        impl #trait_name for #item_name {
            #(#impl_decls)*
        }

    })
    .into()
}
