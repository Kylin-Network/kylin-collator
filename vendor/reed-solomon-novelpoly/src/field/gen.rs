/// Declare a field that does not need imlpementations for encode and decode algorithms.
macro_rules! decl_field {
	($name:literal, $element:ident, $wide:ident, $fbits:literal, gen=$generator:literal, cantor=$cantor:expr) => {
		pub const FIELD_NAME: &'static str = $name;

		pub type Elt = $element;
		pub type Wide = $wide;

		pub const FIELD_BITS: usize = $fbits;
		pub const FIELD_SIZE: usize = 1_usize << FIELD_BITS;

		/// Quotient ideal generator given by tail of irreducible polynomial
		pub const GENERATOR: Elt = $generator;

		pub const ONEMASK: Elt = (FIELD_SIZE - 1) as Elt;

		// Cantor basis
		pub const BASE: [Elt; FIELD_BITS] = $cantor;

		#[cfg(table_bootstrap_complete)]
		include!(concat!(env!("OUT_DIR"), "/table_", $name, ".rs"));
	};
}
