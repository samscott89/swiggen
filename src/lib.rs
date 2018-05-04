#![feature(proc_macro)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;


#[proc_macro_derive(Swig)]
pub fn swig_it(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    // Build the impl
    if let syn::Data::Struct(_) = ast.data {
        let gen = impl_extern_it(&ast);
        gen.into()
    } else {
        panic!("#[derive(Swig)] is only defined for structs, not for enums!");
    }
}




fn impl_extern_it(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let hidden_name = syn::Ident::from(format!("__SWIG_{}", name));
    let default_name = syn::Ident::from(format!("__swig_default_{}", name));
    let free_name = syn::Ident::from(format!("__swig_free_{}", name));
    let comment = format!("#[doc=\"\
__SWIG_CODE 
  class {name} {{ 
        ffi::{name} *self; 
        public:
            {name}() {{ self = {default_name}(); }};
            ~{name}(){{
                {free_name}(self);
                self = NULL;
            }};
            {name}(ffi::{name} *ptr) {{
                self = ptr;
            }};
   }};
__END_SWIG_CODE
__SWIG_HDR_CODE 
  class {name} {{ 
        public:
            {name}();
            ~{name}();
   }};
__END_SWIG_HDR_CODE \
        \"] \
        struct {hidden_name}; \
        ",
        name=name,
        default_name=default_name,
        free_name=free_name,
        hidden_name=hidden_name
        );
    // println!("{}", comment);
    let doc_comment: syn::ItemStruct = syn::parse_str(&comment).unwrap();

    quote! {
        #doc_comment

        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #default_name() -> *mut #name {
            Box::into_raw(Box::new(#name::default()))
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn #free_name(arg: *mut #name) {
            unsafe {
                assert!(!arg.is_null());
                &*arg;
            }
        }
    }
}

#[proc_macro_attribute]
pub fn swiggen(_arg: TokenStream, input: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(input).unwrap();
    let gen = impl_extern_fn(&ast);
    gen.into()
}


fn impl_extern_fn(ast: &syn::ItemFn) -> quote::Tokens {
    let name = &ast.ident;
    let ext_name = syn::Ident::from(format!("__swig_{}", name));
    let hidden_name = syn::Ident::from(format!("__SWIG_{}", name));
    let out = match ast.decl.output {
        syn::ReturnType::Default => unimplemented!(),
        syn::ReturnType::Type(_, ref ty) => ty,
    };

    let comment = format!("#[doc=\"\
__SWIG_CODE 
  {out} {name}() {{
      return {out}({ext_name}());
  }}
__END_SWIG_CODE
__SWIG_HDR_CODE 
    %newobject {name};
    {out} {name}();
__END_SWIG_HDR_CODE\
        \"] \
        fn {hidden_name}(){{}}",
    name=name,
    ext_name=ext_name,
    out=out.into_tokens(),
    hidden_name=hidden_name
    );
    // println!("{}", comment);
    let doc_comment: syn::ItemFn = syn::parse_str(&comment).unwrap();
    quote! {
        #ast

        #doc_comment

        #[no_mangle]
        pub extern "C" fn #ext_name() -> *mut #out {
            let res = #name();
            Box::into_raw(Box::new(res))
        }
    }
}
