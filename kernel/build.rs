const CHIP_LINKER_SCRIPT: &str = "mps2_an38x.x";
const COMMON_LINKER_SCRIPT: &str = "common.x";

fn main() {
    println!("cargo:rerun-if-changed={}", CHIP_LINKER_SCRIPT);
    println!("cargo:rerun-if-changed={}", COMMON_LINKER_SCRIPT);
}
