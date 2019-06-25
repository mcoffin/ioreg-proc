extern crate proc_macro;
extern crate syn;

use std::convert::TryFrom;
use proc_macro::{TokenStream, TokenTree};
use syn::{parse_macro_input, braced, parenthesized, token, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

struct IoRegs {
    name: syn::Ident,
    location_token: Token![@],
    location: syn::LitInt,
    equals_token: Token![=],
    brace_token: token::Brace,
    registers: Punctuated<Register, Token![,]>,
}

impl Parse for IoRegs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(IoRegs {
            name: input.parse()?,
            location_token: input.parse()?,
            location: input.parse()?,
            equals_token: input.parse()?,
            brace_token: braced!(content in input),
            registers: content.parse_terminated(Register::parse)?,
        })
    }
}

#[derive(Debug)]
enum RegisterType {
    Reg32,
}

impl Parse for RegisterType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: syn::Ident = input.parse()?;
        match ty.to_string().as_ref() {
            "reg32" => Ok(RegisterType::Reg32),
            _ => Err(syn::Error::new(ty.span(), format!("Invalid ioregs register type: {}", &ty))),
        }
    }
}

struct Register {
    offset: syn::LitInt,
    arrow_token: Token![=>],
    ty: RegisterType,
    ident: syn::Ident,
    brace_token: token::Brace,
    fields: Punctuated<RegisterField, Token![,]>,
}

impl Parse for Register {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Register {
            offset: input.parse()?,
            arrow_token: input.parse()?,
            ty: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(RegisterField::parse)?,
        })
    }
}

enum RegisterFieldOffset {
    Bit(syn::LitInt),
    BitRange(syn::ExprRange),
}

impl Parse for RegisterFieldOffset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: improve the error messages that this would generate to indicate all options
        if input.fork().parse::<syn::ExprRange>().is_ok() {
            Ok(RegisterFieldOffset::BitRange(input.parse()?))
        } else {
            Ok(RegisterFieldOffset::Bit(input.parse()?))
        }
    }
}

enum RegisterProperty {
    SetToClear,
    WriteOnly,
    ReadOnly,
}

impl Parse for RegisterProperty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_ref() {
            "set_to_clear" => Ok(RegisterProperty::SetToClear),
            "wo" => Ok(RegisterProperty::WriteOnly),
            "ro" => Ok(RegisterProperty::ReadOnly),
            _ => Err(syn::Error::new(ident.span(), format!("Invalid ioregs register property: {}", ident))),
        }
    }
}

struct RegisterProperties {
    colon_token: Token![:],
    paren_token: token::Paren,
    properties: Punctuated<RegisterProperty, Token![,]>,
}

fn parse_optional_register_properties(input: ParseStream) -> syn::Result<Option<RegisterProperties>> {
    let has_colon = input.peek(Token![:]);
    if !has_colon {
        return Ok(None);
    }
    let content;
    Ok(Some(RegisterProperties {
        colon_token: input.parse()?,
        paren_token: parenthesized!(content in input),
        properties: content.parse_terminated(RegisterProperty::parse)?,
    }))
}

struct RegisterVariant {
    value: syn::LitInt,
    arrow_token: Token![=>],
    ident: syn::Ident,
}

impl Parse for RegisterVariant {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RegisterVariant {
            value: input.parse()?,
            arrow_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

struct RegisterVariants {
    brace_token: token::Brace,
    variants: Punctuated<RegisterVariant, Token![,]>,
}

fn parse_optional_register_variants(input: ParseStream) -> syn::Result<Option<RegisterVariants>> {
    let has_brace = input.peek(token::Brace);
    if !has_brace {
        return Ok(None)
    }
    let content;
    Ok(Some(RegisterVariants {
        brace_token: braced!(content in input),
        variants: content.parse_terminated(RegisterVariant::parse)?,
    }))
}

struct RegisterField {
    offset: RegisterFieldOffset,
    arrow_token: Token![=>],
    ident: syn::Ident,
    variants: Option<RegisterVariants>,
    properties: Option<RegisterProperties>,
}

impl Parse for RegisterField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RegisterField {
            offset: input.parse()?,
            arrow_token: input.parse()?,
            ident: input.parse()?,
            variants: input.call(parse_optional_register_variants)?,
            properties: input.call(parse_optional_register_properties)?,
        })
    }
}

#[proc_macro]
pub fn ioregs(item: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(item as IoRegs);
    TokenStream::new()
}
