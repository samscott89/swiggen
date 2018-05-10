#![feature(prelude_import)]
#![no_std]
#![feature(proc_macro)]
#![feature(proc_macro_lib)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;

#[macro_use]
extern crate swig_derive;
use swig_derive::{swiggen, swiggen_hack};

#[swig_derive(Default)]
pub struct Test {
    pub field: u32,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::default::Default for Test {
    #[inline]
    fn default() -> Test { Test{field: ::std::default::Default::default(),} }
}
#[doc =
      "__SWIG_CODE\n// Wrapper for Rust class Test\nclass Test {\n    public:\n        ffi::Test *self;\n        Test(ffi::Test *ptr) {\n            self = ptr;\n        };\n        ~Test(){\n            ffi::__SWIG_INJECT_free_Test(self);\n            self = NULL;\n        };\n    Test() { self = __SWIG_INJECT_default_Test(); };\n};\n__SWIG_END_CODE\n__SWIG_HDR\n// Wrapper for Rust class Test\nclass Test {\n    ffi::Test *self;\n    public:\n        ~Test();\n    Test();\n};\n__SWIG_END_HDR\n"]
#[allow(non_camel_case_types)]
struct __SWIG_INJECT_Test;
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __SWIG_INJECT_free_Test(arg: *mut Test) {
    unsafe {
        if !!arg.is_null() {









            {
                ::rt::begin_panic("assertion failed: !arg.is_null()",
                                  &("swig-derive-test/src/lib.rs", 8u32,
                                    19u32))
            }
        };
        &*arg;
    }
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __SWIG_INJECT_default_Test() -> *mut Test {
    Box::into_raw(Box::new(Test::default()))
}
#[repr(C)]
pub struct Test2 {
    pub field: u32,
}
impl Test {
    pub fn new(field: u32) -> Self { Self{field: field,} }
    #[allow(non_snake_case)]
    #[doc =
          "__SWIG_CODE\n__SWIG_END_CODE\n__SWIG_HDR\n\n                %extend Test {\n                     Test(uint32_t field) {\n                        return new swiggen::Test(ffi::__SWIG_INJECT_ffi_new(field));\n                    }\n                };\n__SWIG_END_HDR\n"]
    fn __SWIG_INJECT_hidden_ffi_new() { }
    #[allow(non_snake_case)]
    #[no_mangle]
    pub extern "C" fn __SWIG_INJECT_ffi_new(field: u32) -> *mut Test {
        #[allow(unused_macros)]
        macro_rules! ffi_ref(( $ name : ident ) => (
                             let $ name = unsafe {
                             assert ! ( ! $ name . is_null (  ) ) ; * $ name }
                             ; ) ; ( @ ref $ name : ident ) => (
                             let $ name = unsafe {
                             assert ! ( ! $ name . is_null (  ) ) ; & * $ name
                             } ; ) ; ( @ prim $ name : ident ) => {  } ;);
        #[allow(unused_macros)]
        macro_rules! box_ptr(( $ x : expr ) => (
                             Box :: into_raw ( Box :: new ( $ x ) ) ) ; (
                             @ prim $ x : expr ) => ( $ x ) ;);
        let res = Test::new(field);
        Box::into_raw(Box::new(res))
    }
    pub fn get_field(&self) -> u32 { self.field }
    #[allow(non_snake_case)]
    #[doc =
          "__SWIG_CODE\n__SWIG_END_CODE\n__SWIG_HDR\n\n                %extend Test {\n                    uint32_t get_field() {\n                        return uint32_t(ffi::__SWIG_INJECT_ffi_get_field($self->self));\n                    }\n                };\n__SWIG_END_HDR\n"]
    fn __SWIG_INJECT_hidden_ffi_get_field() { }
    #[allow(non_snake_case)]
    #[no_mangle]
    pub extern "C" fn __SWIG_INJECT_ffi_get_field(wrapped_self: *const Test)
     -> u32 {
        #[allow(unused_macros)]
        macro_rules! ffi_ref(( $ name : ident ) => (
                             let $ name = unsafe {
                             assert ! ( ! $ name . is_null (  ) ) ; * $ name }
                             ; ) ; ( @ ref $ name : ident ) => (
                             let $ name = unsafe {
                             assert ! ( ! $ name . is_null (  ) ) ; & * $ name
                             } ; ) ; ( @ prim $ name : ident ) => {  } ;);
        #[allow(unused_macros)]
        macro_rules! box_ptr(( $ x : expr ) => (
                             Box :: into_raw ( Box :: new ( $ x ) ) ) ; (
                             @ prim $ x : expr ) => ( $ x ) ;);
        let wrapped_self =
            unsafe {
                if !!wrapped_self.is_null() {
                    {
                        ::rt::begin_panic("assertion failed: !wrapped_self.is_null()",
                                          &("swig-derive-test/src/lib.rs",
                                            20u32, 1u32))
                    }
                };
                &*wrapped_self
            };
        let res = Test::get_field(wrapped_self);
        res
    }
}
#[allow(non_snake_case)]
#[doc =
      "__SWIG_CODE\n__SWIG_END_CODE\n__SWIG_HDR\n\n                %extend Test {\n                     Test(uint32_t field) {\n                        return new swiggen::Test(ffi::__SWIG_INJECT_ffi_new(field));\n                    }\n                };\n__SWIG_END_HDR\n"]
