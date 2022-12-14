// #[cfg(all(feature = "v1", feature = "v1alpha2"))]
// compile_error!("features `v1` and `v1alpha2` are mutually exclusive");

use proc_macro::{TokenStream};

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{Ident, TraitItem, Block, Token, Type, TypePath, AngleBracketedGenericArguments, GenericArgument, ConstParam, ItemMod};
use syn::visit_mut::{visit_angle_bracketed_generic_arguments_mut, visit_const_param_mut, visit_item_mod_mut, visit_type_path_mut, VisitMut};

const RUNTIME_MOD: &str = "runtime_service_server";
const RUNTIME_TRAIT: &str = "RuntimeService";
const IMAGE_MOD: &str = "image_service_server";
const IMAGE_TRAIT: &str = "ImageService";

#[inline]
fn remove_fisrt(segments: &mut syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]>) {
    let mut segment: syn::punctuated::Punctuated<_, Token![::]> = syn::punctuated::Punctuated::new();
    segments.iter().skip(1).for_each(|it| segment.push(it.clone()));
    segments.clear();
    segments.extend(segment);
}

struct SuperRemover;

impl VisitMut for SuperRemover {
    fn visit_type_path_mut(&mut self, i: &mut TypePath) {
        visit_type_path_mut(self, i);

        if i.path.segments.len() > 1 {
            if i.path.segments.first().unwrap().ident == "super" {
                remove_fisrt(&mut i.path.segments);
            }
        }
    }

    fn visit_const_param_mut(&mut self, i: &mut ConstParam) {
        visit_const_param_mut(self, i);

        match &mut i.ty {
            Type::Path(p) => {
                self.visit_type_path_mut(&mut p.clone());
            }
            _ => { /* NOOP */ }
        }
    }


    fn visit_angle_bracketed_generic_arguments_mut(&mut self, i: &mut AngleBracketedGenericArguments) {
        visit_angle_bracketed_generic_arguments_mut(self, i);
        i.args.iter_mut().for_each(|it| {
            match it {
                GenericArgument::Type(t) => {
                    match t {
                        Type::Path(p) => {
                            self.visit_type_path_mut(p);
                        }
                        _ => {}
                    }
                }
                GenericArgument::Constraint(_) => {}
                _ => {}
            }
        })
    }
}

struct RuntimeRedundantRemover;
impl VisitMut for RuntimeRedundantRemover {
    fn visit_item_mod_mut(&mut self, i: &mut ItemMod) {
        visit_item_mod_mut(self, i);

        // remove redundant mod
        if i.ident == IMAGE_MOD {
            i.vis = syn::Visibility::Inherited;
        }
    }
}

struct ImageRedundantRemover;
impl VisitMut for ImageRedundantRemover {
    fn visit_item_mod_mut(&mut self, i: &mut ItemMod) {
        visit_item_mod_mut(self, i);

        // remove redundant mod
        if i.ident == RUNTIME_MOD {
            i.vis = syn::Visibility::Inherited;
        }
    }
}

#[derive(Clone, Copy)]
enum ServiceType {
    Runtime,
    Image,
}

impl ServiceType {
    const fn get_mod(&self) -> &str {
        match self {
            ServiceType::Runtime => RUNTIME_MOD,
            ServiceType::Image => IMAGE_MOD,
        }
    }

    const fn get_trait(&self) -> &str {
        match self {
            ServiceType::Runtime => RUNTIME_TRAIT,
            ServiceType::Image => IMAGE_TRAIT,
        }
    }
}

impl syn::parse::Parse for ServiceType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Err(syn::Error::new(
                Span::call_site(),
                "proto type is required(runtime or image)",
            ))
        } else {
            let ident = input.parse::<Ident>()?;
            if ident == "runtime" {
                Ok(ServiceType::Runtime)
            } else if ident == "image" {
                Ok(ServiceType::Image)
            } else {
                Err(syn::Error::new(
                    Span::call_site(),
                    "proto type is required(runtime or image)",
                ))
            }
        }
    }
}

