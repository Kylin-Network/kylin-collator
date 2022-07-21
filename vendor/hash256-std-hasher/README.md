# Specialized Hasher for 32-bit keys

Provides `Hash256StdHasher`, a specialized `core::hash::Hasher` that takes just 8 bytes of the provided value and may only be used for keys which are 32 bytes.

The crate is `no_std`-compatible.