#![feature(proc_macro)]

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
