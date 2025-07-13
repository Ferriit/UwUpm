fn main() {
    println!("cargo:rustc-link-lib=static=clib"); // link libclib.a
    println!("cargo:rustc-link-search=native=.");
}

