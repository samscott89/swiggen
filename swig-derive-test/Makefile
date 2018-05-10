default: bindings compile
bindings:
	cargo build
	cargo run --manifest-path=../swiggen/Cargo.toml

compile:
	swig -python  -Wextra -c++ -o swig_wrap.cpp  swig.i
	g++ -z noexecstack -std=c++17 -Wno-register -Wall -fPIC -c -g  -o swig_wrap.o swig_wrap.cpp -I. $(shell python-config --includes)
	g++ -z noexecstack -std=c++17 -Wno-deprecated-register -static-libgcc -shared -o _swiggen.so swig_wrap.o ../target/debug/libswig_derive_test.a 

