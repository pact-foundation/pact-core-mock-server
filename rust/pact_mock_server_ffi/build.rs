fn main() {
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let crate_name = env!("CARGO_PKG_NAME");
    let base_name = crate_name.find("-c").map(|pos| &crate_name[0..pos]).unwrap_or(&crate_name[..]);
    let source_name = base_name.replace("-", "_");
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_namespace("handles")
        .with_language(cbindgen::Language::Cxx)
        .with_namespace(&source_name)
        .with_include_guard(format!("{}_H", source_name.to_uppercase()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(format!("{}.h", base_name));
}
