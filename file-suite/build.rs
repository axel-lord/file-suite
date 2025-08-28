//! Build script for file-suite

use ::std::path::{Path, PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=tools.json");

    let out_dir = PathBuf::from(::std::env::var_os("OUT_DIR").unwrap());

    ::std::fs::write(
        out_dir.join("tools.rs"),
        ::file_suite_build::tool_json_to_rust(Path::new("tools.json")),
    )
    .unwrap_or_else(|err| panic!("could not write rust code generated from tools.json, {err}"));
}
