use super::{IoRegs, Register, RegisterType, RegisterField, RegisterFieldOffset};
use quote::{ ToTokens, quote };

pub trait SynBuilder {
    type Output: ToTokens;

    fn build(&self) -> syn::Result<Self::Output>;
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

impl SynBuilder for Register {
    type Output = syn::export::TokenStream2;

    fn build(&self) -> syn::Result<Self::Output> {
        let ty = register_type(self.ty);
        let ident = &self.ident;
        Ok(quote! {
            pub struct #ident(#ty);
        })
    }
}
