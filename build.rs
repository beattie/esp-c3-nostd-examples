fn main() {
    // No build script needed for no_std project - esp-hal provides linker scripts
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
