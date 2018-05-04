#![feature(proc_macro)]

#[macro_use]
extern crate swig_derive;
use swig_derive::swiggen;

#[derive(Default, Swig)]
pub struct Test {
    pub field: u32
}

#[repr(C)]
pub struct Test2 {
    pub field: u32
}

impl Test {
    pub fn new() -> Self {
        Self {
            field: 12,
        }
    }
}


#[swiggen]
pub fn test_function() -> Test {
    Test::new()
}


#[no_mangle]
pub extern "C" fn test_value() -> u32 {
    Test::new().field
}

#[no_mangle]
pub extern "C" fn test2_func() -> Test2 {
    Test2 {
        field: 12
    }
}
