// This file is part of Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Decodable variant of the RuntimeMetadata.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

cfg_if::cfg_if! {
	if #[cfg(feature = "std")] {
		use codec::{Decode, Error, Input};
		use serde::{
			Deserialize,
			Serialize,
		};
	} else {
		extern crate alloc;
		use alloc::vec::Vec;
	}
}

use codec::{Encode, Output};

/// A type that decodes to a different type than it encodes.
#[cfg(any(
	feature = "v13",
	feature = "v12",
	feature = "v11",
	feature = "v10",
	feature = "v9",
	feature = "v8",
	feature = "legacy"
))]
pub mod decode_different;

/// Metadata v8
#[cfg(feature = "v8")]
pub mod v8;

/// Metadata v9
#[cfg(feature = "v9")]
pub mod v9;

/// Metadata v10
#[cfg(feature = "v10")]
pub mod v10;

/// Metadata v11
#[cfg(feature = "v11")]
pub mod v11;

/// Metadata v12
#[cfg(feature = "v12")]
pub mod v12;

/// Metadata v13
#[cfg(feature = "v13")]
pub mod v13;

/// Metadata v14
#[cfg(feature = "v14")]
pub mod v14;

// Reexport all the types from the latest version.
//
// When a new version becomes available, update this.
#[cfg(feature = "v14")]
pub use self::v14::*;

/// Metadata prefixed by a u32 for reserved usage
#[derive(Eq, Encode, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize, Debug))]
pub struct RuntimeMetadataPrefixed(pub u32, pub RuntimeMetadata);

impl Into<Vec<u8>> for RuntimeMetadataPrefixed {
	fn into(self) -> Vec<u8> {
		self.encode()
	}
}

/// The metadata of a runtime.
/// The version ID encoded/decoded through
/// the enum nature of `RuntimeMetadata`.
#[derive(Eq, Encode, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize, Debug))]
pub enum RuntimeMetadata {
	/// Unused; enum filler.
	V0(RuntimeMetadataDeprecated),
	/// Version 1 for runtime metadata. No longer used.
	V1(RuntimeMetadataDeprecated),
	/// Version 2 for runtime metadata. No longer used.
	V2(RuntimeMetadataDeprecated),
	/// Version 3 for runtime metadata. No longer used.
	V3(RuntimeMetadataDeprecated),
	/// Version 4 for runtime metadata. No longer used.
	V4(RuntimeMetadataDeprecated),
	/// Version 5 for runtime metadata. No longer used.
	V5(RuntimeMetadataDeprecated),
	/// Version 6 for runtime metadata. No longer used.
	V6(RuntimeMetadataDeprecated),
	/// Version 7 for runtime metadata. No longer used.
	V7(RuntimeMetadataDeprecated),
	/// Version 8 for runtime metadata.
	#[cfg(any(feature = "v8", feature = "legacy"))]
	V8(v8::RuntimeMetadataV8),
	/// Version 8 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v8"))]
	V8(OpaqueMetadata),
	/// Version 9 for runtime metadata.
	#[cfg(any(feature = "v9", feature = "legacy"))]
	V9(v9::RuntimeMetadataV9),
	/// Version 9 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v9"))]
	V9(OpaqueMetadata),
	/// Version 10 for runtime metadata.
	#[cfg(any(feature = "v10", feature = "legacy"))]
	V10(v10::RuntimeMetadataV10),
	/// Version 10 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v10"))]
	V10(OpaqueMetadata),
	/// Version 11 for runtime metadata.
	#[cfg(any(feature = "v11", feature = "legacy"))]
	V11(v11::RuntimeMetadataV11),
	/// Version 11 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v11"))]
	V11(OpaqueMetadata),
	/// Version 12 for runtime metadata
	#[cfg(any(feature = "v12", feature = "legacy"))]
	V12(v12::RuntimeMetadataV12),
	/// Version 12 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v12"))]
	V12(OpaqueMetadata),
	/// Version 13 for runtime metadata.
	#[cfg(any(feature = "v13", feature = "legacy"))]
	V13(v13::RuntimeMetadataV13),
	/// Version 13 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v13"))]
	V13(OpaqueMetadata),
	/// Version 14 for runtime metadata.
	#[cfg(feature = "v14")]
	V14(v14::RuntimeMetadataV14),
	/// Version 14 for runtime metadata, as raw encoded bytes.
	#[cfg(not(feature = "v14"))]
	V14(OpaqueMetadata),
}

