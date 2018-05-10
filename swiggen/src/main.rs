extern crate cbindgen;
extern crate tempdir;
extern crate swiggen;

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
    // println!("Output: {:#?}", output);
    tmp_file.write_all(&output.stdout).unwrap();

    gen_bindings(&file_path);
    swiggen::gen_swig(str::from_utf8(&output.stdout).unwrap());
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
