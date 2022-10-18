extern crate core;

use proc_macro::TokenStream;
use syn::{FnArg, parse_macro_input};

use quote::{quote, ToTokens};

use change_case::pascal_case;

struct Trait(pub syn::LitStr);

impl syn::parse::Parse for Trait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ = input.parse()?;
        Ok(Trait(trait_))
    }
}

#[proc_macro_attribute]
pub fn auto_impl_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let attrs = parse_macro_input!(attr as syn::ExprLit);
    let item = parse_macro_input!(item as syn::DeriveInput);

    let item_name = item.ident.clone();

    let trait_location = match attrs.lit {
        syn::Lit::Str(trait_) => Trait(trait_),
        _ => panic!("Expected a string literal"),
    };

    let trait_content = std::fs::read_to_string(trait_location.0.value())
        .expect("Something went wrong reading the file");

    let trait_meta = syn::parse_str::<syn::ItemTrait>(trait_content.as_str()).unwrap();

    let trait_name = trait_meta.ident.clone();
    let trait_functions = trait_meta.items.clone();
    let trait_function_names = trait_meta
        .items
        .clone()
        .iter()
        .map(|it| {
            if let syn::TraitItem::Method(method) = it {
                let method_name = method.sig.ident.clone();
                return quote! {
                    mod #method_name;
                };
            } else {
                panic!("Expected a method");
            }
        })
        .collect::<Vec<_>>();

    let impl_functions = trait_functions
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
            _ => panic!("Expected a method"),
        })
        .collect::<Vec<_>>();

    (quote! {
        #(#trait_function_names)*
        #item

        #trait_meta
        impl #trait_name for #item_name {
             #(#impl_functions)*
        }

    })
    .into()
}
