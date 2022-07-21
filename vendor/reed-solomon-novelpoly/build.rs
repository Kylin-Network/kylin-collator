#![allow(unused)]

use std::io;

include!("src/util.rs");
include!("src/field/gen.rs");

mod f2e16 {
	include!("inc_gen_field_tables.rs");
	include!("src/field/f2e16.rs");
}

mod f256 {
	include!("inc_gen_field_tables.rs");
	include!("src/field/f256.rs");
}

#[cfg(feature = "with-alt-cxx-impl")]
fn gen_ffi_novel_poly_basis_lib() {
	cc::Build::new().file("cxx/RSErasureCode.c").include("cxx").compile("novelpolycxxffi");
}

#[cfg(feature = "with-alt-cxx-impl")]
fn gen_ffi_novel_poly_basis_bindgen() {
	use std::env;
	use std::path::PathBuf;

	println!("cargo:rustc-link-lib=novelpolycxxffi");

	println!("cargo:rerun-if-changed=wrapper.h");

	let bindings = bindgen::Builder::default()
		.header("wrapper.h")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.generate()
		.expect("Unable to generate bindings");

	// Write the bindings to the $OUT_DIR/bindings.rs file.
	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
}

fn main() -> io::Result<()> {
	f2e16::gen_field_tables()?;
	f256::gen_field_tables()?;

	#[cfg(feature = "with-alt-cxx-impl")]
	{
		gen_ffi_novel_poly_basis_lib();
		gen_ffi_novel_poly_basis_bindgen();
	}

	Ok(())
}
