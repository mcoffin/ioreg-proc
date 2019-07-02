use crate::{IoRegs, RegisterOrGroup, Register, RegisterGroup, LitVecSize};
use std::collections::LinkedList;
use quote::{quote, ToTokens};
use super::RegisterExt;
use super::casing::ToCasing;

use super::alignment::AlignmentTest;

struct UnionBuilder {
    field_definitions: LinkedList<proc_macro2::TokenStream>,
    register_definitions: LinkedList<proc_macro2::TokenStream>,
    alignment_tests: LinkedList<AlignmentTest>,
    union_ident: syn::Ident,
    mod_ident: syn::Ident,
    offset: usize,
    padding_count: usize,
}

impl UnionBuilder {
    fn new(union: &IoRegs) -> Self {
        UnionBuilder {
            field_definitions: LinkedList::new(),
            register_definitions: LinkedList::new(),
            alignment_tests: LinkedList::new(),
            union_ident: union.name.to_camel_case(),
            mod_ident: union.name.to_snake_case(),
            offset: 0,
            padding_count: 0,
        }
    }

    fn from_group(group: &RegisterGroup) -> Self {
        UnionBuilder {
            field_definitions: LinkedList::new(),
            register_definitions: LinkedList::new(),
            alignment_tests: LinkedList::new(),
            union_ident: group.ident.to_camel_case(),
            mod_ident: group.ident.to_snake_case(),
            offset: 0,
            padding_count: 0,
        }
    }

    fn ty_path(&self) -> syn::Path {
        let mod_ident = &self.mod_ident;
        let union_ident = &self.union_ident;

        // This should never fail, so the unwrap should be Ok
        syn::parse2(quote!(#mod_ident::#union_ident))
            .unwrap()
    }

    fn add_register_group(&mut self, group: &RegisterGroup) -> syn::Result<&mut Self> {
        let group_ident = group.ident.to_snake_case();
        let mut builder = UnionBuilder::from_group(group);
        for member in &group.members {
            match member {
                &RegisterOrGroup::Single(ref reg) => builder.add_register(reg).map(|_| ())?,
                &RegisterOrGroup::Group(ref group) => builder.add_register_group(group).map(|_| ())?,
            }
        }
        self.advance_to_offset(group.offset.value() as usize, || group.offset.span());
        let group_ty = builder.ty_path();
        let group_ty = &group_ty;
        let group_ident = &group_ident;
        let field_ty = repeated_type(group_ty.clone().into_token_stream(), group.count.clone());
        let field_definition = quote! {
            pub #group_ident: #field_ty
        };
        self.offset += group.byte_length() as usize;
        #[cfg(feature = "alignment_tests")]
        {
            let test_ident = syn::Ident::new(&format!("test_align_{}_{}", &self.mod_ident, &group_ident), group_ident.span());
            self.alignment_tests.push_back(AlignmentTest {
                test_ident: test_ident,
                struct_ident: self.union_ident.clone(),
                field_ident: group_ident.clone(),
                field_ty: group_ty.clone(),
                field_offset: group.offset.clone(),
            });
        }
        self.field_definitions.push_back(field_definition);
        self.register_definitions.push_back(builder.into_token_stream());
        Ok(self)
    }

    fn advance_to_offset<F>(&mut self, offset: usize, get_span: F) where
        F: FnOnce() -> proc_macro2::Span,
    {
        use syn::IntSuffix;

        if offset == self.offset {
            return;
        }

        let padding_ident_s = format!("_padding{}", self.padding_count);
        self.padding_count += 1;
        let span = get_span();
        let padding_ident = syn::Ident::new(&padding_ident_s, span.clone());
        let padding_size = syn::LitInt::new((offset - self.offset) as u64, IntSuffix::None, span);

        self.field_definitions.push_back(quote! {
            #padding_ident: [u8; #padding_size]
        });
        self.offset += padding_size.value() as usize;
    }

    fn add_register(&mut self, reg: &Register) -> syn::Result<&mut Self> {
        self.advance_to_offset(reg.byte_start() as usize, || reg.offset.span());
        let (idents, struct_definition) = super::build_register_struct(reg)?;
        let reg_ident = &reg.ident;
        let reg_ty = &idents.base;
        let field_ty = repeated_type(reg_ty.clone().into_token_stream(), reg.count.clone());
        let field_definition = quote! {
            pub #reg_ident: #field_ty
        };
        self.offset += reg.byte_length() as usize;
        #[cfg(feature = "alignment_tests")]
        {
            let test_ident = syn::Ident::new(&format!("test_align_{}_{}", &self.mod_ident, reg_ident), reg_ident.span());
            let reg_ty = syn::parse2(reg_ty.clone().into_token_stream())?;
            self.alignment_tests.push_back(AlignmentTest {
                test_ident: test_ident,
                struct_ident: self.union_ident.clone(),
                field_ident: reg_ident.clone(),
                field_ty: reg_ty,
                field_offset: reg.offset.clone(),
            });
        }
        self.field_definitions.push_back(field_definition);
        self.register_definitions.push_back(struct_definition);
        Ok(self)
    }
}

impl ToTokens for UnionBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mod_ident = &self.mod_ident;
        let union_ident = &self.union_ident;
        let register_definitions = self.register_definitions.iter();
        let field_definitions = self.field_definitions.iter();
        let tests = self.alignment_tests.iter();

        tokens.extend(quote! {
            pub mod #mod_ident {
                #( #register_definitions )*
                #[repr(C)]
                pub struct #union_ident {
                    #( #field_definitions ),*
                }
                #( #tests )*
            }
        });
    }
}

fn repeated_type(ty: proc_macro2::TokenStream, count: Option<LitVecSize>) -> proc_macro2::TokenStream {
    match count {
        Some(size) => {
            let len = &size.count;
            quote!([#ty; #len])
        },
        None => ty,
    }
}

pub(crate) fn build_union(union: &IoRegs) -> syn::Result<proc_macro2::TokenStream> {
    let mut builder = UnionBuilder::new(union);
    for reg_or_group in union.registers.iter() {
        match reg_or_group {
            &RegisterOrGroup::Single(ref reg) => builder.add_register(reg).map(|_| ())?,
            &RegisterOrGroup::Group(ref group) => builder.add_register_group(group).map(|_| ())?,
        }
    }
    Ok(builder.into_token_stream())
}
