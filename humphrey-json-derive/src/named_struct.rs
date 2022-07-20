//! Provides functions for deriving the traits on named structs.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Ident};

/// Derives the `FromJson` trait for a named struct.
pub fn from_json_named_struct(ast: DeriveInput, r#struct: &DataStruct) -> TokenStream {
    let idents: Vec<Ident> = r#struct
        .fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();

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

/// Derives the `IntoJson` trait for a named struct.
pub fn into_json_named_struct(ast: DeriveInput, r#struct: &DataStruct) -> TokenStream {
    let idents: Vec<Ident> = r#struct
        .fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();

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
