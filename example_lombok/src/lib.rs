extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(GetterSetter)]
pub fn getter_setter_derive(input: TokenStream) -> TokenStream{
    println!("input -> {:?}", input.to_string().clone());
    
    let cloned = input.clone();
    let ast = parse_macro_input!(cloned as DeriveInput);
    let name = ast.ident.clone();
    println!("name -> {:?}", name); 
    let result = impl_getter_setter_macro(&ast);
    result.into()
}

fn impl_getter_setter_macro(ast: &syn::DeriveInput) -> syn::export::TokenStream2 {
    /*
    let idents: Vec<Ident> = match ast.body {
        syn::Body::Struct(vdata) => {
            match vdata {
            
            }
        },
        syn::Body::Enum(_) => panic!("You can only derive this on structs!"),
    }
    */
    
    let name = &ast.ident;
    let result = quote!{
        impl #name {
            fn helloworld(&self) {
                println!("Hello world!! {}", stringify!(#name));
            }
        }
    };
    result
}