impl RuntimeMetadata {
	/// Get the version number of the metadata.
	pub fn version(&self) -> u32 {
		match self {
			RuntimeMetadata::V0(_) => 0,
			RuntimeMetadata::V1(_) => 1,
			RuntimeMetadata::V2(_) => 2,
			RuntimeMetadata::V3(_) => 3,
			RuntimeMetadata::V4(_) => 4,
			RuntimeMetadata::V5(_) => 5,
			RuntimeMetadata::V6(_) => 6,
			RuntimeMetadata::V7(_) => 7,
			RuntimeMetadata::V8(_) => 8,
			RuntimeMetadata::V9(_) => 9,
			RuntimeMetadata::V10(_) => 10,
			RuntimeMetadata::V11(_) => 11,
			RuntimeMetadata::V12(_) => 12,
			RuntimeMetadata::V13(_) => 13,
			RuntimeMetadata::V14(_) => 14,
		}
	}
}

/// Stores the encoded `RuntimeMetadata` as raw bytes.
#[derive(Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Decode, Serialize, Deserialize, Debug))]
pub struct OpaqueMetadata(pub Vec<u8>);

/// Enum that should fail.
#[derive(Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub enum RuntimeMetadataDeprecated {}

impl Encode for RuntimeMetadataDeprecated {
	fn encode_to<W: Output + ?Sized>(&self, _dest: &mut W) {}
}

impl codec::EncodeLike for RuntimeMetadataDeprecated {}

#[cfg(feature = "std")]
impl Decode for RuntimeMetadataDeprecated {
	fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
		Err("Decoding is not supported".into())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use std::fs;

	fn load_metadata(version: u32) -> Vec<u8> {
		fs::read(format!("./test_data/ksm_metadata_v{}.bin", version)).unwrap()
	}

	#[test]
	fn should_decode_metadatav9() {
		let meta: RuntimeMetadataPrefixed =
			Decode::decode(&mut load_metadata(9).as_slice()).unwrap();
		assert!(matches!(meta.1, RuntimeMetadata::V9(_)));
	}

	#[test]
	fn should_decode_metadatav10() {
		let meta: RuntimeMetadataPrefixed =
			Decode::decode(&mut load_metadata(10).as_slice()).unwrap();
		assert!(matches!(meta.1, RuntimeMetadata::V10(_)));
	}

	#[test]
	fn should_decode_metadatav11() {
		let meta: RuntimeMetadataPrefixed =
			Decode::decode(&mut load_metadata(11).as_slice()).unwrap();
		assert!(matches!(meta.1, RuntimeMetadata::V11(_)));
	}

	#[test]
	fn should_decode_metadatav12() {
		let meta: RuntimeMetadataPrefixed =
			Decode::decode(&mut load_metadata(12).as_slice()).unwrap();
		assert!(matches!(meta.1, RuntimeMetadata::V12(_)));
	}

	#[test]
	fn should_decode_metadatav13() {
		let meta: RuntimeMetadataPrefixed =
			Decode::decode(&mut load_metadata(13).as_slice()).unwrap();
		assert!(matches!(meta.1, RuntimeMetadata::V13(_)));
	}

	#[test]
	fn should_decode_metadatav14() {
		let meta: RuntimeMetadataPrefixed =
			Decode::decode(&mut load_metadata(14).as_slice()).unwrap();
		assert!(matches!(meta.1, RuntimeMetadata::V14(_)));
	}
}
