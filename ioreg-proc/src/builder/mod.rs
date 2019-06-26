use super::{IoRegs, Register, RegisterType, RegisterField, RegisterFieldOffset, RegisterPropertyValue};
use quote::{ ToTokens, quote };
use std::borrow::Cow;
use std::collections::{LinkedList, HashMap};
use std::iter;

pub mod alignment;
pub mod union;
pub mod casing;

pub(crate) trait RegisterExt {
    fn is_write_only(&self) -> bool;
    fn byte_start(&self) -> u64;
}

impl RegisterExt for Register {
    fn is_write_only(&self) -> bool {
        self.fields.iter().all(|f| {
            f.properties
                .as_ref()
                .and_then(|p| p.properties.iter().map(|p| p.value).find(|&v| v == RegisterPropertyValue::WriteOnly))
                .is_some()
        })
    }

    fn byte_start(&self) -> u64 {
        self.offset.value()
    }
}

trait RegisterFieldExt {
    fn shift_expr(&self) -> &syn::LitInt;
    fn mask_expr(&self) -> syn::LitInt;
    fn primitive_extract_expr<T: ToTokens>(&self, value_expr: &T, ty: RegisterType) -> proc_macro2::TokenStream;
}

impl RegisterFieldExt for RegisterField {
    fn shift_expr(&self) -> &syn::LitInt {
        match &self.offset {
            &RegisterFieldOffset::Bit(ref low) => low,
            &RegisterFieldOffset::BitRange(ref range) => &range.start,
        }
    }

    fn mask_expr(&self) -> syn::LitInt {
        use syn::IntSuffix;
        let span = self.offset.span();
        let value = (1 << self.offset.bit_size() as u64) - 1;
        syn::LitInt::new(value, IntSuffix::None, span)
    }

