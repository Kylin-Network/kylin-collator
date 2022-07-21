// Bitcoin secp256k1 bindings
// Written in 2014 by
//   Dawid Ciężarkiewicz
//   Andrew Poelstra
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Provides a signing function that allows recovering the public key from the
//! signature.
//!

use core::ptr;
use key;
use super::ffi as super_ffi;
use self::super_ffi::CPtr;
use ffi::recovery as ffi;
use super::*;
use {Verification, Secp256k1, Signing, Message};

/// A tag used for recovering the public key from a compact signature.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RecoveryId(i32);

/// An ECDSA signature with a recovery ID for pubkey recovery.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RecoverableSignature(ffi::RecoverableSignature);

impl RecoveryId {
#[inline]
/// Allows library users to create valid recovery IDs from i32.
pub fn from_i32(id: i32) -> Result<RecoveryId, Error> {
    match id {
        0 | 1 | 2 | 3 => Ok(RecoveryId(id)),
        _ => Err(Error::InvalidRecoveryId)
    }
}

#[inline]
/// Allows library users to convert recovery IDs to i32.
pub fn to_i32(self) -> i32 {
    self.0
}
}

impl RecoverableSignature {
    #[inline]
    /// Converts a compact-encoded byte slice to a signature. This
    /// representation is nonstandard and defined by the libsecp256k1 library.
    pub fn from_compact(data: &[u8], recid: RecoveryId) -> Result<RecoverableSignature, Error> {
        if data.is_empty() {return Err(Error::InvalidSignature);}

        let mut ret = ffi::RecoverableSignature::new();

        unsafe {
            if data.len() != 64 {
                Err(Error::InvalidSignature)
            } else if ffi::secp256k1_ecdsa_recoverable_signature_parse_compact(
                super_ffi::secp256k1_context_no_precomp,
                &mut ret,
                data.as_c_ptr(),
                recid.0,
            ) == 1
            {
                Ok(RecoverableSignature(ret))
            } else {
                Err(Error::InvalidSignature)
            }
        }
    }

    /// Obtains a raw pointer suitable for use with FFI functions.
    #[inline]
    pub fn as_ptr(&self) -> *const ffi::RecoverableSignature {
        &self.0
    }

    /// Obtains a raw mutable pointer suitable for use with FFI functions.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut ffi::RecoverableSignature {
        &mut self.0
    }

    #[inline]
    /// Serializes the recoverable signature in compact format.
    pub fn serialize_compact(&self) -> (RecoveryId, [u8; 64]) {
        let mut ret = [0u8; 64];
        let mut recid = 0i32;
        unsafe {
            let err = ffi::secp256k1_ecdsa_recoverable_signature_serialize_compact(
                super_ffi::secp256k1_context_no_precomp,
                ret.as_mut_c_ptr(),
                &mut recid,
                self.as_c_ptr(),
            );
            assert!(err == 1);
        }
        (RecoveryId(recid), ret)
    }

    /// Converts a recoverable signature to a non-recoverable one (this is needed
    /// for verification).
    #[inline]
    pub fn to_standard(&self) -> Signature {
        unsafe {
            let mut ret = super_ffi::Signature::new();
            let err = ffi::secp256k1_ecdsa_recoverable_signature_convert(
                super_ffi::secp256k1_context_no_precomp,
                &mut ret,
                self.as_c_ptr(),
            );
            assert!(err == 1);
            Signature(ret)
        }
    }

    /// Determines the public key for which this [`Signature`] is valid for `msg`. Requires a
    /// verify-capable context.
    #[inline]
    #[cfg(feature = "global-context")]
    #[cfg_attr(docsrs, doc(cfg(feature = "global-context")))]
    pub fn recover(&self, msg: &Message) -> Result<key::PublicKey, Error> {
        SECP256K1.recover_ecdsa(msg, self)
    }
}


impl CPtr for RecoverableSignature {
    type Target = ffi::RecoverableSignature;
    fn as_c_ptr(&self) -> *const Self::Target {
        self.as_ptr()
    }

    fn as_mut_c_ptr(&mut self) -> *mut Self::Target {
        self.as_mut_ptr()
    }
}

/// Creates a new recoverable signature from a FFI one.
impl From<ffi::RecoverableSignature> for RecoverableSignature {
    #[inline]
    fn from(sig: ffi::RecoverableSignature) -> RecoverableSignature {
        RecoverableSignature(sig)
    }
}

impl<C: Signing> Secp256k1<C> {
    /// Constructs a signature for `msg` using the secret key `sk` and RFC6979 nonce.
    /// Requires a signing-capable context.
    #[deprecated(since = "0.21.0", note = "Use sign_ecdsa_recoverable instead.")]
    pub fn sign_recoverable(&self, msg: &Message, sk: &key::SecretKey) -> RecoverableSignature {
        self.sign_ecdsa_recoverable(msg, sk)
    }

