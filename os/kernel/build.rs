pub fn main() {
    if ::std::env::var("TARGET").unwrap().contains("aarch64-none-elf") {
        println!("cargo:rustc-link-search=native=ext");
        println!("cargo:rustc-link-lib=static=sd");
        println!("cargo:rerun-if-changed=ext/libsd.a");
    }

    println!("cargo:rerun-if-changed=ext/layout.ld");
    println!("cargo:rerun-if-changed=ext/init.S");
}
