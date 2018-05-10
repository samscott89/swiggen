#include <cstdint>
#include <cstdlib>

struct Test;

struct Test2 {
  uint32_t field;
};

extern "C" {

Test *__SWIG_INJECT_default_Test();

uint32_t __SWIG_INJECT_ffi_get_field(const Test *wrapped_self);

Test *__SWIG_INJECT_ffi_new(uint32_t field);

void __SWIG_INJECT_free_Test(Test *arg);

Test2 test2_func();

uint32_t test_value();

} // extern "C"
