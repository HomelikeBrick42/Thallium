#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives the `thallium_ecs::Component` trait
#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::thallium::ecs::Component for #name #ty_generics #where_clause {
        }
    }
    .into()
}

/// Derives the `thallium_ecs::Resource` trait
#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::thallium::ecs::Resource for #name #ty_generics #where_clause {
        }
    }
    .into()
}
