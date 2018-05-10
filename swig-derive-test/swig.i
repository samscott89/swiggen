%module swig_derive_test
#define PKG_NAME swig_derive_test
%include <std_vector.i>
%include <stdint.i>
%include <std_string.i>


%{
    namespace ffi {
        #include "bindings.h"
    }

    using namespace ffi;

    namespace swig_derive_test {
// Wrapper for Rust class Test
class Test {
    public:
        ffi::Test *self;
        Test(ffi::Test *ptr) {
            self = ptr;
        };
        ~Test(){
            ffi::__SWIG_INJECT_free_Test(self);
            self = NULL;
        };
    Test() { self = __SWIG_INJECT_default_Test(); };
};
Test different_test() {
                    return Test(ffi::__SWIG_INJECT_ffi_different_test());
                }}
%}

namespace swig_derive_test {
    // Wrapper for Rust class Test
class Test {
    ffi::Test *self;
    public:
        ~Test();
    Test();
};

                %extend Test {
                     Test(uint32_t field) {
                        return new PKG_NAME::Test(ffi::__SWIG_INJECT_ffi_new(field));
                    }
                };

                %extend Test {
                    uint32_t get_field() {
                        return uint32_t(ffi::__SWIG_INJECT_ffi_get_field($self->self));
                    }
                };
Test different_test();
}

%ignore __SWIG_INJECT_;
%include "bindings.h";
