use heck::{CamelCase, SnakeCase};

pub trait ToCasing {
    fn to_snake_case(&self) -> Self;
    fn to_camel_case(&self) -> Self;
}

impl ToCasing for syn::Ident {
    fn to_snake_case(&self) -> Self {
        let s = <str as SnakeCase>::to_snake_case(self.to_string().as_ref());
        syn::Ident::new(&s, self.span())
    }

    fn to_camel_case(&self) -> Self {
        let s = <str as CamelCase>::to_camel_case(self.to_string().as_ref());
        syn::Ident::new(&s, self.span())
    }
}
