extern crate cbindgen;
extern crate syn;
extern crate tempdir;

use tempdir::TempDir;
use std::env;
use std::process::Command;
use cbindgen::{Builder, Config, Language};
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::str;

pub fn main() {
    let tmp_dir = TempDir::new("cargo-exp").unwrap();
    let file_path = tmp_dir.path().join("expanded.rs");
    let mut tmp_file = File::create(&file_path).unwrap();

    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    
    let mut cmd = Command::new(cargo);
    cmd.arg("expand");
    let output = cmd.output().unwrap();
    println!("Output: {:#?}", output);
    tmp_file.write_all(&output.stdout).unwrap();

    gen_bindings(&file_path);
    gen_swig(str::from_utf8(&output.stdout).unwrap());
}

fn gen_bindings(path: &Path) {
    let config = Config { language: Language::Cxx, .. Config::default() };

    Builder::new()
          .with_src(path)
          .with_config(config)
          .generate()
          .expect("Unable to generate bindings")
          .write_to_file("bindings.h");
}

fn gen_swig(src: &str) {
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

    println!("Expanded File: {}", src);
    println!("-------------");
    println!("-------------");
    println!("-------------");

    let syntax = syn::parse_file(&src).expect("Unable to parse file");
    // println!("{:?}", syntax);

    let mut hdr = String::new();

    syntax.items.iter().filter_map(|i| {
        match i {
            syn::Item::Struct(syn::ItemStruct { attrs, ident, .. }) |
            syn::Item::Fn(syn::ItemFn { attrs, ident, ..}) => {
                if ident.to_string().starts_with("__SWIG") {
                    println!("{:?}", i);
                    Some(attrs)
                } else {
                    None
                }
            },
            _ => None
        }
    }).for_each(|attrs| {
        attrs.iter().for_each(|attr| {
            match &attr.style {
                syn::AttrStyle::Outer => {
                    let code_prefix ="__SWIG_CODE";
                    let code_suffix = "__END_SWIG_CODE";
                    let hdr_prefix ="__SWIG_HDR_CODE";
                    let hdr_suffix = "__END_SWIG_HDR_CODE";

                    let mut swig_class = attr.tts.to_string();
                    let prefix_offset = swig_class.find(code_prefix).expect("no code prefix") + code_prefix.len();
                    let suffix_offset = swig_class.find(code_suffix).expect("no code suffix");
                    let final_class = &swig_class[prefix_offset..suffix_offset];


                    let prefix_offset = swig_class.find(hdr_prefix).expect("no header prefix") + hdr_prefix.len();
                    let suffix_offset = swig_class.find(hdr_suffix).expect("no header suffix");
                    let final_hdr = &swig_class[prefix_offset..suffix_offset];

                    tmp_file.write_all(&final_class.replace("\\n", "\n").as_bytes()).unwrap();
                    hdr += &final_hdr.replace("\\n", "\n");
                },
                _ => ()
            }
        })
    });

    tmp_file.write_all(format!("\
    }}
%}}

namespace {name} {{
    {header}
}}
%ignore \"__swig\";
%include \"bindings.h\"

", name=pkg_name, header=hdr).as_bytes()).unwrap();
}