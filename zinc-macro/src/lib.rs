// Zinc, the bare metal stack for rust.
// Copyright 2014 Matt "mcoffin" Coffin <mcoffin13@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn zinc_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let main_ident = &input.ident;
    let start = quote! {
        #[start]
        fn start(_: isize, _: *const *const u8) -> isize {
            #main_ident();
            0
        }
    };
    TokenStream::from(quote! {
        #input
        #start
    })
}