    /// Constructs a signature for `msg` using the secret key `sk` and RFC6979 nonce
    /// Requires a signing-capable context.
    pub fn sign_ecdsa_recoverable(&self, msg: &Message, sk: &key::SecretKey) -> RecoverableSignature {
        let mut ret = ffi::RecoverableSignature::new();
        unsafe {
            // We can assume the return value because it's not possible to construct
            // an invalid signature from a valid `Message` and `SecretKey`
            assert_eq!(
                ffi::secp256k1_ecdsa_sign_recoverable(
                    self.ctx,
                    &mut ret,
                    msg.as_c_ptr(),
                    sk.as_c_ptr(),
                    super_ffi::secp256k1_nonce_function_rfc6979,
                    ptr::null()
                ),
                1
            );
        }

        RecoverableSignature::from(ret)
    }
}

impl<C: Verification> Secp256k1<C> {
    /// Determines the public key for which `sig` is a valid signature for
    /// `msg`. Requires a verify-capable context.
    #[deprecated(since = "0.21.0", note = "Use recover_ecdsa instead.")]
    pub fn recover(&self, msg: &Message, sig: &RecoverableSignature) -> Result<key::PublicKey, Error> {
        self.recover_ecdsa(msg, sig)
    }

    /// Determines the public key for which `sig` is a valid signature for
    /// `msg`. Requires a verify-capable context.
    pub fn recover_ecdsa(&self, msg: &Message, sig: &RecoverableSignature)
                   -> Result<key::PublicKey, Error> {

        unsafe {
            let mut pk = super_ffi::PublicKey::new();
            if ffi::secp256k1_ecdsa_recover(self.ctx, &mut pk,
                                            sig.as_c_ptr(), msg.as_c_ptr()) != 1 {
                return Err(Error::InvalidSignature);
            }
            Ok(key::PublicKey::from(pk))
        }
    }
}


