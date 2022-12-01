use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var("OUT_DIR").unwrap();
    drop(fs::create_dir_all(&out_dir));
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rustc-env=NATE_DERIVE_OUTDIR={}", out_dir);
}
