//! Provides functions for deriving the traits on enums.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, DeriveInput, Ident, Lit, Meta, Variant};

/// Derives the `FromJson` trait for an enum.
pub fn from_json_enum(ast: DeriveInput, r#enum: &DataEnum) -> TokenStream {
    let variants: Vec<Variant> = r#enum.variants.iter().cloned().collect();

    let idents: Vec<Ident> = variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    let names: Vec<String> = variants
        .iter()
        .map(|variant| {
            if variant.attrs.is_empty() {
                variant.ident.to_string()
            } else {
                let attr = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.path.is_ident("rename"))
                    .expect("Unknown attribute");
                let meta = attr.parse_meta().unwrap();

                match meta {
                    Meta::NameValue(name_value) => match name_value.lit {
                        Lit::Str(s) => s.value(),
                        _ => panic!("Attribute format incorrect"),
                    },
                    _ => panic!("Attribute format incorrect"),
                }
            }
        })
        .collect();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::FromJson for #name #ty_generics #where_clause {
            fn from_json(value: &::humphrey_json::Value) -> Result<Self, ::humphrey_json::error::ParseError> {
                match value.as_str() {
                    Some(string) => match string {
                        #(
                            #names => Ok(Self::#idents),
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
    let variants: Vec<Variant> = r#enum.variants.iter().cloned().collect();

    let idents: Vec<Ident> = variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();

    let names: Vec<String> = variants
        .iter()
        .map(|variant| {
            if variant.attrs.is_empty() {
                variant.ident.to_string()
            } else {
                let attr = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.path.is_ident("rename"))
                    .expect("Unknown attribute");
                let meta = attr.parse_meta().unwrap();

                match meta {
                    Meta::NameValue(name_value) => match name_value.lit {
                        Lit::Str(s) => s.value(),
                        _ => panic!("Attribute format incorrect"),
                    },
                    _ => panic!("Attribute format incorrect"),
                }
            }
        })
        .collect();

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics ::humphrey_json::traits::IntoJson for #name #ty_generics #where_clause {
            fn to_json(&self) -> ::humphrey_json::Value {
                use ::humphrey_json::json;

                match self {
                    #(
                        Self::#idents => json!(#names),
                    )*
                }
            }
        }
    };

    TokenStream::from(tokens)
}