#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use rand::{RngCore, thread_rng};
    use key::SecretKey;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    #[test]
    #[cfg(all(feature="std", feature = "rand-std"))]
    fn capabilities() {
        let sign = Secp256k1::signing_only();
        let vrfy = Secp256k1::verification_only();
        let full = Secp256k1::new();

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();

        // Try key generation
        let (sk, pk) = full.generate_keypair(&mut thread_rng());

        // Try signing
        assert_eq!(sign.sign_ecdsa_recoverable(&msg, &sk), full.sign_ecdsa_recoverable(&msg, &sk));
        let sigr = full.sign_ecdsa_recoverable(&msg, &sk);

        // Try pk recovery
        assert!(vrfy.recover_ecdsa(&msg, &sigr).is_ok());
        assert!(full.recover_ecdsa(&msg, &sigr).is_ok());

        assert_eq!(vrfy.recover_ecdsa(&msg, &sigr),
                   full.recover_ecdsa(&msg, &sigr));
        assert_eq!(full.recover_ecdsa(&msg, &sigr), Ok(pk));
    }

    #[test]
    fn recid_sanity_check() {
        let one = RecoveryId(1);
        assert_eq!(one, one.clone());
    }

    #[test]
    #[cfg(not(fuzzing))]  // fixed sig vectors can't work with fuzz-sigs
    #[cfg(all(feature="std", feature = "rand-std"))]
    fn sign() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());
        let one: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

        let sk = SecretKey::from_slice(&one).unwrap();
        let msg = Message::from_slice(&one).unwrap();

        let sig = s.sign_ecdsa_recoverable(&msg, &sk);
        assert_eq!(Ok(sig), RecoverableSignature::from_compact(&[
            0x66, 0x73, 0xff, 0xad, 0x21, 0x47, 0x74, 0x1f,
            0x04, 0x77, 0x2b, 0x6f, 0x92, 0x1f, 0x0b, 0xa6,
            0xaf, 0x0c, 0x1e, 0x77, 0xfc, 0x43, 0x9e, 0x65,
            0xc3, 0x6d, 0xed, 0xf4, 0x09, 0x2e, 0x88, 0x98,
            0x4c, 0x1a, 0x97, 0x16, 0x52, 0xe0, 0xad, 0xa8,
            0x80, 0x12, 0x0e, 0xf8, 0x02, 0x5e, 0x70, 0x9f,
            0xff, 0x20, 0x80, 0xc4, 0xa3, 0x9a, 0xae, 0x06,
            0x8d, 0x12, 0xee, 0xd0, 0x09, 0xb6, 0x8c, 0x89],
            RecoveryId(1)))
    }

    #[test]
    #[cfg(all(feature="std", feature = "rand-std"))]
    fn sign_and_verify_fail() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();

        let (sk, pk) = s.generate_keypair(&mut thread_rng());

        let sigr = s.sign_ecdsa_recoverable(&msg, &sk);
        let sig = sigr.to_standard();

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();
        assert_eq!(s.verify_ecdsa(&msg, &sig, &pk), Err(Error::IncorrectSignature));

        let recovered_key = s.recover_ecdsa(&msg, &sigr).unwrap();
        assert!(recovered_key != pk);
    }

    #[test]
    #[cfg(all(feature="std", feature = "rand-std"))]
    fn sign_with_recovery() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();

        let (sk, pk) = s.generate_keypair(&mut thread_rng());

        let sig = s.sign_ecdsa_recoverable(&msg, &sk);

        assert_eq!(s.recover_ecdsa(&msg, &sig), Ok(pk));
    }

    #[test]
    #[cfg(all(feature="std", feature = "rand-std"))]
    fn bad_recovery() {
        let mut s = Secp256k1::new();
        s.randomize(&mut thread_rng());

        let msg = Message::from_slice(&[0x55; 32]).unwrap();

        // Zero is not a valid sig
        let sig = RecoverableSignature::from_compact(&[0; 64], RecoveryId(0)).unwrap();
        assert_eq!(s.recover_ecdsa(&msg, &sig), Err(Error::InvalidSignature));
        // ...but 111..111 is
        let sig = RecoverableSignature::from_compact(&[1; 64], RecoveryId(0)).unwrap();
        assert!(s.recover_ecdsa(&msg, &sig).is_ok());
    }

    #[test]
    fn test_debug_output() {
        let sig = RecoverableSignature::from_compact(&[
            0x66, 0x73, 0xff, 0xad, 0x21, 0x47, 0x74, 0x1f,
            0x04, 0x77, 0x2b, 0x6f, 0x92, 0x1f, 0x0b, 0xa6,
            0xaf, 0x0c, 0x1e, 0x77, 0xfc, 0x43, 0x9e, 0x65,
            0xc3, 0x6d, 0xed, 0xf4, 0x09, 0x2e, 0x88, 0x98,
            0x4c, 0x1a, 0x97, 0x16, 0x52, 0xe0, 0xad, 0xa8,
            0x80, 0x12, 0x0e, 0xf8, 0x02, 0x5e, 0x70, 0x9f,
            0xff, 0x20, 0x80, 0xc4, 0xa3, 0x9a, 0xae, 0x06,
            0x8d, 0x12, 0xee, 0xd0, 0x09, 0xb6, 0x8c, 0x89],
            RecoveryId(1)).unwrap();
        assert_eq!(&format!("{:?}", sig), "RecoverableSignature(6673ffad2147741f04772b6f921f0ba6af0c1e77fc439e65c36dedf4092e88984c1a971652e0ada880120ef8025e709fff2080c4a39aae068d12eed009b68c8901)");
    }

    #[test]
    fn test_recov_sig_serialize_compact() {
        let recid_in = RecoveryId(1);
        let bytes_in = &[
            0x66, 0x73, 0xff, 0xad, 0x21, 0x47, 0x74, 0x1f,
            0x04, 0x77, 0x2b, 0x6f, 0x92, 0x1f, 0x0b, 0xa6,
            0xaf, 0x0c, 0x1e, 0x77, 0xfc, 0x43, 0x9e, 0x65,
            0xc3, 0x6d, 0xed, 0xf4, 0x09, 0x2e, 0x88, 0x98,
            0x4c, 0x1a, 0x97, 0x16, 0x52, 0xe0, 0xad, 0xa8,
            0x80, 0x12, 0x0e, 0xf8, 0x02, 0x5e, 0x70, 0x9f,
            0xff, 0x20, 0x80, 0xc4, 0xa3, 0x9a, 0xae, 0x06,
            0x8d, 0x12, 0xee, 0xd0, 0x09, 0xb6, 0x8c, 0x89];
        let sig = RecoverableSignature::from_compact(
            bytes_in,
            recid_in,
        ).unwrap();
        let (recid_out, bytes_out) = sig.serialize_compact();
        assert_eq!(recid_in, recid_out);
        assert_eq!(&bytes_in[..], &bytes_out[..]);
    }

    #[test]
    fn test_recov_id_conversion_between_i32() {
        assert!(RecoveryId::from_i32(-1).is_err());
        assert!(RecoveryId::from_i32(0).is_ok());
        assert!(RecoveryId::from_i32(1).is_ok());
        assert!(RecoveryId::from_i32(2).is_ok());
        assert!(RecoveryId::from_i32(3).is_ok());
        assert!(RecoveryId::from_i32(4).is_err());
        let id0 = RecoveryId::from_i32(0).unwrap();
        assert_eq!(id0.to_i32(), 0);
        let id1 = RecoveryId(1);
        assert_eq!(id1.to_i32(), 1);
    }
}


#[cfg(all(test, feature = "unstable"))]
mod benches {
    use rand::{thread_rng, RngCore};
    use test::{Bencher, black_box};
    use super::{Message, Secp256k1};

    #[bench]
    pub fn bench_recover(bh: &mut Bencher) {
        let s = Secp256k1::new();
        let mut msg = [0u8; 32];
        thread_rng().fill_bytes(&mut msg);
        let msg = Message::from_slice(&msg).unwrap();
        let (sk, _) = s.generate_keypair(&mut thread_rng());
        let sig = s.sign_ecdsa_recoverable(&msg, &sk);

        bh.iter(|| {
            let res = s.recover_ecdsa(&msg, &sig).unwrap();
            black_box(res);
        });
    }
}
