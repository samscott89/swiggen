#include <cstdint>
#include <cstdlib>

struct Test;

extern "C" {

Test *__SWIG_INJECT_default_Test();

Test *__SWIG_INJECT_ffi_different_test();

uint32_t __SWIG_INJECT_ffi_get_field(const Test *wrapped_self);

Test *__SWIG_INJECT_ffi_new(uint32_t field);

void __SWIG_INJECT_free_Test(Test *arg);

uint32_t manual_extern();

} // extern "C"
