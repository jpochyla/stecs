extern crate proc_macro;

use std::fmt::Write;
use std::str::FromStr;

use myn::prelude::*;
use proc_macro::{Delimiter, Ident, Span, TokenStream};

#[proc_macro_derive(Lend)]
pub fn derive_lend(input: TokenStream) -> TokenStream {
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
        field_type,
    } in struct_fields
    {
        let _ = write!(
            fields,
            r#"
            {field_name}: {field_type}!($obj, {field_name}),
            "#
        );
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
    field_type: Ident,
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
            let ty = input.try_ident()?;
            let _ = input.parse_path(); // Skip any possible type params.
            let _ = input.expect_punct(',');

            let field = Field {
                field_name: name,
                field_type: ty,
            };
            args.push(field);
        }

        Ok(args)
    }
}
