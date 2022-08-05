//! Provides the derive macros for Humphrey JSON.
//!
//! These macros should only be used from the Humphrey JSON crate itself, never directly from this crate. You can read about their uses in the [Humphrey JSON documentation](https://humphrey.whenderson.dev/json/data-structures.html).

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

extern crate proc_macro;

mod enum_type;
mod named_struct;
mod tuple_struct;

use proc_macro::TokenStream;

use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Error};

/// Derives the `FromJson` trait for a type.
///
/// This macro can be used on named structs, tuple structs, and enums. It is not currently supported for enums with data variants.
#[proc_macro_derive(FromJson, attributes(rename))]
pub fn derive_from_json(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match &ast.data.clone() {
        Data::Struct(r#struct) => {
            if r#struct
                .fields
                .iter()
                .map(|field| field.ident.clone())
                .all(|ident| ident.is_some())
                && !r#struct.fields.is_empty()
            {
                named_struct::from_json_named_struct(ast, r#struct)
            } else if !r#struct.fields.is_empty() {
                tuple_struct::from_json_tuple_struct(ast, r#struct)
            } else {
                Error::new(ast.span(), "`FromJson` cannot be derived for empty structs")
                    .to_compile_error()
                    .into()
            }
        }

        Data::Enum(r#enum) => {
            if !r#enum
                .variants
                .iter()
                .any(|variant| !variant.fields.is_empty())
            {
                enum_type::from_json_enum(ast, r#enum)
            } else {
                Error::new(
                    ast.span(),
                    "`FromJson` cannot be derived for enums with data variants",
                )
                .to_compile_error()
                .into()
            }
        }

        _ => Error::new(
            ast.span(),
            "`FromJson` can only be derived for non-empty structs and enums",
        )
        .to_compile_error()
        .into(),
    }
}

/// Derives the `IntoJson` trait for a type.
///
/// This macro can be used on named structs, tuple structs, and enums. It is not currently supported for enums with data variants.
#[proc_macro_derive(IntoJson, attributes(rename))]
pub fn derive_into_json(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    match &ast.data.clone() {
        Data::Struct(r#struct) => {
            if r#struct
                .fields
                .iter()
                .map(|field| field.ident.clone())
                .all(|ident| ident.is_some())
                && !r#struct.fields.is_empty()
            {
                named_struct::into_json_named_struct(ast, r#struct)
            } else if !r#struct.fields.is_empty() {
                tuple_struct::into_json_tuple_struct(ast, r#struct)
            } else {
                Error::new(ast.span(), "`IntoJson` cannot be derived for empty structs")
                    .to_compile_error()
                    .into()
            }
        }

        Data::Enum(r#enum) => {
            if !r#enum
                .variants
                .iter()
                .any(|variant| !variant.fields.is_empty())
            {
                enum_type::into_json_enum(ast, r#enum)
            } else {
                Error::new(
                    ast.span(),
                    "`IntoJson` cannot be derived for enums with data variants",
                )
                .to_compile_error()
                .into()
            }
        }

        _ => Error::new(
            ast.span(),
            "`IntoJson` can only be derived for non-empty structs and enums",
        )
        .to_compile_error()
        .into(),
    }
}
