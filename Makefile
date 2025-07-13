CC = g++
CFLAGS = -O2 -c
AR = ar
ARFLAGS = rcs

CSRC = src/main.cpp
COBJ = $(CSRC:.cpp=.o)
CLIB = libclib.a

TARGET = uwupm

# Build static library from C++ sources
$(CLIB): $(COBJ)
	$(AR) $(ARFLAGS) $@ $^

# Compile .cpp to .o
%.o: %.cpp
	$(CC) $(CFLAGS) $< -o $@

# Build Rust project (using Cargo)
rust:
	cargo build --release


clean:
	rm -f $(COBJ) $(CLIB)
	cargo clean

