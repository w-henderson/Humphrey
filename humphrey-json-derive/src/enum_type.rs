//! Provides functions for deriving the traits on enums.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Ident};

/// Derives the `FromJson` trait for an enum.
pub fn from_json_enum(ast: DeriveInput, r#enum: &DataEnum) -> TokenStream {
    let variants: Vec<Ident> = r#enum
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    let variant_names: Vec<String> = variants.iter().map(|variant| variant.to_string()).collect();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::FromJson for #name #ty_generics #where_clause {
            fn from_json(value: &::humphrey_json::Value) -> Result<Self, ::humphrey_json::error::ParseError> {
                match value.as_str() {
                    Some(string) => match string {
                        #(
                            #variant_names => Ok(Self::#variants),
                        )*
                        _ => Err(::humphrey_json::error::ParseError::TypeError),
                    },
                    None => Err(::humphrey_json::error::ParseError::TypeError),
                }
            }
        }
    };

    TokenStream::from(tokens)
}

/// Derives the `IntoJson` trait for an enum.
pub fn into_json_enum(ast: DeriveInput, r#enum: &DataEnum) -> TokenStream {
    let variants: Vec<Ident> = r#enum
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    let variant_names: Vec<String> = variants.iter().map(|variant| variant.to_string()).collect();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::IntoJson for #name #ty_generics #where_clause {
            fn to_json(&self) -> ::humphrey_json::Value {
                use ::humphrey_json::json;

                match self {
                    #(
                        Self::#variants => json!(#variant_names),
                    )*
                }
            }
        }
    };

    TokenStream::from(tokens)
}
