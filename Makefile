CC = g++
AR = ar
ARFLAGS = rcs
CFLAGS = -O2 -c -static

CSRC = src/main.cpp
COBJ = $(CSRC:.cpp=.o)
CLIB = libclib.a

TARGET = uwupm

# Architecture selection (default: x64)
ARCH ?= x64

ifeq ($(ARCH),x86)
	RUST_TARGET = i686-unknown-linux-musl
	COMPILER_FLAGS = -m32
else ifeq ($(ARCH),x64)
	RUST_TARGET = x86_64-unknown-linux-musl
	COMPILER_FLAGS =
else ifeq ($(ARCH),arm)
	RUST_TARGET = armv7-unknown-linux-musl
	COMPILER_FLAGS = -march=armv7-a
else
$(error Unknown ARCH '$(ARCH)'. Use ARCH=x86, x64, or arm)
endif

# Static Rust binary build path
RUST_BIN = target/$(RUST_TARGET)/release/uwupm

# Default target: build static C++ lib first, then Rust
all: $(CLIB) rust
	@echo "Build complete. Binary at $(RUST_BIN)"

# Rust build depends on static lib (to ensure link succeeds)
rust: $(CLIB)
	cargo build --release --target $(RUST_TARGET)

# Compile C++ code
%.o: %.cpp
	$(CC) $(CFLAGS) $(COMPILER_FLAGS) -o $@ $<

# Static C++ library
$(CLIB): $(COBJ)
	$(AR) $(ARFLAGS) $@ $^

clean:
	rm -f $(COBJ) $(CLIB)
	cargo clean

