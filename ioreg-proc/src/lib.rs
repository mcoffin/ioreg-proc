#![recursion_limit="128"]
#![allow(dead_code)]

extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate proc_macro2;
extern crate heck;

mod builder;
pub(crate) mod util;

use proc_macro::TokenStream;
use syn::{parse_macro_input, braced, parenthesized, token, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use std::iter;
use quote::{ToTokens, quote};
use util::ParseOptional;
pub(crate) use util::LitVecSize;

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
    pub(crate) name: syn::Ident,
    pub(crate) location_token: Token![@],
    pub(crate) location: syn::LitInt,
    pub(crate) equals_token: Token![=],
    pub(crate) brace_token: token::Brace,
    pub(crate) registers: Punctuated<RegisterOrGroup, Token![,]>,
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

impl RegisterType {
    fn byte_length(self) -> u64 {
        use RegisterType::*;
        match self {
            Reg8 => 1,
            Reg16 => 2,
            Reg32 => 4,
            Reg64 => 8,
        }
    }
}

impl ToTokens for RegisterType {
    fn to_tokens(&self, output: &mut proc_macro2::TokenStream) {
        use RegisterType::*;
        let tokens = match *self {
            Reg8 => quote!(u8),
            Reg16 => quote!(u16),
            Reg32 => quote!(u32),
            Reg64 => quote!(u64),
        };
        output.extend(tokens);
    }
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

pub(crate) enum RegisterOrGroup {
    Single(Register),
    Group(RegisterGroup),
}

impl RegisterOrGroup {
    #[inline]
    pub(crate) fn byte_length(&self) -> u64 {
        match self {
            &RegisterOrGroup::Single(ref reg) => reg.byte_length(),
            &RegisterOrGroup::Group(ref group) => group.byte_length(),
        }
    }
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

pub(crate) struct RegisterGroup {
    pub(crate) offset: syn::LitInt,
    pub(crate) arrow_token: Token![=>],
    pub(crate) group_ident: syn::Ident,
    pub(crate) ident: syn::Ident,
    pub(crate) count: Option<LitVecSize>,
    pub(crate) brace_token: token::Brace,
    pub(crate) members: Punctuated<RegisterOrGroup, Token![,]>,
}

impl RegisterGroup {
    pub(crate) fn count_value(&self) -> u64 {
        self.count
            .as_ref()
            .map(|c| c.value())
            .unwrap_or(1)
    }

    pub(crate) fn byte_length(&self) -> u64 {
        let single_size: u64 = self.members
            .iter()
            .map(|m| m.byte_length())
            .sum();
        single_size * self.count_value()
    }
}

impl Parse for RegisterGroup {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let brace_content;
        let ret = RegisterGroup {
            offset: input.parse()?,
            arrow_token: input.parse()?,
            group_ident: input.call(|s| parse_exact_ident(s, "group"))?,
            ident: input.parse()?,
            count: input.call(ParseOptional::parse_optional)?,
            brace_token: braced!(brace_content in input),
            members: brace_content.parse_terminated(RegisterOrGroup::parse)?,
        };
        Ok(ret)
    }
}

struct Register {
    offset: syn::LitInt,
    arrow_token: Token![=>],
    ty: RegisterType,
    ident: syn::Ident,
    count: Option<LitVecSize>,
    brace_token: token::Brace,
    fields: Punctuated<RegisterField, Token![,]>,
}

impl Register {
    pub(crate) fn count_value(&self) -> u64 {
        self.count
            .as_ref()
            .map(|c| c.value())
            .unwrap_or(1)
    }

    pub(crate) fn byte_length(&self) -> u64 {
        self.ty.byte_length() * self.count_value()
    }
}

impl Parse for Register {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Register {
            offset: input.parse()?,
            arrow_token: input.parse()?,
            ty: input.parse()?,
            ident: input.parse()?,
            count: input.call(ParseOptional::parse_optional)?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(RegisterField::parse)?,
        })
    }
}

#[derive(Clone)]
pub(crate) struct LitIntRange {
    pub(crate) start: syn::LitInt,
    pub(crate) range_sep: Token![..],
    pub(crate) end: syn::LitInt,
}

