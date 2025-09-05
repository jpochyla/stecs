extern crate proc_macro;

use std::fmt::Write;
use std::str::FromStr;

use myn::prelude::*;
use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

#[proc_macro_derive(Mosaic)]
pub fn derive_mosaic(input: TokenStream) -> TokenStream {
    let Struct {
        struct_name,
        struct_fields,
    } = match Struct::parse(input) {
        Ok(ast) => ast,
        Err(err) => return err,
    };

    let mut fields = String::new();
    for Field {
        field_name,
        field_ref,
    } in struct_fields
    {
        match field_ref {
            FieldRef::Ref => {
                let _ = write!(
                    fields,
                    r#"
                    {field_name}: &$obj.{field_name},
                    "#
                );
            }
            FieldRef::Mut => {
                let _ = write!(
                    fields,
                    r#"
                    {field_name}: &mut $obj.{field_name},
                    "#
                );
            }
            FieldRef::Type(ident) => {
                let _ = write!(
                    fields,
                    r#"
                    {field_name}: {ident}!($obj, {field_name}),
                    "#
                );
            }
        }
    }

    let code = format!(
        r#"
        macro_rules! {struct_name} {{
            ($obj:ident $(, $_field:ident)*) => {{
                {struct_name} {{
                    {fields}
                }}
            }};
        }}
        "#
    );

    match TokenStream::from_str(&code) {
        Ok(stream) => stream,
        Err(err) => spanned_error(err.to_string(), Span::call_site()),
    }
}

struct Field {
    field_name: Ident,
    field_ref: FieldRef,
}

struct Struct {
    struct_name: Ident,
    struct_fields: Vec<Field>,
}

impl Struct {
    fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        let mut input = input.into_token_iter();
        input.parse_attributes()?;
        input.parse_visibility()?;
        input.expect_ident("struct")?;

        let name = input.try_ident()?;
        let _ = input.parse_path(); // Skip any possible type params.
        let content = input.expect_group(Delimiter::Brace)?;
        let fields = Self::parse_fields(content)?;

        let this = Self {
            struct_name: name,
            struct_fields: fields,
        };
        Ok(this)
    }

    fn parse_fields(mut input: TokenIter) -> Result<Vec<Field>, TokenStream> {
        let mut args = Vec::new();

        while input.peek().is_some() {
            input.parse_visibility()?;
            let name = input.try_ident()?;
            input.expect_punct(':')?;

            let is_ref = input.next_if(|t| is_punct(t, '&')).is_some();

            let field_ref = if is_ref {
                let has_lifetime = input.next_if(|t| is_punct(t, '\'')).is_some();
                if has_lifetime {
                    let _ = input.try_ident()?;
                }
                let is_mut = input.next_if(|t| is_ident(t, "mut")).is_some();
                let _ = input.try_ident()?;

                if is_mut {
                    FieldRef::Mut
                } else {
                    FieldRef::Ref
                }
            } else {
                let ty = input.try_ident()?;

                FieldRef::Type(ty)
            };

            let _ = input.parse_path(); // Skip the type remainder.
            let _ = input.expect_punct(',');

            let field = Field {
                field_name: name,
                field_ref,
            };
            args.push(field);
        }

        Ok(args)
    }
}

enum FieldRef {
    Ref,
    Mut,
    Type(Ident),
}

fn is_punct(token: &TokenTree, char: char) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == char)
}

fn is_ident(token: &TokenTree, name: &str) -> bool {
    matches!(token, TokenTree::Ident(ident) if ident.to_string() == name)
}
