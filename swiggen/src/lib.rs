#![allow(unused_doc_comment)]

/// # swiggen
/// 
/// The `swiggen` library is used to generate `extern "C"` definitions and
/// SWIG wrapper code from Rust functions.
///
/// This basically does two things: generates the `extern "C"` methods by
/// applying typemaps from cbindgen, or some fairly crude heuristics - 
/// such as converting an opaque `Foo` into a `*mut Foo`, and running
/// `Box::into_raw(Box::new(foo))` to convert it into a pointer.
///
/// These exported functions all have mangled names like `__SWIG_INJECT_new_Foo`.
/// The code also generates SWIG wrapper code which wraps these functions sp
/// that `Foo` behaves like a native object with methods like `Foo.new`.
/// The SWIG code is injected into the expanded Rust source code through doc
/// comments on various structs/functions.


extern crate cbindgen;
#[macro_use]
extern crate log;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use quote::TokenStreamExt;

use std::fmt;

use std::fs::File;
use std::io::Write;
use std::str;

use cbindgen::ir::ty;
use cbindgen::utilities::SynAbiHelpers;
use cbindgen::writer::{Source, SourceWriter};

/// Tags used to indicate swig binding code injected into the Rust source.
enum SwigTag {
    CodeStart,
    CodeEnd,
    HdrStart,
    HdrEnd,
    SwigInject,
}

impl fmt::Display for SwigTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = self.to_str();
        write!(f, "{}", tag)
    }
}

impl SwigTag {
    fn to_str(&self) -> &'static str {
        match self {
            SwigTag::CodeStart => "__SWIG_CODE\n",
            SwigTag::CodeEnd   => "__SWIG_END_CODE\n",
            SwigTag::HdrStart  => "__SWIG_HDR\n",
            SwigTag::HdrEnd    => "__SWIG_END_HDR\n",
            SwigTag::SwigInject=> "__SWIG_INJECT_",
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            SwigTag::CodeStart => "__SWIG_CODE\n",
            SwigTag::CodeEnd   => "__SWIG_END_CODE\n",
            SwigTag::HdrStart  => "__SWIG_HDR\n",
            SwigTag::HdrEnd    => "__SWIG_END_HDR\n",
            SwigTag::SwigInject=> "__SWIG_INJECT_",
        }.len()
    }
}

pub trait ToSwig {
    fn to_swig(&self) -> String;
}

/// A type implementing `AsExtern` can be converted into an type compatible with
/// `extern "C"` functions.
pub trait AsExtern {
    fn as_extern(&self) -> TokenStream;
}

impl AsExtern for syn::DeriveInput {
    fn as_extern(&self) -> TokenStream {
        let name = &self.ident;
        let free_name = swig_free(&name);
        // For an stuct we want to derive Swig for, we add a `free_Foo`
        // method so we can free it from SWIG code.
        let mut tokens = quote! {
            #[allow(non_snake_case)]
            #[no_mangle]
            pub extern "C" fn #free_name(arg: *mut #name) {
                unsafe {
                    assert!(!arg.is_null());
                    &*arg;
                }
            }
        };
        let default_name = swig_fn(&name, "default");

        // TOOD: Add more derive capabilities
        // Extracting the derived methods from `#[swig_derive(...)]`.
        // We need to automatically add the SWIG code since we cant somehow
        // add the `#[swiggen(Foo)]` attribute to the derived methods.
        let derivs = get_derives(&self.attrs);
        let new_toks = derivs.iter().filter_map(|w| {
            match w.as_str() {
                "Default" => {
                    Some(quote! {
                        #[allow(non_snake_case)]
                        #[no_mangle]
                        pub extern "C" fn #default_name() -> *mut #name {
                            Box::into_raw(Box::new(#name::default()))
                        }
                    })
                },
                _ => None
            }
        });
        tokens.append_all(new_toks);
        tokens
    }
}

/// A method definition inside an impl block has an additional
/// `base` variable corresponding to the name of the type.
struct InternalFn<'a> {
    base: &'a Option<syn::Ident>,
    fn_def: &'a syn::ItemFn,
}

/// Convenience method to use cbindgen to convert types into C-compat types.
/// e.g. "input: u32" -> `cbindgen_write((input, u32))` might output `uint32 input`.
fn cbindgen_write<S: Source>(s: &S) -> String {
    let mut buf = Vec::new();
    {
        let cfg = cbindgen::Config::default();
        let mut sw = SourceWriter::new(&mut buf, &cfg);
        s.write(&cfg, &mut sw);
    }
    String::from_utf8(buf).unwrap().replace("str", "char")
}