impl ToTokens for LitIntRange {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.start.to_tokens(tokens);
        self.range_sep.to_tokens(tokens);
        self.end.to_tokens(tokens);
    }
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

#[derive(Clone)]
enum RegisterFieldOffset {
    Bit(syn::LitInt),
    BitRange(LitIntRange),
}

impl ToTokens for RegisterFieldOffset {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use RegisterFieldOffset::*;
        match self {
            Bit(ref v) => v.to_tokens(tokens),
            BitRange(ref range) => range.to_tokens(tokens),
        }
    }
}

impl RegisterFieldOffset {
    pub(crate) fn bit_size(&self) -> u64 {
        match self {
            &RegisterFieldOffset::Bit(..) => 1,
            &RegisterFieldOffset::BitRange(ref range) => range.bit_size(),
        }
    }

    pub(crate) fn span(&self) -> proc_macro2::Span {
        match self {
            &RegisterFieldOffset::Bit(ref v) => v.span(),
            &RegisterFieldOffset::BitRange(ref range) => range.span(),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegisterPropertyValue {
    SetToClear,
    WriteOnly,
    ReadOnly,
    ReadWrite
}

impl RegisterPropertyValue {
    fn is_access_modifier(self) -> bool {
        use RegisterPropertyValue::*;
        match self {
            WriteOnly => true,
            ReadOnly => true,
            ReadWrite => true,
            _ => false,
        }
    }
}

struct RegisterProperty {
    value: RegisterPropertyValue,
    span: proc_macro2::Span,
}

impl RegisterProperty {
    #[inline(always)]
    fn is_access_modifier(&self) -> bool {
        self.value.is_access_modifier()
    }
}

impl Parse for RegisterProperty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        use RegisterPropertyValue::*;
        let value = match ident.to_string().as_ref() {
            "set_to_clear" => Ok(SetToClear),
            "wo" => Ok(WriteOnly),
            "ro" => Ok(ReadOnly),
            "rw" => Ok(ReadWrite),
            _ => Err(syn::Error::new(ident.span(), format!("Invalid ioregs register property: {}", ident))),
        };
        value.map(|v| RegisterProperty {
            value: v,
            span: ident.span(),
        })
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
        let ret = RegisterPropertyList::Multiple {
            paren_token: paren_token,
            properties: properties,
        };
        ret.validate()?;
        Ok(ret)
    }

    fn span(&self) -> proc_macro2::Span {
        // TODO: improve span handling for Multiple case
        match self {
            &RegisterPropertyList::Single(ref prop) => prop.span,
            &RegisterPropertyList::Multiple { ref paren_token, .. } => paren_token.span,
        }
    }

    fn validate(&self) -> syn::Result<()> {
        let access_modifiers = self.iter()
            .filter(|&prop| prop.is_access_modifier())
            .count();
        if access_modifiers > 1 {
            return Err(syn::Error::new(self.span(), format!("more than one access modifier found for register field")));
        }
        let set_to_clear_conflicts = self.iter()
            .filter(|&prop| prop.value == RegisterPropertyValue::SetToClear || prop.value == RegisterPropertyValue::ReadOnly)
            .count();
        if set_to_clear_conflicts >= 2 {
            return Err(syn::Error::new(self.span(), format!("set_to_clear and ro cannot be set on the same register field")));
        }
        Ok(())
    }

    pub(crate) fn iter<'a>(&'a self) -> Box<Iterator<Item=&'a RegisterProperty> + 'a> {
        match self {
            &RegisterPropertyList::Single(ref prop) => Box::new(iter::once(prop)),
            &RegisterPropertyList::Multiple { ref properties, .. } => Box::new(properties.iter()),
        }
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

pub(crate) struct RegisterVariant {
    pub(crate) value: syn::LitInt,
    arrow_token: Token![=>],
    pub(crate) ident: syn::Ident,
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
    let input = parse_macro_input!(item as IoRegs);
    let output = builder::union::build_union(&input)
        .expect("failed to build union");
    TokenStream::from(output)
}
