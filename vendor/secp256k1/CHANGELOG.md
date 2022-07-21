# 0.21.3 - 2022-01-31

* Several documentation improvements ([#366](https://github.com/rust-bitcoin/rust-secp256k1/pull/366), [#365](https://github.com/rust-bitcoin/rust-secp256k1/pull/365), [#373](https://github.com/rust-bitcoin/rust-secp256k1/pull/373), [#381](https://github.com/rust-bitcoin/rust-secp256k1/pull/381), [#369](https://github.com/rust-bitcoin/rust-secp256k1/pull/369), [#389](https://github.com/rust-bitcoin/rust-secp256k1/pull/389), [#391](https://github.com/rust-bitcoin/rust-secp256k1/pull/391), [#397](https://github.com/rust-bitcoin/rust-secp256k1/pull/397), [#399](https://github.com/rust-bitcoin/rust-secp256k1/pull/399), [#340](https://github.com/rust-bitcoin/rust-secp256k1/pull/365))
* Deprecate the [`generate_schnorrsig_keypair` method](https://github.com/rust-bitcoin/rust-secp256k1/pull/372) (unclear value)
* Add [serde traits to `KeyPair`](https://github.com/rust-bitcoin/rust-secp256k1/pull/379)
* Redo the [API of the new `Parity` type](https://github.com/rust-bitcoin/rust-secp256k1/pull/382) to more clearly match our desired semantics; **the `From<i32>` impl on this type is now deprecated**. Also [#400](https://github.com/rust-bitcoin/rust-secp256k1/pull/400).
* Randomize [the global context on creation](https://github.com/rust-bitcoin/rust-secp256k1/pull/385) when possible; weaken [`global-context-less-secure` feature accordingly](https://github.com/rust-bitcoin/rust-secp256k1/pull/407).
* Improve [the global context API](https://github.com/rust-bitcoin/rust-secp256k1/pull/392)
* Fix [the `Debug` impl](https://github.com/rust-bitcoin/rust-secp256k1/pull/393) for `RecoverableSignature`
* Implement [`LowerHex` and `Display`](https://github.com/rust-bitcoin/rust-secp256k1/pull/398)

# 0.21.0 - 2022-01-02

* Fix `KeyPair::from_seckey_slice` [error return value](https://github.com/rust-bitcoin/rust-secp256k1/pull/316)
* Reduce the `lowmemory` [precomp table size](https://github.com/rust-bitcoin/rust-secp256k1/pull/323)
* [Add `KeyPair::serialize_sec`](https://github.com/rust-bitcoin/rust-secp256k1/pull/308)
* Increase [`bitcoin_hashes` version to 0.10](https://github.com/rust-bitcoin/rust-secp256k1/pull/326); rename `secp256k1::bitcoin_hashes` module to `secp256k1::hashes` to align with `bitcoin` crate naming
* Add new [error variant for `PublicKey::combine_keys`](https://github.com/rust-bitcoin/rust-secp256k1/pull/304)
* Change `Display` and `Debug` for secret keys to [only output a truncated hash](https://github.com/rust-bitcoin/rust-secp256k1/pull/312)
* [Improve documentation](https://github.com/rust-bitcoin/rust-secp256k1/pull/307)
* [Implement `Hash` for `schnorrsig::Signature`](https://github.com/rust-bitcoin/rust-secp256k1/pull/335)
* Refactor modules to put [Schnorr and ECDSA on more equal footing](https://github.com/rust-bitcoin/rust-secp256k1/pull/327)
* Add serde traits [for `KeyPair` type](https://github.com/rust-bitcoin/rust-secp256k1/pull/313)
* Fix [context bound requirements for a few methods](https://github.com/rust-bitcoin/rust-secp256k1/pull/342)
* Add a [static immutable-zero aligned type](https://github.com/rust-bitcoin/rust-secp256k1/pull/345)
* Change `tweak_add_assign` and `tweak_add_check` to [use an opaque `Parity` type rather than a boolean](https://github.com/rust-bitcoin/rust-secp256k1/pull/344/)

# 0.20.3 - 2021-06-10

* Fix [`SecretKey` validation in `from_str`](https://github.com/rust-bitcoin/rust-secp256k1/pull/296)
* Add [`global-context-less-secure` feature](https://github.com/rust-bitcoin/rust-secp256k1/pull/279) which creates a non-randomized global context (and does not require `rand` or `std`)
* Add [`schnorrsig::KeyPair::from_secret_key` convenience function](https://github.com/rust-bitcoin/rust-secp256k1/pull/294)
* Add [`combine_keys` function to `PublicKey`](https://github.com/rust-bitcoin/rust-secp256k1/pull/291)
* [Reduce symbol visibility in C compilation to allow LTO to work](https://github.com/rust-bitcoin/rust-secp256k1/pull/289)
* Add [`alloc` feature](https://github.com/rust-bitcoin/rust-secp256k1/pull/300) **requiring rustc 1.36+** to enable context creation without std
* [Rewrite stubbed-out-for-fuzzing version of the library](https://github.com/rust-bitcoin/rust-secp256k1/pull/282) to improve fuzzer accessibility

# 0.20.2 - 2021-04-27

* Fix some WASM build issues
* Add [some missing `#derive`s to `Error`](https://github.com/rust-bitcoin/rust-secp256k1/pull/277/)
* Add [serde support for Schnorr signatures and for deserializing from owned types](https://github.com/rust-bitcoin/rust-secp256k1/pull/270/)

# 0.20.0 - 2020-12-21

* [remove `ffi::PublicKey::blank`](https://github.com/rust-bitcoin/rust-secp256k1/pull/232) and replace with unsafe [`ffi::PublicKey::new` and `ffi::PublicKey::from_array_unchecked`](https://github.com/rust-bitcoin/rust-secp256k1/pull/253/); similar for all other FFI types
* [support wasm32-wasi target](https://github.com/rust-bitcoin/rust-secp256k1/pull/242)
* [make the global-context feature depend on the rand-std feature](https://github.com/rust-bitcoin/rust-secp256k1/pull/246)
* [add a lexicographic ordering to `PublicKey`](https://github.com/rust-bitcoin/rust-secp256k1/pull/248) which does **not** match the ordering used by Bitcoin Core (matching this would be impossible as it requires tracking a compressedness flag, which libsecp256k1 does not have)
* [implement BIP340 Schnorr signatures](https://github.com/rust-bitcoin/rust-secp256k1/pull/237)
* [require use of new `AlignedType` in preallocated-context API to enforce alignment requirements](https://github.com/rust-bitcoin/rust-secp256k1/pull/233); previously it was possible to get UB by using misaligned memory stores
* [enforce correct alignment when using preallocated context API](https://github.com/rust-bitcoin/rust-secp256k1/pull/233)
* [stop using cargo features for dangerous build-breaking options, require setting `RUSTFLAGS` instead](https://github.com/rust-bitcoin/rust-secp256k1/pull/263)
* [implement low-R signing and function to grind even smaller signatures](https://github.com/rust-bitcoin/rust-secp256k1/pull/259)
* [remove endomorphism feature, following upstream in enabling it by default](https://github.com/rust-bitcoin/rust-secp256k1/pull/257)

# 0.19.0 - 2020-08-27

* **Update MSRV to 1.29.0**

# 0.18.0 - 2020-08-26

* Add feature-gated `bitcoin_hashes` dependency and [`ThirtyTwoByteHash` trait](https://github.com/rust-bitcoin/rust-secp256k1/pull/206/)
* Add feature-gated [global static context](https://github.com/rust-bitcoin/rust-secp256k1/pull/224)
* Allow [all-zero messages](https://github.com/rust-bitcoin/rust-secp256k1/pull/207) to be constructed
* Bump rust-secp-sys to 0.2.0

# 0.17.2
- Fix linking in the `fuzztarget` feature.

# 0.17.1

- Correctly prefix the secp256k1-sys links field in Cargo.toml.

# 0.17.0

- Move FFI into secp256k1-sys crate.
- Add `external-symbols` feature for not building upstream.
- Add functions to create a context from a raw pointer.
- Support passing custom hash functions to ECDH.
- Wrap Secp256k1 from raw context in a ManuallyDrop.

# 0.15.4 - 2019-09-06

- Add `rand-std` feature.
- Pin the cc build-dep version to `< 1.0.42` to remain
  compatible with rustc 1.22.0.
- Changed all `as_*ptr()` to a new safer `CPtr` trait

# 0.15.2 - 2019-08-08

- Add feature `lowmemory` that reduces the EC mult window size to require
  significantly less memory for the validation context (~680B instead of
  ~520kB), at the cost of slower validation. It does not affect the speed of
  signing, nor the size of the signing context.

# 0.15.0 - 2019-07-25

* Implement hex human-readable serde for PublicKey
* Implement fmt::LowerHex for SecretKey and PublicKey
* Relax `cc` dependency requirements
* Add links manifest key to prevent cross-version linkage

# 0.14.1 - 2019-07-14

* Implemented FFI functions: `secp256k1_context_create` and `secp256k1_context_destroy` in rust.

# 0.14.0 - 2019-07-08

* [Feature-gate endormorphism optimization](https://github.com/rust-bitcoin/rust-secp256k1/pull/120)
  because of a lack of clarity with respect to patents
* Got full no-std support including eliminating all use of libc in C bindings.
  [PR 1](https://github.com/rust-bitcoin/rust-secp256k1/pull/115)
  [PR 2](https://github.com/rust-bitcoin/rust-secp256k1/pull/125).
  This library should be usable in bare-metal environments and with rust-wasm.
  Thanks to Elichai Turkel for driving this forward!
* Update upstream libsecp256k1 version to 143dc6e9ee31852a60321b23eea407d2006171da

# 0.13.0 - 2019-05-21

* Update minimum supported rust compiler 1.22.
* Replace `serialize_der` function with `SerializedSignature` struct.
* Allow building without a standard library (`no_std`). `std` feature is on by default.
* Add human readable serialization to `Signatures` and `SecretKeys`.
* Stop displaying 0 bytes if a `Signature` is less than 72 bytes.
* Only compile recovery module if feature `recovery` is set (non-default).
* Update `rand` dependency from 0.4 to 0.6 and add `rand_core` 0.4 dependency.
* Relax `cc` dependency requirements.

# 0.12.2 - 2019-01-18

* Fuzzer bug fix

# 0.12.1 - 2019-01-15

* Minor bug fixes
* Fixed `cc` crate version to maintain minimum compiler version without breakage
* Removed `libc` dependency as it our uses have been subsumed into stdlib

# 0.12.0 - 2018-12-03

* **Overhaul API to remove context object when no precomputation is needed**
* Add `ThirtyTwoByteHash` trait which allows infallible conversions to `Message`s
* Disallow 0-valued `Message` objects since signatures on them are forgeable for all keys
* Remove `ops::Index` implementations for `Signature`
* Remove depecated constants and unsafe `ZERO_KEY` constant

# 0.11.5 - 2018-11-09

* Use `pub extern crate` to export dependencies whose types are exported

# 0.11.4 - 2018-11-04

* Add `FromStr` and `Display` for `Signature` and both key types
* Fix `build.rs` for Windows and rustfmt configuration for docs.rs
* Correct endianness issue for `Signature` `Debug` output

# 0.11.3 - 2018-10-28

* No changes, just fixed docs.rs configuration

# 0.11.2 - 2018-09-11

* Correct endianness issue in RFC6979 nonce generation

# 0.11.1 - 2018-08-22

* Put `PublicKey::combine` back because it is currently needed to implement Lightning BOLT 3

# 0.11.0 - 2018-08-22

* Update `rand` to 0.4 and `gcc` 0.3 to `cc` 1.0. (`rand` 0.5 exists but has a lot of breaking changes and no longer compiles with 1.14.0.)
* Remove `PublicKey::combine` from API since it cannot be used with anything else in the API
* Detect whether 64-bit compilation is possible, and do it if we can (big performance improvement)

# 0.10.0 - 2018-07-25

* A [complete API overhaul](https://github.com/rust-bitcoin/rust-secp256k1/pull/27) to move many runtime errors into compiletime errors
* Update [libsecp256k1 to `1e6f1f5ad5e7f1e3ef79313ec02023902bf8`](https://github.com/rust-bitcoin/rust-secp256k1/pull/32). Should be no visible changes.
* [Remove `PublicKey::new()` and `PublicKey::is_valid()`](https://github.com/rust-bitcoin/rust-secp256k1/pull/37) since `new` was unsafe and it should now be impossible to create invalid `PublicKey` objects through the API
* [Reintroduce serde support](https://github.com/rust-bitcoin/rust-secp256k1/pull/38) behind a feature gate using serde 1.0
* Clean up build process and various typos


