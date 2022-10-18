use proc_macro::TokenStream;
use syn::parse_macro_input;

use quote::quote;

#[derive(Debug)]
struct Trait(pub syn::Ident);

impl syn::parse::Parse for Trait {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ = input.parse()?;
        Ok(Trait(trait_))
    }
}

fn get_trait_functions(trait_: &Trait) {
    // get all trait functions names
}

#[proc_macro_attribute]
pub fn auto_impl_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    println!("{:#?}", &attr);

    let attrs = parse_macro_input!(attr as Trait);
    let item = parse_macro_input!(item as syn::DeriveInput);

    // Get all Trait functions
    // let trait_functions = get_trait_functions(&attrs);

    quote! {}.into()
}
