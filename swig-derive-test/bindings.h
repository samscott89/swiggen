#include <cstdint>
#include <cstdlib>

struct Test;

struct Test2 {
  uint32_t field;
};

extern "C" {

Test *__swig_default_Test();

void __swig_free_Test(Test *arg);

Test *__swig_test_function();

Test2 test2_func();

uint32_t test_value();

} // extern "C"
