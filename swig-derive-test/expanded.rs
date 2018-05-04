#[doc="__SWIG_CLASS 
  class Test { 
        ffi::Test *self; 
        public:
            Test() { self = __swig_default_Test };
            ~Test(){
                __swig_free_Test(self);
                self = NULL;
            };
        protected:
            Test(*ffi::Test *ptr) {
                self = ptr;
            }
   }
__END_SWIG_CLASS "] struct __SWIG_Test; 
#[doc="__SWIG_FN 
  Test *test_function() {
      return Test(__swig_test_function());
  }
__END_SWIG_FN "] fn __SWIG_test_function(){}
#![feature(prelude_import)]
#![no_std]
#![feature(proc_macro)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;

#[macro_use]
extern crate swig_derive;
use swig_derive::swiggen;

// #[repr(C)]
pub struct Test {
    field: u32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::default::Default for Test {
    #[inline]
    fn default() -> Test { Test{field: ::std::default::Default::default(),} }
}
#[doc =
      "__SWIG_CLASS \n  class Test { \n        ffi::Test *self; \n        public:\n            Test() { self = __swig_default_Test };\n            ~Test(){\n                __swig_free_Test(self);\n                self = NULL;\n            };\n        protected:\n            Test(*ffi::Test *ptr) {\n                self = ptr;\n            }\n   }\n__END_SWIG_CLASS "]
struct __SWIG_Test;
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __swig_default_Test() -> *mut Test {
    Box::into_raw(Box::new(Test::default()))
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __swig_free_Test(arg: *mut Test) {
    unsafe {
        if !!arg.is_null() {



            {
                ::rt::begin_panic("assertion failed: !arg.is_null()",
                                  &("src/lib.rs", 7u32, 19u32))
            }
        };
        &*arg;
    }
}
impl Test {
    pub fn new() -> Self { Self{field: 12,} }
}
pub fn test_function() -> Test { Test::new() }
#[doc =
      "__SWIG_FN \n  Test *test_function() {\n      return Test(__swig_test_function());\n  }\n__END_SWIG_FN "]
fn __SWIG_test_function() { }
#[no_mangle]
pub extern "C" fn __swig_test_function() -> *mut Test {
    let res = test_function();
    Box::into_raw(Box::new(res))
}