/// Hacky method to take a `&self` or `self` function argument and produce
/// something compatible with `extern "C"` method. Since we can't use `self`, 
/// we coerce this to a pointer, and call the arg `wrapped_self`.
fn convert_self_type(arg: &syn::FnArg, base: &Option<syn::Ident>) -> syn::FnArg {
    let base = base.clone().expect("Cannot convert `self` arg without provided base name.
                            Try: `#[swiggen(Foo)]` in macro");
    let mut arg = arg.clone().into_token_stream().to_string();
    arg = if arg.starts_with('&') {
        arg.replace("&", "*const ")
    } else {
        "*mut ".to_string() + &arg
    };
    arg = format!("wrapped_self: {}", arg.replace("self", &base.to_string()));
    syn::parse_str(&arg).unwrap()
}

/// For inputs, if the type is a primitive (as defined by cbindgen), we don't 
/// do anything. Otherwise, assume we will take it in as a pointer.
fn convert_arg_type(syn::ArgCaptured { ref pat, ref ty, .. }: &syn::ArgCaptured) -> syn::FnArg {
    if ty.clone().into_token_stream().to_string().ends_with("str") {
        parse_quote!(#pat: *const c_char)
    } else {
        if needs_ref(ty) {
            parse_quote!(#pat: *const #ty)
        } else {
            parse_quote!(#pat: #ty)
        }
    }
}

/// Similar to above, make sure that we return primitives when 
/// recognised 
fn convert_ret_type(rty: &syn::ReturnType, base: &Option<syn::Ident>) -> syn::ReturnType {
    match rty {
        syn::ReturnType::Default => syn::ReturnType::Default,
        syn::ReturnType::Type(_, ty) => {
            if needs_ref(ty) {
                if ty.clone().into_token_stream().to_string() == "Self" {
                    let base = base.clone().expect("Cannot convert `Self` return type without provided base name.
                            Try: `#[swiggen(Foo)]` in macro");
                    parse_quote!(-> *mut #base)
                } else if ty.clone().into_token_stream().to_string() == "String" {
                    parse_quote!(-> *mut c_char)
                } else {
                    parse_quote!(-> *mut #ty)
                }
            } else {
                parse_quote!(-> #ty)
            }
        }
    }
}

/// For paths, assume we can convert to an opaque pointer.
fn needs_ref(ty: &syn::Type) -> bool {
    match ty::Type::load(ty) {
        Ok(Some(ty::Type::Primitive(_))) => false,
        Ok(Some(ty::Type::Path(_)))=> true,
        _ => false,
    }
}

impl<'a> AsExtern for InternalFn<'a> {
    fn as_extern(&self) -> TokenStream {
        // Messy blob of code to convert function name, arguments, types, 
        // return type and generate appropriate code.
        // Should be extracted out into smaller functions.
        let name = &self.fn_def.ident;
        let ext_name = swig_fn(&name, "ffi");
        let mut args = Vec::<TokenStream>::new();
        let mut caller = Vec::<syn::Ident>::new();
        let mut caller_ref = Vec::<TokenStream>::new();
        self.fn_def.decl.inputs.iter().for_each(|ref arg| {
            match arg {
                syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                    // For self methods, we do some extra work to wrap the
                    // function so that `impl Foo { fn bar(&self); }`
                    // becomes `Foo_bar(wrapped_self: *const Foo)`.
                    let wrapped_self = convert_self_type(&arg, self.base);
                    args.push(wrapped_self.into_token_stream());

                    let ws = syn::Ident::new("wrapped_self", Span::call_site());
                    caller.push(ws.clone());
                    caller_ref.push(quote!{@ref #ws});
                }
                syn::FnArg::Captured(ref ac) => {
                    let id = match &ac.pat {
                        syn::Pat::Ident(pi) => {
                            &pi.ident
                        },
                        _ => unimplemented!(),
                    };
                    args.push(convert_arg_type(ac).into_token_stream());
                    caller.push(id.clone());

                    // this later calls the appropriate macro function as to
                    // whether we need to do some pointer/box stuff
                    if ac.ty.clone().into_token_stream().to_string().ends_with("str") {
                        caller_ref.push(quote!{@str #id});
                    } else if let syn::Type::Reference(_) = ac.ty {
                        caller_ref.push(quote!{@ref #id});
                    } else {
                        caller_ref.push(quote!{@prim #id});
                    }
                },
                _ => ()
            }
        });
        let base = self.base;
        let out = convert_ret_type(&self.fn_def.decl.output, self.base);
        // Similar to the above, this later calls the appropriate macro function
        // as to whether we need to do some pointer/box stuff
        let res_ref = if let syn::ReturnType::Type(_, ref ty) = self.fn_def.decl.output {
            if ty.clone().into_token_stream().to_string() == "String" {
                quote!{@str res}
            } else if needs_ref(&ty) {
                quote!{res}
            } else {
                quote!{@prim res}
            }
        } else {
            quote!{@prim res}
        };

        /// Generate the function. We also inject some macro
        /// definitions to help with converting pointers into types and types
        /// into pointers.
        let tokens = quote! {
            #[allow(non_snake_case)]
            #[no_mangle]
            pub extern "C" fn #ext_name(#(#args),*)  #out {
                #(ffi_ref!(#caller_ref);)*
                let res = #base::#name(#(#caller),*);
                box_ptr!(#res_ref)
            }
        };
        tokens
    }
}


/// Helper function to define the exported/mangled names.
fn swig_fn(name: &syn::Ident, fn_name: &str) -> syn::Ident {
    syn::Ident::new(&format!("{}{}_{}", SwigTag::SwigInject, fn_name, name), Span::call_site())
}

fn swig_free(name: &syn::Ident) -> syn::Ident {
    swig_fn(name, "free")
}

impl ToSwig for syn::DeriveInput {
    fn to_swig(&self) -> String {
        /// Generate the SWIG wrapper code as a string.
        /// Basically, a class for the Rust struct `Foo` is just a wrapper
        /// class called `Foo` which contains a pointer to the actual Rust
        /// object.

        // prefix with tag
        let mut swigged = SwigTag::CodeStart.to_string();
        let mut swigged_h = SwigTag::HdrStart.to_string();

        let name = &self.ident;
        match &self.data {
            syn::Data::Struct(ref _ds) => {
                // simple wrapper definition to wrap opaque pointer.
                // methods get added elsewhere
                swigged.push_str(&format!("\
// Wrapper for Rust class {name}
class {name} {{
    public:
        ffi::{name} *self;
        {name}(ffi::{name} *ptr) {{
            self = ptr;
        }};
        ~{name}(){{
            ffi::{free_name}(self);
            self = NULL;
        }};
    ", name=name, free_name=swig_free(&name))
                );

swigged_h.push_str(&format!("\
// Wrapper for Rust class {name}
class {name} {{
    ffi::{name} *self;
    public:
        ~{name}();
    ", name=name)
                );
                // pull out any derive implementations we want to wrap
                // TODO: do this in a less ad-hoc way
                get_derives(&self.attrs).iter().for_each(|w| {
                    match w.as_str() {
                        "Default" => {
                            swigged.push_str(&format!(
                                "{name}() {{ self = {def_name}(); }};\n",
                                name=name, def_name=swig_fn(&name, "default")
                            ));
                            swigged_h.push_str(&format!("{}();\n",name));
                        },
                        _ => (),
                    }

                });
                swigged.push_str("};\n");
                swigged_h.push_str("};\n");
            },
            _ => unimplemented!(),
        }
        swigged.push_str(&SwigTag::CodeEnd.to_str());
        swigged_h.push_str(&SwigTag::HdrEnd.to_str());
        swigged.push_str(&swigged_h);
        swigged
    }
}

impl<'a> ToSwig for InternalFn<'a> {
    fn to_swig(&self) -> String {
        // Generate SWIG wrapper for methods.
        // Main complication is making sure that namespaces are correct since
        // we are basically overwriting names.
        // Also a bit of magic to take an impl method, and add it back into
        // being a class method.

        // prefix with tag
        let mut swigged = SwigTag::CodeStart.to_string();
        let mut swigged_h = SwigTag::HdrStart.to_string();

        let name = &self.fn_def.ident;
        let cb_fn = cbindgen::ir::Function::load(name.to_string(), 
                                                    &self.fn_def.decl,
                                                    true,
                                                    &[],
                                                    &None).unwrap();

        let mut args = String::new();
        let mut caller = String::new();

        // Convert function arguments
        cb_fn.args.iter().for_each(|arg| {
            if args.len() > 0 {
                args += ", ";
            }
            if caller.len() > 0 {
                caller += ", ";
            }
            if arg.0 == "self" {
                caller += "$self->self";
            } else {
                args += &cbindgen_write(arg);
                caller += &arg.0;
            }
        });


        // Convert return type
        let mut out = cbindgen_write(&cb_fn.ret);
        if out == "Self" {
            out = self.base.clone().expect("Cannot convert `Self` return type without provided base name.
                            Try: `#[swiggen(Foo)]` in macro").to_string();
        } else if out == "String" {
            out = "char *".to_string()
        }
        let mut ret_out = out.clone();


        // Convert function name.
        let name = if name.to_string() == "new" {
            // Custom format for new functions
            ret_out = "".to_string();
            out = "new PKG_NAME::".to_string() + &out;
            self.base.clone().expect("Cannot convert `Self` return type without provided base name.
                            Try: `#[swiggen(Foo)]` in macro").to_string()
        } else {
            name.to_string()
        };
        
        // Get the mangled name exported by Rust
        let ext_name = swig_fn(&self.fn_def.ident, "ffi");

        // The following code generates the function definitions and the header
        // Code needed for SWIG to generate bindings.

        if self.base.is_none() {
            swigged.push_str(&format!("\
                {ret_out} {name}({args}) {{
                    return ({out})(ffi::{ext_name}({caller}));
                }}"
                , name=name, ext_name=ext_name, out=out, ret_out=ret_out, args=args, caller=caller));
        }
        if let Some(base) = self.base {
            // Note the %extend is used by SWIG to make this a class method for
            // `base`.
            swigged_h.push_str(&format!("
                %extend {base_name} {{
                    {ret_out} {name}({args}) {{
                        return ({out})(ffi::{ext_name}({caller}));
                    }}
                }};\n"
                ,name=name, ext_name=ext_name, base_name=base, ret_out=ret_out, out=out, args=args, caller=caller));
        } else {
            swigged_h.push_str(&format!("\
                {out} {name}({args});"
                , name=name, out=out, args=args));
        }

        swigged.push_str(&SwigTag::CodeEnd.to_str());
        swigged_h.push_str(&SwigTag::HdrEnd.to_str());
        swigged.push_str(&swigged_h);
        swigged
    }
}


/// Generate extern and SWIG code for a `#[derive(Swig)]` annotated item.
pub fn impl_extern_it(ast: &syn::DeriveInput) -> TokenStream {
    let comment = ast.to_swig();
    let comment = format!("#[doc=\"{}\"] #[allow(non_camel_case_types)] struct {}{};", comment, SwigTag::SwigInject, ast.ident);
    let doc_comment: syn::ItemStruct = syn::parse_str(&comment).expect("failed to generate SWIG code correctly");
    let mut tokens: TokenStream = doc_comment.into_token_stream();
    tokens.append_all(ast.as_extern().into_iter());
    tokens
}

/// Generate extern and SWIG code for a `#[swiggen]` annotated method.
pub fn impl_extern_fn(base_name: &Option<syn::Ident>, ast: &syn::ItemFn) -> TokenStream {
    let ifn = InternalFn {
        base: base_name,
        fn_def: ast,
    };

    let tok = ifn.as_extern();
    let comment = ifn.to_swig();
    let hidden = swig_fn(&ast.ident, "hidden_ffi");
    quote! {
        #[allow(non_snake_case)]
        #[doc=#comment]
        fn #hidden(){}

        #tok
    }
}

/// Write the swig code (injected via doc comments) into `swig.i`.
/// This parses expanded Rust code, and writes the SWIG code to a file.
pub fn gen_swig(pkg_name: &str, src: &str) {
    let mut tmp_file = File::create("swig.i").unwrap();

    tmp_file.write_all(format!("\
%module {name}
#define PKG_NAME {name}
%include <std_vector.i>
%include <stdint.i>
%include <std_string.i>

%typemap(newfree) char * \"free_string($1);\";


%{{
    namespace ffi {{
        #include \"bindings.h\"
    }}

    using namespace ffi;

    namespace {name} {{
", name=pkg_name).as_bytes()).unwrap();

    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    trace!("Syntax: {:#?}", syntax);
    let mut hdr = String::new();

    // SWIG code is inside doc comments:
    // #[doc = "<swig code here>"]
    // struct __SWIG_INJECT_Foo;
    //
    // So we extract this out.

    syntax.items.iter().flat_map(|i| {
        // Extract out all of the attributes which are attached to structs/functions
        // starting with "__SWIG_INJECT"
        match i {
            syn::Item::Impl(ii) => {
                ii.items.iter().fold(Vec::new(), |mut acc, ref ii| {
                    match ii {
                        syn::ImplItem::Method(iim) => {
                            debug!("{:#?}", iim);
                            if iim.sig.ident.to_string().starts_with(SwigTag::SwigInject.to_str()) {
                                acc.extend_from_slice(&iim.attrs[..]);
                            }            
                            acc
                        },
                        _ => Vec::new(),
                    }
                })
            },
            syn::Item::Struct(syn::ItemStruct { attrs, ident, .. }) |
            syn::Item::Fn(syn::ItemFn { attrs, ident, ..}) => {
                if ident.to_string().starts_with(SwigTag::SwigInject.to_str()) {
                    debug!("{:#?}", attrs);
                    attrs.clone()
                } else {
                    Vec::new()
                }
            },
            _ => Vec::new()
        }
    }).for_each(|ref attr| {
        match attr.interpret_meta() {
            Some(syn::Meta::NameValue(ref mnv)) if &mnv.ident.to_string() == "doc" => {
                // Extract out the doc comment for these attributes
                if let syn::Lit::Str(ref ls) = mnv.lit {
                    let mut swig_class = ls.value().replace("\\n", "\n");
                    let prefix_offset = swig_class.find(SwigTag::CodeStart.to_str()).expect("no code prefix") + SwigTag::CodeStart.len();
                    let suffix_offset = swig_class.find(SwigTag::CodeEnd.to_str()).expect("no code suffix");
                    let final_class = &swig_class[prefix_offset..suffix_offset];

                    let prefix_offset = swig_class.find(SwigTag::HdrStart.to_str()).expect("no header prefix") + SwigTag::HdrStart.len();
                    let suffix_offset = swig_class.find(SwigTag::HdrEnd.to_str()).expect("no header suffix");
                    let final_hdr = &swig_class[prefix_offset..suffix_offset];

                    tmp_file.write_all(&final_class.replace("\\n", "\n").as_bytes()).unwrap();
                    hdr += &final_hdr.replace("\\n", "\n");
                    debug!("{}", final_hdr);
                    debug!("{}", final_class);                    

                }
            },
            _ => ()
            }
    });

    tmp_file.write_all(format!("\
    }}
%}}

namespace {name} {{
    {header}
}}

%ignore {inject};
%include \"bindings.h\";
", name=pkg_name, header=hdr, inject=SwigTag::SwigInject).as_bytes()).unwrap();
}


/// Extract out any `derive(Foo)` attributes.
fn get_derives(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs.iter().filter_map(|a| a.interpret_meta())
          .filter_map(|a| {
            if let syn::Meta::List(ml) = a {
                Some(ml)
            } else {
                None
            }
          }).filter(|ml| ml.ident.to_string() == "swig_derive")
          .flat_map(|ml| ml.nested)
          .filter_map(|nm| {
            if let syn::NestedMeta::Meta(m) = nm {
                if let syn::Meta::Word(w) = m {
                    Some(w.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }).collect()
}

/// Parse a Rust file to extract any extern "C" functions or
/// `#[swiggen]`-annotated methods and move these out of the impl block.
pub fn split_out_externs(ast: &syn::ItemImpl) -> TokenStream {
    let mut tokens = TokenStream::new();
    tokens.append_all(ast.items.iter().filter_map(|item| {
        match item {
            syn::ImplItem::Method(iim) => {
                if iim.sig.abi.is_c(){
                    Some(item.into_token_stream())
                } else {
                    let mut ret = None;
                    for attr in iim.attrs.iter().filter_map(|a| a.interpret_meta()) {
                        match attr {
                            syn::Meta::List(ml) => if ml.ident == syn::Ident::new("swiggen", Span::call_site()) {
                                if let Some(v) = ml.nested.first().map(|p| p.into_value()) {
                                    match v {
                                        syn::NestedMeta::Meta(m) => {
                                            let base_name = Some(m.name());
                                            ret = Some(impl_extern_fn(&base_name, &iim_to_itemfn(iim.clone())));
                                        },
                                        _ => {}
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                    ret
                }
            },
            _ => None,
        }
    }));

    quote!{
        #ast

        #tokens
    }
}

#[derive(Debug)]
pub struct Args(pub Option<syn::Ident>);

// Extract an `Option<Ident>` from `(T)` or `""`.
impl syn::synom::Synom for Args {
    named!(parse -> Self, map!(option!(map!(
        parens!(syn!(syn::Ident)),
        |(_parens, id)| id
    )), |o| Args(o)));
}

fn iim_to_itemfn(iim: syn::ImplItemMethod) -> syn::ItemFn {
    syn::ItemFn {
        attrs: iim.attrs,
        vis: iim.vis,
        constness: iim.sig.constness,
        unsafety: iim.sig.unsafety,
        abi: iim.sig.abi,
        ident: iim.sig.ident,
        decl: Box::new(iim.sig.decl),
        block: Box::new(iim.block),
    }
}
