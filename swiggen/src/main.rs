extern crate cbindgen;
extern crate failure;
#[macro_use]
extern crate serde;
extern crate swiggen;
extern crate tempdir;
extern crate toml;

use tempdir::TempDir;
use std::env;
use std::process::Command;
use cbindgen::{Builder, Config, Language};
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::str;

use failure::Error;

// let metadata = cargo_metadata::metadata(Some(Path::new("./Cargo.toml"))).unwrap();
#[derive(Clone, Deserialize, Debug)]
pub struct Manifest {
    pub package: Package,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Package {
    pub name: String,
}

/// Parse the Cargo.toml for a given path
pub fn manifest(manifest_path: &Path) -> Result<Manifest, Error> {
    let mut s = String::new();
    let mut f = File::open(manifest_path)?;
    f.read_to_string(&mut s)?;

    toml::from_str::<Manifest>(&s).map_err(|x| x.into())
}
pub fn main() {
    let manifest = manifest(&Path::new("./Cargo.toml")).unwrap();
    // println!("{:#?}", metadata);
    let package_name = &manifest.package.name.replace("-", "_");

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
    swiggen::gen_swig(package_name, str::from_utf8(&output.stdout).unwrap());
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
