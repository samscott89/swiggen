#![feature(proc_macro)]
#![feature(proc_macro_lib)]

#[macro_use]
extern crate swig_derive;
use swig_derive::{swiggen, swiggen_hack};

#[derive(Default, Swig)]
#[swig_derive(Default)]
pub struct Test {
    pub field: u32
}

#[repr(C)]
pub struct Test2 {
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
pub fn test_function() -> Test {
    Test::new(5)
}


#[no_mangle]
pub extern "C" fn test_value() -> u32 {
    Test::new(13).field
}

#[no_mangle]
pub extern "C" fn test2_func() -> Test2 {
    Test2 {
        field: 12
    }
}

