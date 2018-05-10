Rust FFI Autogen
================

_A noble snippet emswiggens the smallest macro_

This is a proof of concept using procedural macros to generate:
 - (a) Rust `extern "C"` function definitions
 - (b) SWIG wrapper code for language bindings


**WARNING**: This code is pretty much stream-of-conciousness code without
any regard for sanity or style. Partially an experiment to see if possible,
and partially just stumbling around procedural macros and syn.

Using procedural macros, so of course this needs nightly for now.

## Requirements

Needs `cargo-expand` and `swig` installed.

## Example

A full example can be found in [swig-derive-test](swig-derive-test/), with
example Makefile etc. Clone this repository, navigate to the test folder
and run `make test` to see it in action.

Summary:

 - Write Rust code
 - Add `#[derive(Swig)]` and #[swiggen]` macros where appropriate
 - Build + run swiggen to produce lib, headers and bindings
 - Run swig on binding code
 - Compile swig output
 - Run Rust code in chosen language.

Starting with:

```rust
#![feature(proc_macro)] // <- we need nightly

#[macro_use]
extern crate swig_derive;
use swig_derive::{swiggen, swiggen_hack};

#[derive(Default, Swig)]
#[swig_derive(Default)]
pub struct Test {
    pub field: u32
}

swiggen_hack!{
impl Test {
    #[swiggen(Test)]
    pub fn new(field: u32) -> Self {
        Self {
            field: field,
        }
    }

    #[swiggen(Test)]
    pub fn get_field(&self) -> u32 {
        self.field
    }
}
}

#[swiggen]
pub fn different_test() -> Test {
    Test::new(42)
}

#[no_mangle]
pub extern "C" fn manual_extern() -> u32 {
    Test::new(13).get_field()
}

```

Building this with [`crate-type` set to `staticlib` or `cdylib`](https://doc.rust-lang.org/reference/linkage.html)
 will produce some files of the form `lib_*.a`, `lib_*.so` containing a number of symbols like
`__SWIG_INJECT_get_field_Test`.

The [swiggen](swiggen/) crate contains a binary which processes a Rust crate
and outputs (a) a header file, and (b) a SWIG bindings file.
The former produced by calling out to [cbindgen](https://github.com/eqrion/cbindgen).

Using [SWIG](www.swig.org/) on the swig file, calling the appropriate
build functions (example for Python [here](swig-derive-test/Makefile)), and
we are done:

```py
>>> import swig_derive_test as sdt
>>> t = sdt.Test()
>>> t.get_field()
0
>>> t = sdt.Test(12)
>>> t.get_field()
12
>>> sdt.different_test().get_field()
42
>>> sdt.manual_extern()
13
```

## Functionality

Based on the above, what kind of seems to be working so far:

 - `#[swig(Derive)]` on a struct will generate appropriate cpp-style bindings
   in SWIG to produce nicely object-oriented code in the target language.
 - `#[swig_derive(...)]` attribute to autogen wrappers for derived methods (so far only `Default` is supported)
 - `#[swiggen]` on a regular method to get appropriately bound method
 - Some support for converting primitive types into extern types (thanks to cbindgen)
 - `Self` types can be used on method signatures (the correct struct is taken from the attribute)
 - `#[swiggen(Foo)]` generates class methods for `Foo` when used on an "impl" function
 - Regular `extern "C"` functions are still exported in the bindings

Things that don't really work:

 - `swig_hack!` is needed on an impl block so that the extern methods generated
    inside the block get pushed outside the block (otherwise they aren't actually exported)
 - No idea how well other types/structs will actually work. No real testing.
 - Currently just hacked together by making loads of the cbindgen library public
 - Probably a million more problems
 - The code is not well written at all, everything is very hacky
 - Unwraps everywhere. No real error handling. Also all hidden behind macros
   so probably incredibly impenetrable errors.