    #[cfg(not(feature = "x86_64_bmi1_optimization"))]
    fn primitive_extract_expr<T: ToTokens>(&self, value_expr: &T, _ty: RegisterType) -> proc_macro2::TokenStream {
        let shift = self.shift_expr();
        let mask = self.mask_expr();
        quote!((#value_expr >> #shift) & #mask)
    }

    #[cfg(feature = "x86_64_bmi1_optimization")]
    fn primitive_extract_expr<T: ToTokens>(&self, value_expr: &T, ty: RegisterType) -> proc_macro2::TokenStream {
        let shift = self.shift_expr();
        let mask = self.mask_expr();
        let default_implementation = quote!((#value_expr >> #shift) & #mask);
        match ty {
            RegisterType::Reg32 => {
                let start = self.shift_expr();
                let len = syn::LitInt::new(self.offset.bit_size(), syn::IntSuffix::None, self.offset.span());
                quote! {
                    {
                        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
                        {
                            #[cfg(target_feature = "bmi1")]
                            unsafe {
                                #[cfg(target_arch = "x86")]
                                use core::arch::x86::_bextr2_u32;
                                #[cfg(target_arch = "x86_64")]
                                use core::arch::x86_64::_bextr2_u32;

                                _bextr2_u32(#value_expr, Self::bextr_control32(#start, #len))
                            }
                            #[cfg(not(target_feature = "bmi1"))]
                            { #default_implementation }
                        }
                        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                        { #default_implementation }
                    }
                }
            },
            RegisterType::Reg64 => {
                let start = self.shift_expr();
                let len = syn::LitInt::new(self.offset.bit_size(), syn::IntSuffix::None, self.offset.span());
                quote! {
                    {
                        #[cfg(and(target_feature = "bmi1", not(target_arch = "x86")))]
                        unsafe { core::arch::x86_64::_bextr2_u64(#value_expr, Self::bextr_control64(#start, #len)) }
                        #[cfg(not(and(target_feature = "bmi1", not(target_arch = "x86"))))]
                        { #default_implementation }
                    }
                }
            },
            _ => {
                let shift = self.shift_expr();
                let mask = self.mask_expr();
                quote!((#value_expr >> #shift) & #mask)
            },
        }
    }
}

fn register_type(ty: RegisterType) -> impl ToTokens {
    match ty {
        RegisterType::Reg8 => quote!(u8),
        RegisterType::Reg16 => quote!(u16),
        RegisterType::Reg32 => quote!(u32),
        RegisterType::Reg64 => quote!(u64),
    }
}

fn register_field_primitive(field: &RegisterField) -> syn::Result<syn::export::TokenStream2> {
    match &field.offset {
        &RegisterFieldOffset::Bit(..) => Ok(quote!(bool)),
        &RegisterFieldOffset::BitRange(ref range) => {
            let size = range.bit_size();
            if size <= 8 {
                Ok(quote!(u8))
            } else if size <= 16 {
                Ok(quote!(u16))
            } else if size <= 32 {
                Ok(quote!(u32))
            } else if size <= 64 {
                Ok(quote!(u64))
            } else {
                Err(syn::Error::new(range.span(), format!("Invalid register field size: {}", size)))
            }
        },
    }
}

fn camel_case_cow<'a, T: ?Sized>(input: Cow<'a, T>) -> Cow<'a, T> where
    T: heck::CamelCase + ToOwned,
{
    use heck::CamelCase;
    Cow::Owned(input.to_camel_case())
}

fn build_register_field_enum(field: &RegisterField, register_ty: Option<RegisterType>) -> syn::Result<Option<(syn::Ident, syn::export::TokenStream2)>> {
    if let Some(variants) = field.variants.as_ref() {
        let enum_ident = {
            let mut enum_ident = field.ident.to_string().into();
            enum_ident = camel_case_cow(enum_ident);
            syn::Ident::new(enum_ident.as_ref(), field.ident.span())
        };
        let default_primitive = register_field_primitive(field)?;
        let primitive = register_ty
            .map(ToTokens::into_token_stream)
            .unwrap_or(default_primitive);
        let get_variant_idents = || variants.variants.iter().map(|v| &v.ident);
        let variant_idents = get_variant_idents();
        let variant_idents2 = get_variant_idents();
        let get_variant_values = || variants.variants.iter().map(|v| &v.value);
        let variant_values = get_variant_values();
        let variant_values2 = get_variant_values();
        let enum_ident_ref = &enum_ident;
        let enum_ident_rep = iter::repeat(enum_ident_ref);
        #[cfg(test)]
        let derive_expr = quote!(#[derive(Debug, Clone, Copy, PartialEq, Eq)]);
        #[cfg(not(test))]
        let derive_expr = quote!(#[derive(Clone, Copy, PartialEq, Eq)]);
        let definition = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[repr(#primitive)]
            pub enum #enum_ident_ref {
                #( #variant_idents = #variant_values ),*
            }

            impl core::convert::TryFrom<#primitive> for #enum_ident_ref {
                type Error = #primitive;

                #[inline(always)]
                fn try_from(primitive: #primitive) -> Result<Self, Self::Error> {
                    match primitive {
                        #( #variant_values2 => Ok(#enum_ident_rep::#variant_idents2), )*
                        v => Err(v),
                    }
                }
            }
        };
        Ok(Some((enum_ident, definition)))
    } else {
        Ok(None)
    }
}

pub struct RegisterStructIdents {
    pub base: syn::Ident,
    pub update: syn::Ident,
    pub get: syn::Ident,
}

pub(crate) fn build_register_struct(register: &Register) -> syn::Result<(RegisterStructIdents, syn::export::TokenStream2)> {
    let struct_idents = {
        let struct_ident = {
            use heck::CamelCase;
            let mut struct_ident = register.ident.to_string().into();
            struct_ident = camel_case_cow(struct_ident);
            syn::Ident::new(&struct_ident, register.ident.span())
        };
        let struct_ident_str = struct_ident.to_string();
        let update_ident = syn::Ident::new(&format!("{}{}", &struct_ident_str, "Update"), register.ident.span());
        let get_ident = syn::Ident::new(&format!("{}{}", &struct_ident_str, "Get"), register.ident.span());
        RegisterStructIdents {
            base: struct_ident,
            update: update_ident,
            get: get_ident,
        }
    };
    let mod_ident = {
        use heck::SnakeCase;
        let s = <str as SnakeCase>::to_snake_case(register.ident.to_string().as_ref());
        syn::Ident::new(&s, register.ident.span())
    };
    let mut enum_register_definitions = LinkedList::new();
    let mut enum_register_idents = HashMap::new();
    for field in register.fields.iter() {
        if let Some((enum_ident, ts)) = build_register_field_enum(field, Some(register.ty))? {
            let mod_ident = &mod_ident;
            let enum_path: syn::Path = syn::parse2(quote!(#mod_ident::#enum_ident))?;
            enum_register_idents.insert(field.ident.clone(), enum_path);
            enum_register_definitions.push_back(ts);
        }
    }
    let mod_definition = quote! {
        pub mod #mod_ident {
            #( #enum_register_definitions )*
        }
    };
    let register_ty = &register.ty;
    let struct_ident = &struct_idents.base;
    let update_ident = &struct_idents.update;
    let get_ident = &struct_idents.get;
    let struct_definition = quote! {
        pub struct #struct_ident {
            value: ::volatile_cell::VolatileCell<#register_ty>,
        }

        impl #struct_ident {
            #[doc="Create a new updater"]
            #[inline(always)]
            pub fn update<'a>(&'a self) -> #update_ident<'a> {
                #update_ident::new(self)
            }

            #[doc="Create a getter representing the current state of the register"]
            #[inline(always)]
            pub fn get(&self) -> #get_ident {
                #get_ident::new(self)
            }
        }
    };
    let get_function_definitions = register.fields.iter().filter_map(|field| {
        let is_write_only = field.properties
            .as_ref()
            .and_then(|props| {
                use std::iter::Iterator;
                props.properties.iter().find(|&p| p.value == RegisterPropertyValue::ReadOnly)
            })
            .is_some();
        if is_write_only {
            return None;
        }
        let getter_ident = {
            use heck::SnakeCase;
            let s = <str as SnakeCase>::to_snake_case(field.ident.to_string().as_ref());
            syn::Ident::new(&s, field.ident.span())
        };
        let mut is_enum = true;
        let field_ty: Cow<syn::Path> = enum_register_idents.get(&field.ident)
            .map(Cow::Borrowed)
            .map(Ok)
            .unwrap_or_else(|| {
                is_enum = false;
                if field.offset.bit_size() == 1 {
                    register_field_primitive(&field)
                        .and_then(syn::parse2)
                        .map(Cow::Owned)
                } else {
                    syn::parse2(register_ty.into_token_stream())
                        .map(Cow::Owned)
                }
            })
            .unwrap(); // TODO: get rid of this unwrap
        let field_ty = field_ty.as_ref();
        let shift = field.shift_expr();
        let mask = field.mask_expr();
        let primitive_expr = field.primitive_extract_expr(&quote!(self.value), register.ty);
        let value = if field.offset.bit_size() == 1 {
            quote! {
                let val = #primitive_expr;
                val != 0x0
            }
        } else if enum_register_idents.get(&field.ident).is_none() {
            primitive_expr
        } else {
            #[cfg(ioregs_variant_unchecked)]
            {
                quote!(unsafe { core::mem::transmute::<_, #field_ty>(#primitive_expr) })
            }
            #[cfg(not(ioregs_variant_unchecked))]
            {
                quote! {
                    use core::convert::TryFrom;
                    let primitive_value: #register_ty = #primitive_expr;
                    #field_ty::try_from(primitive_value).unwrap()
                }
            }
        };
        let ret = quote! {
            #[inline(always)]
            pub fn #getter_ident(&self) -> #field_ty {
                #value
            }
        };
        Some(ret)
    });
    let update_function_definitions = register.fields.iter().filter_map(|field| {
        use std::borrow::Borrow;
        let is_read_only = field.properties
            .as_ref()
            .and_then(|props| {
                use std::iter::Iterator;
                props.properties.iter().find(|&p| p.value == RegisterPropertyValue::ReadOnly)
            })
            .is_some();
        if is_read_only {
            return None;
        }
        let setter_ident = {
            use heck::SnakeCase;
            let s = <str as SnakeCase>::to_snake_case(field.ident.to_string().as_ref());
            syn::Ident::new(&format!("set_{}", s), field.ident.span())
        };
        let mut is_enum = false;
        let field_ty: Cow<syn::Path> = enum_register_idents.get(&field.ident)
            .map(Cow::Borrowed)
            .map(Ok)
            .unwrap_or_else(|| {
                is_enum = true;
                register_field_primitive(&field)
                    .and_then(syn::parse2)
                    .map(Cow::Owned)
            })
            .unwrap(); // TODO: get rid of this unwrap
        let field_ty = field_ty.as_ref();
        let shift = field.shift_expr();
        let mask = field.mask_expr();
        let register_ty = &register.ty;
        let ret = quote! {
            #[inline(always)]
            pub fn #setter_ident<'b>(&'b mut self, new_value: #field_ty) -> &'b mut Self {
                let context_mask: #register_ty = #mask << #shift;
                self.value = (self.value & !context_mask) | (((new_value as #register_ty) & #mask) << #shift);
                self.mask |= context_mask;
                self
            }
        };
        Some(ret)
    });
    #[cfg(feature = "x86_64_bmi1_optimization")]
    let get_function_definitions = get_function_definitions.chain(iter::once(quote! {
        const fn bextr_control32(start: u32, len: u32) -> u32 {
            (start & 0xff) | ((len & 0xff) << 8)
        }

        const fn bextr_control64(start: u64, len: u64) -> u64 {
            (start & 0xff) | ((len & 0xff) << 8)
        }
    }));
    let get_definition = {
        quote! {
            #[derive(Clone)]
            pub struct #get_ident {
                value: #register_ty,
            }

            impl #get_ident {
                #[doc = "Create a getter reflecting the current value of the register"]
                #[inline(always)]
                pub fn new(reg: & #struct_ident) -> #get_ident {
                    #get_ident {
                        value: reg.value.get(),
                    }
                }

                #( #get_function_definitions )*
            }
        }
    };
    let update_definition = {
        let mut clear: u64 = 0;
        for field in register.fields.iter() {
            if field.properties.as_ref().and_then(|p| p.properties.iter().map(|p| p.value).find(|&value| value == RegisterPropertyValue::SetToClear)).is_some() {
                let mask = field.mask_expr().value();
                clear |= mask << field.shift_expr().value();
            }
        }
        let initial_value = if register.is_write_only() {
            quote!(0)
        } else {
            quote! {
                if self.write_only {
                    0
                } else {
                    self.reg.value.get()
                }
            }
        };
        quote! {
            pub struct #update_ident<'a> {
                value: #register_ty,
                mask: #register_ty,
                write_only: bool,
                reg: &'a #struct_ident,
            }

            impl<'a> #update_ident<'a> {
                #[inline(always)]
                pub fn new(reg: &'a #struct_ident) -> #update_ident<'a> {
                    #update_ident {
                        value: 0,
                        mask: 0,
                        write_only: false,
                        reg: reg,
                    }
                }

                #[inline(always)]
                pub fn new_ignoring_state(reg: &'a #struct_ident) -> #update_ident<'a> {
                    #update_ident {
                        value: 0,
                        mask: 0,
                        write_only: true,
                        reg: reg,
                    }
                }

                const fn clear_mask() -> #register_ty {
                    #clear as #register_ty
                }

                #( #update_function_definitions )*
            }

            impl<'a> Drop for #update_ident<'a> {
                #[inline(always)]
                fn drop(&mut self) {
                    let clear_mask = Self::clear_mask();
                    if self.mask != 0 {
                        let v: #register_ty = #initial_value & (!clear_mask) & (!self.mask);
                        self.reg.value.set(self.value | v);
                    }
                }
            }
        }
    };
    let ret = quote! {
        #mod_definition
        #struct_definition
        #update_definition
        #get_definition
    };
    Ok((struct_idents, ret))
}
