extern crate proc_macro;

use proc_macro::TokenStream;

use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Error, Ident};

use quote::quote;

#[proc_macro_derive(FromJson)]
pub fn derive_from_json(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let idents: Vec<Ident> = match ast.data {
        Data::Struct(r#struct) => r#struct
            .fields
            .iter()
            .map(|field| field.ident.clone().unwrap())
            .collect(),
        _ => {
            return Error::new(
                ast.span(),
                "`FromJson` can currently only be derived for structs",
            )
            .to_compile_error()
            .into()
        }
    };

    let fields: Vec<String> = idents
        .clone()
        .into_iter()
        .map(|ident| ident.to_string())
        .collect();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::FromJson for #name #ty_generics #where_clause {
            fn from_json(value: &::humphrey_json::Value) -> Result<Self, ::humphrey_json::error::ParseError> {
                Ok(Self {
                    #(
                        #idents: ::humphrey_json::traits::FromJson::from_json(value.get(#fields).unwrap_or(&::humphrey_json::Value::Null))?,
                    )*
                })
            }
        }
    };

    TokenStream::from(tokens)
}

#[proc_macro_derive(IntoJson)]
pub fn derive_into_json(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let idents: Vec<Ident> = match ast.data {
        Data::Struct(r#struct) => r#struct
            .fields
            .iter()
            .map(|field| field.ident.clone().unwrap())
            .collect(),
        _ => {
            return Error::new(
                ast.span(),
                "`IntoJson` can currently only be derived for structs",
            )
            .to_compile_error()
            .into()
        }
    };

    let fields: Vec<String> = idents
        .clone()
        .into_iter()
        .map(|ident| ident.to_string())
        .collect();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::IntoJson for #name #ty_generics #where_clause {
            fn to_json(&self) -> ::humphrey_json::Value {
                use ::humphrey_json::json;

                json!({
                    #(
                        #fields: (::humphrey_json::traits::IntoJson::to_json(&self.#idents)),
                    )*
                })
            }
        }
    };

    TokenStream::from(tokens)
}