fn expand(file_content: String, container: Ident, service_type: &ServiceType, struct_ast: &syn::ItemStruct) -> proc_macro2::TokenStream {
    let mut file = syn::parse_file(&*file_content).expect("parse file faild");

    match &service_type {
        ServiceType::Runtime => {
            RuntimeRedundantRemover.visit_file_mut(&mut file);
        }
        ServiceType::Image => {
            ImageRedundantRemover.visit_file_mut(&mut file);
        }
    }

    let struct_name = struct_ast.ident.clone();
    let trait_name = quote::format_ident!("{}", service_type.get_trait());
    let trait_meta = file.items.iter().find_map::<syn::ItemMod, _>(|item| {
        if let syn::Item::Mod(m) = item {
            if service_type.get_mod() == m.ident.to_string().as_str() {
                Some(m.clone())
            } else {
                None
            }
        } else {
            None
        }
    })
        .expect(format!("{} is not found", service_type.get_mod()).as_str())
        .content
        .expect(format!("{} is empty mod", service_type.get_mod()).as_str());
    let trait_meta = trait_meta
        .1
        .iter()
        .find_map(|it| {
            if let syn::Item::Trait(t) = it {
                if service_type.get_trait() == t.ident.to_string().as_str() {
                    Some(t)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .expect("find RuntimeService faild");

    let mut trait_items = trait_meta.items.clone();
    let trait_items = trait_items.iter_mut().filter_map(|it| {
        match it {
            TraitItem::Method(m) => {
                let name = quote::format_ident!("{}", m.sig.ident);

                SuperRemover.visit_trait_item_method_mut(m);

                let args = m.sig.inputs.iter_mut().filter_map(|it| {
                    match it {
                        syn::FnArg::Typed(t) => {
                            match *t.pat {
                                syn::Pat::Ident(ref mut i) => {
                                    Some(i.ident.clone())
                                }
                                _ => None,
                            }
                        }
                        _ => None
                    }
                }).collect::<Vec<_>>();

                if m.sig.asyncness.is_some() {
                    m.default = Some(syn::parse::<Block>(TokenStream::from(quote! {
                        {self.#name(#(#args),*).await}
                    }.to_token_stream())).unwrap());
                } else {
                    m.default = Some(syn::parse::<Block>(TokenStream::from(quote! {
                        {self.#name(#(#args),*)}
                    }.to_token_stream())).unwrap());
                }

                Some(quote! {
                    #m
                })
            }
            TraitItem::Type(t) => {
                let name = quote::format_ident!("{}", t.ident);
                Some(quote! {
                    type #name = super::#name;
                })
            }
            _ => { None /* not used */ }
        }
    });

    let trait_alias = quote::format_ident!("{}_{}", service_type.get_trait(), container);

    let use_block = match service_type {
        ServiceType::Runtime => {
            quote! {
                use #container::runtime_service_server::#trait_name as #trait_alias;
            }
        }
        ServiceType::Image => {
            quote! {
                use #container::image_service_server::#trait_name as #trait_alias;
            }
        }
    };

    (quote! {
        pub mod #container {
            #file

            #[tonic::async_trait]
            impl super::#trait_alias for super::#struct_name {
                #(#trait_items)*
            }
        }
        #use_block
    }).into()
}

///
/// This crate is for internal use only.
///
#[proc_macro_attribute]
pub fn auto_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let service_type = syn::parse_macro_input!(attr as ServiceType);
    let struct_ast = syn::parse_macro_input!(item as syn::ItemStruct);


    let file_content_v1 = std::fs::read_to_string("./proto/runtime.v1.rs").expect("read ./proto/runtime.v1.rs faild");
    let file_content_v1alpha2 = std::fs::read_to_string("./proto/runtime.v1alpha2.rs").expect("read ./proto/runtime.v1alpha2.rs faild");

    let container_v1 = quote::format_ident!("v1");
    let v1 = expand(file_content_v1, container_v1, &service_type, &struct_ast);
    let container_v1alpha2 = quote::format_ident!("v1alpha2");
    let v1alpha2 = expand(file_content_v1alpha2, container_v1alpha2, &service_type, &struct_ast);


    let doc = match &service_type {
        ServiceType::Runtime => {
            quote! {
                #[doc = "Generated by `#[derive(auto_impl(runtime))]`"]
            }
        }
        ServiceType::Image => {
            quote! {
                #[doc = "Generated by `#[derive(auto_impl(image))]`"]
            }
        }
    };

    (quote! {
        #doc
        #struct_ast

        #[doc = "Version v1 impls generated by auto_impl"]
        #v1
        #[doc = "Version v1alpha2 Impls generated by auto_impl"]
        #v1alpha2
    }).into()
}
