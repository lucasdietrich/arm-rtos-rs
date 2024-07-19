const LINKER_SCRIPT: &str = "mps2_an385.x";

fn main() {
    println!("cargo:rerun-if-changed={}", LINKER_SCRIPT);
}
