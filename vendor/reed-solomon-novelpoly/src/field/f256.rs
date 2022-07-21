decl_field!("f256", u8, u16, 8, gen = 0x1D, cantor = [1, 214, 152, 146, 86, 200, 88, 230]);

include!("inc_log_mul.rs");

#[cfg(table_bootstrap_complete)]
include!("inc_afft.rs");
