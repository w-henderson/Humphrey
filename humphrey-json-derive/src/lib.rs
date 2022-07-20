extern crate proc_macro;

mod enum_type;
mod named_struct;
mod tuple_struct;

use proc_macro::TokenStream;

use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Error};

#[proc_macro_derive(FromJson)]
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

#[proc_macro_derive(IntoJson)]
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
