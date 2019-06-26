use crate::{IoRegs, RegisterOrGroup, Register};
use std::collections::LinkedList;
use quote::quote;
use super::RegisterExt;
use super::casing::ToCasing;

pub(crate) fn build_union(union: &IoRegs) -> syn::Result<proc_macro2::TokenStream> {
    let union_ident = union.name.to_camel_case();
    let mod_ident = union.name.to_snake_case();
    let mut field_definitions = LinkedList::new();
    let mut register_definitions = LinkedList::new();
    let mut offset: usize = 0;
    let mut padding_count: usize = 0;
    println!("build_union: {}::{}", mod_ident, union_ident);
    for reg_or_group in union.registers.iter() {
        match reg_or_group {
            &RegisterOrGroup::Single(ref reg) => {
                if (reg.byte_start() as usize) != offset {
                    use syn::IntSuffix;
                    let padding_ident_s = format!("_padding{}", padding_count);
                    padding_count += 1;
                    let padding_ident = syn::Ident::new(&padding_ident_s, reg.offset.span());
                    let padding_size = syn::LitInt::new(reg.byte_start() - (offset as u64), IntSuffix::None, reg.offset.span());
                    field_definitions.push_back(quote! {
                        #padding_ident: [u8; #padding_size]
                    });
                    offset += padding_size.value() as usize;
                }
                offset += reg.ty.byte_length() as usize;
                let (idents, struct_definition) = super::build_register_struct(reg)?;
                let reg_ident = &reg.ident;
                let reg_ty = &idents.base;
                let field_definition = quote! {
                    pub #reg_ident: #reg_ty
                };
                field_definitions.push_back(field_definition);
                register_definitions.push_back(struct_definition);
            },
            &RegisterOrGroup::Group(ref group) => unimplemented!(),
        }
    }
    Ok(quote! {
        pub mod #mod_ident {
            #( #register_definitions )*
            pub struct #union_ident {
                #( #field_definitions ),*
            }
        }
    })
}
