import swig_derive_test as sdt

t = sdt.Test()
assert t.get_field() == 0

t = sdt.Test(12)
assert t.get_field() == 12

assert sdt.different_test().get_field() == 42

assert sdt.manual_extern() == 13

print("It works!")
