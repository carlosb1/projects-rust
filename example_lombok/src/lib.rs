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
    let name = ast.ident;
    println!("name -> {:?}", name); 
    TokenStream::new() 
}
