//! Provides functions for deriving the traits on named structs.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, Ident, Lit, Meta};

/// Derives the `FromJson` trait for a named struct.
pub fn from_json_named_struct(ast: DeriveInput, r#struct: &DataStruct) -> TokenStream {
    let fields: Vec<Field> = r#struct.fields.iter().cloned().collect();

    let idents: Vec<Ident> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();

    let names: Vec<String> = fields
        .iter()
        .map(|field| {
            if field.attrs.is_empty() {
                field.ident.as_ref().unwrap().to_string()
            } else {
                let attr = field
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
                Ok(Self {
                    #(
                        #idents: ::humphrey_json::traits::FromJson::from_json(value.get(#names).unwrap_or(&::humphrey_json::Value::Null))?,
                    )*
                })
            }
        }
    };

    TokenStream::from(tokens)
}

/// Derives the `IntoJson` trait for a named struct.
pub fn into_json_named_struct(ast: DeriveInput, r#struct: &DataStruct) -> TokenStream {
    let fields: Vec<Field> = r#struct.fields.iter().cloned().collect();

    let idents: Vec<Ident> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();

    let names: Vec<String> = fields
        .iter()
        .map(|field| {
            if field.attrs.is_empty() {
                field.ident.as_ref().unwrap().to_string()
            } else {
                let attr = field
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

                json!({
                    #(
                        #names: (::humphrey_json::traits::IntoJson::to_json(&self.#idents)),
                    )*
                })
            }
        }
    };

    TokenStream::from(tokens)
}
