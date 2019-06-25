#![allow(dead_code)]

extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate proc_macro2;

mod builder;

use proc_macro::TokenStream;
use syn::{parse_macro_input, braced, bracketed, parenthesized, token, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

fn parse_exact_ident<S: AsRef<str>>(input: ParseStream, value: S) -> syn::Result<syn::Ident> {
    let value = value.as_ref();
    input.parse()
        .and_then(|ident: syn::Ident| if &ident.to_string() == value {
            Ok(ident)
        } else {
            Err(syn::Error::new(ident.span(), format!("expected {}", value)))
        })
}

pub(crate) struct IoRegs {
    name: syn::Ident,
    location_token: Token![@],
    location: syn::LitInt,
    equals_token: Token![=],
    brace_token: token::Brace,
    registers: Punctuated<RegisterOrGroup, Token![,]>,
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
            registers: content.parse_terminated(RegisterOrGroup::parse)?,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum RegisterType {
    Reg8,
    Reg16,
    Reg32,
    Reg64,
}

impl Parse for RegisterType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: syn::Ident = input.parse()?;
        match ty.to_string().as_ref() {
            "reg8" => Ok(RegisterType::Reg8),
            "reg16" => Ok(RegisterType::Reg16),
            "reg32" => Ok(RegisterType::Reg32),
            "reg64" => Ok(RegisterType::Reg64),
            _ => Err(syn::Error::new(ty.span(), format!("Invalid ioregs register type: {}", &ty))),
        }
    }
}

enum RegisterOrGroup {
    Single(Register),
    Group(RegisterGroup),
}

impl Parse for RegisterOrGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: improve the error messages that this would generate to indicate all options
        if input.fork().parse::<RegisterGroup>().is_ok() {
            Ok(RegisterOrGroup::Group(input.parse()?))
        } else {
            Ok(RegisterOrGroup::Single(input.parse()?))
        }
    }
}

struct RegisterGroup {
    offset: syn::LitInt,
    arrow_token: Token![=>],
    group_ident: syn::Ident,
    ident: syn::Ident,
    bracket_token: token::Bracket,
    count: syn::LitInt,
    brace_token: token::Brace,
    fields: Punctuated<Register, Token![,]>,
}

impl Parse for RegisterGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let bracket_content;
        let brace_content;
        Ok(RegisterGroup {
            offset: input.parse()?,
            arrow_token: input.parse()?,
            group_ident: input.call(|s| parse_exact_ident(s, "group"))?,
            ident: input.parse()?,
            bracket_token: bracketed!(bracket_content in input),
            count: bracket_content.parse()?,
            brace_token: braced!(brace_content in input),
            fields: brace_content.parse_terminated(Register::parse)?,
        })
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

pub(crate) struct LitIntRange {
    pub(crate) start: syn::LitInt,
    pub(crate) range_sep: Token![..],
    pub(crate) end: syn::LitInt,
}

impl LitIntRange {
    pub(crate) fn bit_size(&self) -> u64 {
        self.end.value() - self.start.value() + 1
    }

    pub(crate) fn span(&self) -> proc_macro2::Span {
        // TODO: nightly could provide better support here
        self.start.span()
    }
}

impl Parse for LitIntRange {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(LitIntRange {
            start: input.parse()?,
            range_sep: input.parse()?,
            end: input.parse()?,
        })
    }
}

enum RegisterFieldOffset {
    Bit(syn::LitInt),
    BitRange(LitIntRange),
}

impl Parse for RegisterFieldOffset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: improve the error messages that this would generate to indicate all options
        if input.fork().parse::<LitIntRange>().is_ok() {
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
    ReadWrite
}

impl Parse for RegisterProperty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_ref() {
            "set_to_clear" => Ok(RegisterProperty::SetToClear),
            "wo" => Ok(RegisterProperty::WriteOnly),
            "ro" => Ok(RegisterProperty::ReadOnly),
            "rw" => Ok(RegisterProperty::ReadWrite),
            _ => Err(syn::Error::new(ident.span(), format!("Invalid ioregs register property: {}", ident))),
        }
    }
}

enum RegisterPropertyList {
    Single(RegisterProperty),
    Multiple {
        paren_token: token::Paren,
        properties: Punctuated<RegisterProperty, Token![,]>,
    }
}

impl RegisterPropertyList {
    fn parse_multiple(input: ParseStream) -> syn::Result<RegisterPropertyList> {
        let content: syn::parse::ParseBuffer<'_>;
        let paren_token: token::Paren = parenthesized!(content in input);
        let properties: Punctuated<RegisterProperty, Token![,]> =
            content.parse_terminated(RegisterProperty::parse)?;
        Ok(RegisterPropertyList::Multiple {
            paren_token: paren_token,
            properties: properties,
        })
    }
}

impl Parse for RegisterPropertyList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let has_paren = input.peek(token::Paren);
        if has_paren {
            RegisterPropertyList::parse_multiple(input)
        } else {
            Ok(RegisterPropertyList::Single(input.parse()?))
        }
    }
}

struct RegisterProperties {
    colon_token: Token![:],
    properties: RegisterPropertyList,
}

fn parse_optional_register_properties(input: ParseStream) -> syn::Result<Option<RegisterProperties>> {
    let has_colon = input.peek(Token![:]);
    if !has_colon {
        return Ok(None);
    }
    Ok(Some(RegisterProperties {
        colon_token: input.parse()?,
        properties: input.parse()?,
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
