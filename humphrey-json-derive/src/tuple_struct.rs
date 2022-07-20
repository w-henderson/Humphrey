//! Provides functions for deriving the traits on tuple structs.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Index};

/// Derives the `FromJson` trait for a tuple struct.
pub fn from_json_tuple_struct(ast: DeriveInput, r#struct: &DataStruct) -> TokenStream {
    let field_count = r#struct.fields.len();
    let field_iter = 0..field_count;

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::FromJson for #name #ty_generics #where_clause {
            fn from_json(value: &::humphrey_json::Value) -> Result<Self, ::humphrey_json::error::ParseError> {
                if value.as_array().map(|v| v.len()).unwrap_or(0) != #field_count {
                    return Err(::humphrey_json::error::ParseError::TypeError);
                }

                Ok(Self(
                    #(
                        ::humphrey_json::traits::FromJson::from_json(value.get(#field_iter).unwrap_or(&::humphrey_json::Value::Null))?,
                    )*
                ))
            }
        }
    };

    TokenStream::from(tokens)
}

/// Derives the `IntoJson` trait for a tuple struct.
pub fn into_json_tuple_struct(ast: DeriveInput, r#struct: &DataStruct) -> TokenStream {
    let field_count = r#struct.fields.len();
    let field_iter = (0..field_count).into_iter().map(Index::from);

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::IntoJson for #name #ty_generics #where_clause {
            fn to_json(&self) -> ::humphrey_json::Value {
                ::humphrey_json::Value::Array(vec![
                    #(
                        ::humphrey_json::traits::IntoJson::to_json(&self.#field_iter),
                    )*
                ])
            }
        }
    };

    TokenStream::from(tokens)
}
