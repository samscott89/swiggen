#![crate_type = "proc-macro"]
#![feature(proc_macro)]
#![feature(proc_macro_lib)]

/// Procedural macros to generate `extern "C"` functions and SWIG wrapper code
/// from Rust code.

extern crate cbindgen;
#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;
extern crate swiggen;

use proc_macro::TokenStream;

/// Generate SWIG wrapper code for a struct, to handle freeing of the memory
/// on destruction.
/// Uses the `#[swig_derive(Foo)]` attribute to also derive these methods
/// in SWIG. (Currently only `Default` is supported).
#[proc_macro_derive(Swig, attributes(swig_derive))]
pub fn swig_it(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    swiggen::impl_extern_it(&ast).into()
}

/// Convert a Rust method into an `extern "C"` definition with SWIG wrapping
/// code. If this is used on a method inside an impl block, an additional
/// parameter needs to be entered like `#[swiggen(Foo)]` to give the context.
/// Currently, the `swiggen_hack` macro needs to also wrap the impl block
/// to make it work
#[proc_macro_attribute]
pub fn swiggen(arg: TokenStream, input: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(input).unwrap();
    // Parses the arg `(Foo)` as `Some(Foo)`.
    let arg: swiggen::Args = syn::parse(arg).unwrap();
    let base_name: Option<syn::Ident> = arg.0;

    let new_meth = swiggen::impl_extern_fn(&base_name, &ast);
    // When there is a base name, we rely on the `swiggen_hack`
    // to put the tokens in the right place later.
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

/// Proc macro to be used on an impl block so that any `#[swiggen]` function
/// can generate the extern code outside of the impl block.
#[proc_macro]
pub fn swiggen_hack(input: TokenStream) -> TokenStream {
    let ast: syn::ItemImpl = syn::parse(input).unwrap();
    swiggen::split_out_externs(&ast).into()
}