#![crate_type = "proc-macro"]
#![feature(proc_macro)]
#![feature(proc_macro_lib)]

extern crate cbindgen;
#[macro_use]
extern crate quote;
extern crate proc_macro;
#[macro_use]
extern crate syn;
extern crate swiggen;

use proc_macro::TokenStream;

#[proc_macro_derive(Swig, attributes(swig_derive))]
pub fn swig_it(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    swiggen::impl_extern_it(&ast).into()
}

#[proc_macro_attribute]
pub fn swiggen(arg: TokenStream, input: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(input).unwrap();
    let arg: swiggen::Args = syn::parse(arg).unwrap();
    let base_name: Option<syn::Ident> = arg.0;

    // ignore this for now, we actually generate the functions later
    // using the `swiggen_hack` proc macro
    let new_meth = swiggen::impl_extern_fn(&base_name, &ast);
    let tokens = if base_name.is_some() {
        quote!{
            #ast
        }
    } else {
        quote!{
            #ast

            #new_meth
        }
    };
    tokens.into()
}

#[proc_macro]
pub fn swiggen_hack(input: TokenStream) -> TokenStream {
    let ast: syn::ItemImpl = syn::parse(input).unwrap();
    swiggen::split_out_externs(&ast).into()
}