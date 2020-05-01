use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let crate_name = env!("CARGO_PKG_NAME");
    if env::var("GENERATE_C_HEADER").is_ok() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let mut path = PathBuf::from(out_dir);
        path.push("..");
        path.push("..");
        path.push("..");
        path.push("..");
        path.push("artifacts");
        let artifacts_dir = path.canonicalize().unwrap();
        let artifacts_dir_str = artifacts_dir.to_str().unwrap();
        let base_name = crate_name.find("-c").map(|pos| &crate_name[0..pos]).unwrap_or(&crate_name[..]);
        let source_name = base_name.replace("-", "_");
        cbindgen::Builder::new()
          .with_crate(crate_dir)
          .with_include_version(true)
          .with_namespace("handles")
          .with_language(cbindgen::Language::Cxx)
          .with_namespace(&source_name)
          .with_include_guard(format!("{}_H", source_name.to_uppercase()))
          .generate()
          .expect("Unable to generate bindings")
          .write_to_file(format!("{}/{}.h", artifacts_dir_str, base_name));

        cbindgen::Builder::new()
          .with_crate(crate_dir)
          .with_include_version(true)
          .with_namespace("handles")
          .with_language(cbindgen::Language::C)
          .with_namespace(&source_name)
          .with_include_guard(format!("{}_H", source_name.to_uppercase()))
          .generate()
          .expect("Unable to generate bindings")
          .write_to_file(format!("{}/{}-c.h", artifacts_dir_str, base_name));
    }
}
