%module swiggen

%include <std_vector.i>
%include <stdint.i>
%include <std_string.i>


%{
    namespace ffi {
        #include "bindings.h"
    }

    using namespace ffi;

    namespace swiggen {
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
}
%}

namespace swiggen {
    // Wrapper for Rust class Test
class Test {
    ffi::Test *self;
    public:
        ~Test();
    Test();
};

                %extend Test {
                     Test(uint32_t field) {
                        return new swiggen::Test(ffi::__SWIG_INJECT_ffi_new(field));
                    }
                };

                %extend Test {
                    uint32_t get_field() {
                        return uint32_t(ffi::__SWIG_INJECT_ffi_get_field($self->self));
                    }
                };

}
