use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-search=./target/debug");
}
