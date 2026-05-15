use std::env::var;
use std::fs::{create_dir_all, OpenOptions};
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    let generated = manifest_dir.parent().unwrap().join("target").join("generated");
    let manifest = generated.join("resources.rs");

    create_dir_all(&generated).unwrap();

    OpenOptions::new()
        .write(true)
        .create(true)
        .open(&manifest)
        .unwrap();

    println!("cargo:rerun-if-changed={}", manifest.display());
}
