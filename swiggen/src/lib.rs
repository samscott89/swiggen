extern crate cbindgen;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use quote::ToTokens;

use std::fmt;

use std::fs::File;
use std::io::Write;
use std::str;

// mod ty;
use cbindgen::ir::ty;
use cbindgen::writer::{Source, SourceWriter};

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

pub trait AsExtern {
    fn as_extern(&self) -> quote::Tokens;
}

impl AsExtern for syn::DeriveInput {
    fn as_extern(&self) -> quote::Tokens {
        let name = self.ident;
        let free_name = swig_free(&name);
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

struct InternalFn<'a> {
    base: &'a Option<syn::Ident>,
    fn_def: &'a syn::ItemFn,
}

fn cbindgen_write<S: Source>(s: &S) -> String {
    let mut buf = Vec::new();
    {
        let cfg = cbindgen::Config::default();
        let mut sw = SourceWriter::new(&mut buf, &cfg);
        s.write(&cfg, &mut sw);
    }
    String::from_utf8(buf).unwrap()
}

fn convert_self_type(arg: &syn::FnArg, base: &Option<syn::Ident>) -> syn::FnArg {
    let base = base.expect("Cannot convert `self` arg without provided base name.
                            Try: `#[swiggen(Foo)]` in macro");
    let mut arg = arg.clone().into_tokens().to_string();
    arg = if arg.starts_with('&') {
        arg.replace("&", "*const ")
    } else {
        "*mut ".to_string() + &arg
    };
    arg = format!("wrapped_self: {}", arg.replace("self", &base.to_string()));
    syn::parse_str(&arg).unwrap()
}

fn convert_arg_type(syn::ArgCaptured { ref pat, ref colon_token, ref ty }: &syn::ArgCaptured) -> syn::FnArg {
    if let Ok(Some(ty::Type::Primitive(t))) = ty::Type::load(ty) {
        parse_quote!(#pat: #ty)
    } else{
        parse_quote!(#pat: *const #ty)
    }
}


fn convert_ret_type(rty: &syn::ReturnType, base: &Option<syn::Ident>) -> syn::ReturnType {
    match rty {
        syn::ReturnType::Default => syn::ReturnType::Default,
        syn::ReturnType::Type(_, ty) => {
            if needs_ref(rty) {
                if ty.clone().into_tokens().to_string() == "Self" {
                    let base = base.expect("Cannot convert `Self` return type without provided base name.
                            Try: `#[swiggen(Foo)]` in macro");
                    parse_quote!(-> *mut #base)
                } else {
                    parse_quote!(-> *mut #ty)
                }
            } else {
                parse_quote!(-> #ty)
            }
        }
    }
}

fn needs_ref(rty: &syn::ReturnType) -> bool {
    match rty {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            match ty::Type::load(ty) {
                Ok(Some(ty::Type::Path(p)))=> {
                    true
                },
                _ => false,
            }
        }
    }
}

fn get_cdecl(ty: &syn::Type) -> String {
    match ty::Type::load(ty) {
        Ok(Some(t)) => {
            cbindgen_write(&t)
        },
        _ => ty.clone().into_tokens().to_string()
    }
}

fn get_cdecl_arg(id: &syn::Ident, ty: &syn::Type) -> String {
    if let Ok(Some(ty)) = ty::Type::load(ty) {
        cbindgen_write(&(id.to_string(), ty))
    } else {
        format!("{} {}", ty.clone().into_tokens().to_string(), id)
    }
}

impl<'a> AsExtern for InternalFn<'a> {
    fn as_extern(&self) -> quote::Tokens {
        let name = self.fn_def.ident;
        let ext_name = swig_fn(&name, "ffi");
        let mut args = Vec::<quote::Tokens>::new();
        let mut caller = Vec::<syn::Ident>::new();
        let mut caller_ref = Vec::<quote::Tokens>::new();
        self.fn_def.decl.inputs.iter().for_each(|ref arg| {
            match arg {
                syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                    let wrapped_self = convert_self_type(&arg, self.base);
                    args.push(wrapped_self.into_tokens());

                    let ws = syn::Ident::from("wrapped_self");
                    caller.push(ws.clone());
                    caller_ref.push(quote!{@ref #ws});
                }
                syn::FnArg::Captured(ref ac) => {
                    let id = match &ac.pat {
                        syn::Pat::Ident(pi) => {
                            pi.ident
                        },
                        _ => unimplemented!(),
                    };
                    args.push(convert_arg_type(ac).into_tokens());
                    caller.push(id.clone());
                    if let syn::Type::Reference(_) = ac.ty {
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
        let res_ref = if needs_ref(&self.fn_def.decl.output) { 
            quote!{res}
        } else {
            quote!{@prim res}
        };
        let tokens = quote! {
            #[allow(non_snake_case)]
            #[no_mangle]
            pub extern "C" fn #ext_name(#(#args),*)  #out {
                #[allow(unused_macros)]
                macro_rules! ffi_ref {
                    ($name:ident) => (
                        let $name = unsafe {
                            assert!(!$name.is_null());
                            *$name
                        };
                    );
                    (@ref $name:ident) => (
                        let $name = unsafe {
                            assert!(!$name.is_null());
                            &*$name
                        };
                    );
                    (@prim $name:ident) => {};
                }
                #[allow(unused_macros)]
                macro_rules! box_ptr {
                    ($x:expr) => (
                        Box::into_raw(Box::new($x))
                    );
                    (@prim $x:expr) => (
                        $x
                    );
                }
                #(ffi_ref!(#caller_ref);),*
                let res = #base::#name(#(#caller),*);
                box_ptr!(#res_ref)
            }
        };
        // let mut fntok = self.fn_def.into_tokens();
        // fntok.append_all(tokens);
        // fntok
        tokens
    }
}


fn swig_fn(name: &syn::Ident, fn_name: &str) -> syn::Ident {
    syn::Ident::from(format!("{}{}_{}", SwigTag::SwigInject, fn_name, name))
}

fn swig_free(name: &syn::Ident) -> syn::Ident {
    swig_fn(name, "free")
}

impl ToSwig for syn::DeriveInput {
    fn to_swig(&self) -> String {
        // prefix with tag
        let mut swigged = SwigTag::CodeStart.to_string();
        let mut swigged_h = SwigTag::HdrStart.to_string();

        let name = self.ident;
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
        // prefix with tag
        let mut swigged = SwigTag::CodeStart.to_string();
        let mut swigged_h = SwigTag::HdrStart.to_string();

        let name = self.fn_def.ident;
        let mut cb_fn = cbindgen::ir::Function::load(name.to_string(), 
                                                    &self.fn_def.decl,
                                                    true,
                                                    &[],
                                                    &None).unwrap();

        let mut args = String::new();
        let mut caller = String::new();

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


        let mut out = cbindgen_write(&cb_fn.ret);
        if out == "Self" {
            out = self.base.expect("Cannot convert `Self` return type without provided base name.
                            Try: `#[swiggen(Foo)]` in macro").to_string();
        }
        let mut ret_out = out.clone();

        let name = if name.to_string() == "new" {
            ret_out = "".to_string();
            out = "new swiggen::".to_string() + &out;
            self.base.expect("Cannot convert `Self` return type without provided base name.
                            Try: `#[swiggen(Foo)]` in macro").to_string()
        } else {
            name.to_string()
        };
    
        let ext_name = swig_fn(&self.fn_def.ident, "ffi");
        if self.base.is_none() {
            swigged.push_str(&format!("\
                {ret_out} {name}({args}) {{
                    return {out}(ffi::{ext_name}({caller}));
                }}"
                , name=name, ext_name=ext_name, out=out, ret_out=ret_out, args=args, caller=caller));
        }
        if let Some(base) = self.base {
            swigged_h.push_str(&format!("
                %extend {base_name} {{
                    {ret_out} {name}({args}) {{
                        return {out}(ffi::{ext_name}({caller}));
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


pub fn impl_extern_it(ast: &syn::DeriveInput) -> quote::Tokens {
    let comment = ast.to_swig();
    let comment = format!("#[doc=\"{}\"] #[allow(non_camel_case_types)] struct {}{};", comment, SwigTag::SwigInject, ast.ident);
    let doc_comment: syn::ItemStruct = syn::parse_str(&comment).expect("failed to generate SWIG code correctly");
    let mut tokens: quote::Tokens = doc_comment.into_tokens();
    tokens.append_all(ast.as_extern().into_iter());
    tokens
}


pub fn impl_extern_fn(base_name: &Option<syn::Ident>, ast: &syn::ItemFn) -> quote::Tokens {
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


pub fn gen_swig(src: &str) {
    let mut tmp_file = File::create("swig.i").unwrap();

    let pkg_name = "swiggen";

    tmp_file.write_all(format!("\
%module {name}

%include <std_vector.i>
%include <stdint.i>
%include <std_string.i>


%{{
    namespace ffi {{
        #include \"bindings.h\"
    }}

    using namespace ffi;

    namespace {name} {{
", name=pkg_name).as_bytes()).unwrap();

    // println!("Expanded File: {}", src);
    // println!("-------------");
    // println!("-------------");
    // println!("-------------");

    let syntax = syn::parse_file(&src).expect("Unable to parse file");

    let mut hdr = String::new();

    syntax.items.iter().flat_map(|i| {
        match i {
            syn::Item::Impl(ii) => {
                ii.items.iter().fold(Vec::new(), |mut acc, ref ii| {
                    match ii {
                        syn::ImplItem::Method(iim) => {
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
", name=pkg_name, header=hdr).as_bytes()).unwrap();
}

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

use cbindgen::utilities::{SynAbiHelpers, SynItemHelpers};

pub fn split_out_externs(ast: &syn::ItemImpl) -> quote::Tokens {
    let mut tokens = quote::Tokens::new();
    tokens.append_all(ast.items.iter().filter_map(|item| {
        match item {
            syn::ImplItem::Method(iim) => {
                // println!("{:#?}", iim);
                if iim.sig.abi.is_c(){
                    Some(item.into_tokens())
                } else {
                    let mut ret = None;
                    for attr in iim.attrs.iter() {
                        let attr = attr.interpret_meta();
                        match attr {
                            Some(syn::Meta::List(ml)) => {
                                if ml.ident == syn::Ident::from("swiggen"){
                                    if let Some(v) = ml.nested.first().map(|p| p.into_value()) {
                                        match v {
                                            syn::NestedMeta::Meta(m) => {
                                                let base_name = Some(m.name());
                                                ret = Some(impl_extern_fn(&base_name, &iim_to_itemfn(iim.clone())));
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                        // println!("{:?}", attr.interpret_meta());
                        // println!("{:#?}", args);
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