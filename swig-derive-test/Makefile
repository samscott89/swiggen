PKG_NAME=swig_derive_test

default: bindings compile
bindings:
	cargo build
	cargo run --manifest-path=../swiggen/Cargo.toml

compile:
	swig -python  -Wextra -c++ -o swig_wrap.cpp  swig.i
	g++ -z noexecstack -std=c++17 -Wno-register -Wall -fPIC -c -g  -o swig_wrap.o swig_wrap.cpp -I. $(shell python-config --includes)
	g++ -z noexecstack -std=c++17 -Wno-deprecated-register -static-libgcc -shared -o _${PKG_NAME}.so swig_wrap.o ../target/debug/lib${PKG_NAME}.a 

test: bindings compile
	python test.py
