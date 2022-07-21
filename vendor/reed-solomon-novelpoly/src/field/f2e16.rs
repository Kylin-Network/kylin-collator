#[cfg(table_bootstrap_complete)]
use super::*;

decl_field!(
	"f2e16",
	u16,
	u32,
	16,
	gen = 0x2D,
	cantor =
		[1_u16, 44234, 15374, 5694, 50562, 60718, 37196, 16402, 27800, 4312, 27250, 47360, 64952, 64308, 65336, 39198]
);

include!("inc_log_mul.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_afft.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_encode.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_reconstruct.rs");
