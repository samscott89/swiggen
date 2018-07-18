#include <cstdint>
#include <cstdlib>

struct Test;

extern "C" {

Test *__SWIG_INJECT_default_Test();

Test *__SWIG_INJECT_ffi_different_test();

void __SWIG_INJECT_free_Test(Test *arg);

uint32_t manual_extern();

} // extern "C"