fn __SWIG_INJECT_hidden_ffi_new() { }
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __SWIG_INJECT_ffi_new(field: u32) -> *mut Test {
    #[allow(unused_macros)]
    macro_rules! ffi_ref(( $ name : ident ) => (
                         let $ name = unsafe {
                         assert ! ( ! $ name . is_null (  ) ) ; * $ name } ; )
                         ; ( @ ref $ name : ident ) => (
                         let $ name = unsafe {
                         assert ! ( ! $ name . is_null (  ) ) ; & * $ name } ;
                         ) ; ( @ prim $ name : ident ) => {  } ;);
    #[allow(unused_macros)]
    macro_rules! box_ptr(( $ x : expr ) => (
                         Box :: into_raw ( Box :: new ( $ x ) ) ) ; (
                         @ prim $ x : expr ) => ( $ x ) ;);
    let res = Test::new(field);
    Box::into_raw(Box::new(res))
}
#[allow(non_snake_case)]
#[doc =
      "__SWIG_CODE\n__SWIG_END_CODE\n__SWIG_HDR\n\n                %extend Test {\n                    uint32_t get_field() {\n                        return uint32_t(ffi::__SWIG_INJECT_ffi_get_field($self->self));\n                    }\n                };\n__SWIG_END_HDR\n"]
fn __SWIG_INJECT_hidden_ffi_get_field() { }
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __SWIG_INJECT_ffi_get_field(wrapped_self: *const Test)
 -> u32 {
    #[allow(unused_macros)]
    macro_rules! ffi_ref(( $ name : ident ) => (
                         let $ name = unsafe {
                         assert ! ( ! $ name . is_null (  ) ) ; * $ name } ; )
                         ; ( @ ref $ name : ident ) => (
                         let $ name = unsafe {
                         assert ! ( ! $ name . is_null (  ) ) ; & * $ name } ;
                         ) ; ( @ prim $ name : ident ) => {  } ;);
    #[allow(unused_macros)]
    macro_rules! box_ptr(( $ x : expr ) => (
                         Box :: into_raw ( Box :: new ( $ x ) ) ) ; (
                         @ prim $ x : expr ) => ( $ x ) ;);
    let wrapped_self =
        unsafe {
            if !!wrapped_self.is_null() {
                {
                    ::rt::begin_panic("assertion failed: !wrapped_self.is_null()",
                                      &("swig-derive-test/src/lib.rs", 20u32,
                                        1u32))
                }
            };
            &*wrapped_self
        };
    let res = Test::get_field(wrapped_self);
    res
}
pub fn test_function() -> Test { Test::new(5) }
#[allow(non_snake_case)]
#[doc =
      "__SWIG_CODE\nTest test_function() {\n                    return Test(ffi::__SWIG_INJECT_ffi_test_function());\n                }__SWIG_END_CODE\n__SWIG_HDR\nTest test_function();__SWIG_END_HDR\n"]
fn __SWIG_INJECT_hidden_ffi_test_function() { }
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn __SWIG_INJECT_ffi_test_function() -> *mut Test {
    #[allow(unused_macros)]
    macro_rules! ffi_ref(( $ name : ident ) => (
                         let $ name = unsafe {
                         assert ! ( ! $ name . is_null (  ) ) ; * $ name } ; )
                         ; ( @ ref $ name : ident ) => (
                         let $ name = unsafe {
                         assert ! ( ! $ name . is_null (  ) ) ; & * $ name } ;
                         ) ; ( @ prim $ name : ident ) => {  } ;);
    #[allow(unused_macros)]
    macro_rules! box_ptr(( $ x : expr ) => (
                         Box :: into_raw ( Box :: new ( $ x ) ) ) ; (
                         @ prim $ x : expr ) => ( $ x ) ;);
    let res = ::test_function();
    Box::into_raw(Box::new(res))
}
#[no_mangle]
pub extern "C" fn test_value() -> u32 { Test::new(13).field }
#[no_mangle]
pub extern "C" fn test2_func() -> Test2 { Test2{field: 12,} }
