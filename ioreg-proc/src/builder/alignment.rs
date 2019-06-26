use quote::{quote, ToTokens};

pub struct AlignmentTest {
    pub test_ident: syn::Ident,
    pub struct_ident: syn::Ident,
    pub field_ident: syn::Ident,
    pub field_ty: syn::Path,
    pub field_offset: syn::LitInt,
}

impl ToTokens for AlignmentTest {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let test_ident = &self.test_ident;
        let struct_ident = &self.struct_ident;
        let field_ident = &self.field_ident;
        let field_ty = &self.field_ty;
        let field_offset = &self.field_offset;
        tokens.extend(quote! {
            #[cfg(test)]
            #[test]
            fn #test_ident() {
                let test: #struct_ident = unsafe { core::mem::zeroed() };
                let base = &test as *const #struct_ident;
                let location = &test.#field_ident as *const #field_ty;
                assert_eq!((location as usize) - (base as usize), #field_offset);
            }
        });
    }
}
