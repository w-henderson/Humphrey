use minify_js::minify;

use std::env::var;
use std::fs::{read_to_string, write};

fn main() {
    println!("cargo:rerun-if-changed=js/main.js");

    let cargo_manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = var("OUT_DIR").unwrap();

    let js = read_to_string(cargo_manifest_dir + "/js/main.js")
        .expect("Failed to read JavaScript source code")
        .as_bytes()
        .to_vec();

    let mut output = Vec::with_capacity(js.len());
    minify(js, &mut output).expect("Failed to minify JavaScript");

    write(out_dir + "/inject.js", output).expect("Failed to write minifed JavaScript to file");
}
