import swig_derive_test as sdt

t = sdt.Test(13)
assert t.get_field() == 13
print("It works!")
