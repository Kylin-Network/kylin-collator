
//! Kylin-airdrop module provide a mapping between Substrate accounts and
//! original Etherum account where initial KYL tokens resides and gives
//! the ability to recieve airdrop tokens.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::Encode;
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, IsType, OnKilledAccount},
	transactional,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use module_evm_utility_macro::keccak256;
use module_support::{AddressMapping, EVMAccountsManager};
use orml_traits::currency::TransferAll;
use primitives::{evm::EthKYLAddress, to_bytes, AccountIndex};
use sp_core::crypto::AccountId32;
use sp_core::{H160, H256};
use sp_io::{
	crypto::secp256k1_ecdsa_recover,
	hashing::{blake2_256, keccak_256},
};
use sp_runtime::{
	traits::{LookupError, StaticLookup, Zero},
	MultiAddress,
};
use sp_std::{marker::PhantomData, vec::Vec};

mod mock;
mod tests;
pub mod weights;

pub use module::*;
pub use weights::WeightInfo;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
pub type Eip712Signature = [u8; 65];

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency for managing Evm account assets.
		type Currency: Currency<Self::AccountId>;

		/// Mapping from address to account id.
		type AddressMapping: AddressMapping<Self::AccountId>;

		/// Chain ID of EVM.
		#[pallet::constant]
		type ChainId: Get<u64>;

		/// Merge free balance from source to dest.
		type TransferAll: TransferAll<Self::AccountId>;

		/// Weight information for the extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Mapping between Substrate accounts and EVM accounts
		/// claim account.
		ClaimAccount {
			account_id: T::AccountId,
			evm_address: EthKYLAddress,
		},
	}

	/// Error for evm accounts module.
	#[pallet::error]
	pub enum Error<T> {
		/// AccountId has mapped
		AccountIdHasMapped,
		/// Eth address has mapped
		EthAddressHasMapped,
		/// Bad signature
		BadSignature,
		/// Invalid signature
		InvalidSignature,
		/// Account ref count is not zero
		NonZeroRefCount,
	}

	/// The Substrate Account for EthKYLAddresses
	///
	/// Accounts: map EthKYLAddress => Option<AccountId>
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageMap<_, Twox64Concat, EthKYLAddress, T::AccountId, OptionQuery>;

	/// The EthKYLAddress for Substrate Accounts
	///
	/// EthKYLAddresses: map AccountId => Option<EthKYLAddress>
	#[pallet::storage]
	#[pallet::getter(fn evm_addresses)]
	pub type EthKYLAddresses<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, EthKYLAddress, OptionQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim account mapping between Substrate accounts and EVM accounts.
		/// Ensure eth_address has not been mapped.
		///
		/// - `eth_address`: The address to bind to the caller's account
		/// - `eth_signature`: A signature generated by the address to prove ownership
		#[pallet::weight(T::WeightInfo::claim_account())]
		#[transactional]
		pub fn claim_account(
			origin: OriginFor<T>,
			eth_address: EthKYLAddress,
			eth_signature: Eip712Signature,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// ensure account_id and eth_address has not been mapped
			ensure!(!EthKYLAddresses::<T>::contains_key(&who), Error::<T>::AccountIdHasMapped);
			ensure!(
				!Accounts::<T>::contains_key(eth_address),
				Error::<T>::EthAddressHasMapped
			);

			// recover evm address from signature
			let address = Self::verify_eip712_signature(&who, &eth_signature).ok_or(Error::<T>::BadSignature)?;
			ensure!(eth_address == address, Error::<T>::InvalidSignature);

			// check if the evm padded address already exists
			let account_id = T::AddressMapping::get_account_id(&eth_address);
			if frame_system::Pallet::<T>::account_exists(&account_id) {
				// merge balance from `evm padded address` to `origin`
				T::TransferAll::transfer_all(&account_id, &who)?;
			}

			Accounts::<T>::insert(eth_address, &who);
			EthKYLAddresses::<T>::insert(&who, eth_address);

			Self::deposit_event(Event::ClaimAccount {
				account_id: who,
				evm_address: eth_address,
			});

			Ok(())
		}

		/// Claim account mapping between Substrate accounts and a generated EVM
		/// address based off of those accounts.
		/// Ensure eth_address has not been mapped
		#[pallet::weight(T::WeightInfo::claim_default_account())]
		pub fn claim_default_account(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let _ = Self::do_claim_default_evm_address(who)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
	// Returns an Etherum public key derived from an Ethereum secret key.
	pub fn eth_public(secret: &libsecp256k1::SecretKey) -> libsecp256k1::PublicKey {
		libsecp256k1::PublicKey::from_secret_key(secret)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
	// Returns an Etherum address derived from an Ethereum secret key.
	// Only for tests
	pub fn eth_address(secret: &libsecp256k1::SecretKey) -> EthKYLAddress {
		EthKYLAddress::from_slice(&keccak_256(&Self::eth_public(secret).serialize()[1..65])[12..])
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "std"))]
	// Constructs a message and signs it.
	pub fn eth_sign(secret: &libsecp256k1::SecretKey, who: &T::AccountId) -> Eip712Signature {
		let msg = keccak_256(&Self::eip712_signable_message(who));
		let (sig, recovery_id) = libsecp256k1::sign(&libsecp256k1::Message::parse(&msg), secret);
		let mut r = [0u8; 65];
		r[0..64].copy_from_slice(&sig.serialize()[..]);
		r[64] = recovery_id.serialize();
		r
	}

	fn verify_eip712_signature(who: &T::AccountId, sig: &[u8; 65]) -> Option<H160> {
		let msg = Self::eip712_signable_message(who);
		let msg_hash = keccak_256(msg.as_slice());

		recover_signer(sig, &msg_hash)
	}

	// Eip-712 message to be signed
	fn eip712_signable_message(who: &T::AccountId) -> Vec<u8> {
		let domain_separator = Self::evm_account_domain_separator();
		let payload_hash = Self::evm_account_payload_hash(who);

		let mut msg = b"\x19\x01".to_vec();
		msg.extend_from_slice(&domain_separator);
		msg.extend_from_slice(&payload_hash);
		msg
	}

	fn evm_account_payload_hash(who: &T::AccountId) -> [u8; 32] {
		let tx_type_hash = keccak256!("Transaction(bytes substrateAddress)");
		let mut tx_msg = tx_type_hash.to_vec();
		tx_msg.extend_from_slice(&keccak_256(&who.encode()));
		keccak_256(tx_msg.as_slice())
	}

	fn evm_account_domain_separator() -> [u8; 32] {
		let domain_hash = keccak256!("EIP712Domain(string name,string version,uint256 chainId,bytes32 salt)");
		let mut domain_seperator_msg = domain_hash.to_vec();
		domain_seperator_msg.extend_from_slice(keccak256!("Kylin airdrop claim")); // name
		domain_seperator_msg.extend_from_slice(keccak256!("1")); // version
		domain_seperator_msg.extend_from_slice(&to_bytes(T::ChainId::get())); // chain id
		domain_seperator_msg.extend_from_slice(frame_system::Pallet::<T>::block_hash(T::BlockNumber::zero()).as_ref()); // genesis block hash
		keccak_256(domain_seperator_msg.as_slice())
	}

	fn do_claim_default_evm_address(who: T::AccountId) -> Result<EthKYLAddress, DispatchError> {
		// ensure account_id has not been mapped
		ensure!(!EthKYLAddresses::<T>::contains_key(&who), Error::<T>::AccountIdHasMapped);

		let eth_address = T::AddressMapping::get_or_create_evm_address(&who);

		Ok(eth_address)
	}
}

fn recover_signer(sig: &[u8; 65], msg_hash: &[u8; 32]) -> Option<H160> {
	secp256k1_ecdsa_recover(sig, msg_hash)
		.map(|pubkey| H160::from(H256::from_slice(&keccak_256(&pubkey))))
		.ok()
}

// Creates a an EthKYLAddress from an AccountId by appending the bytes "evm:" to
// the account_id and hashing it.
fn account_to_default_evm_address(account_id: &impl Encode) -> EthKYLAddress {
	let payload = (b"evm:", account_id);
	EthKYLAddress::from_slice(&payload.using_encoded(blake2_256)[0..20])
}

pub struct EthKYLAddressMapping<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AddressMapping<T::AccountId> for EthKYLAddressMapping<T>
where
	T::AccountId: IsType<AccountId32>,
{
	// Returns the AccountId used to generate the given EthKYLAddress.
	fn get_account_id(address: &EthKYLAddress) -> T::AccountId {
		if let Some(acc) = Accounts::<T>::get(address) {
			acc
		} else {
			let mut data: [u8; 32] = [0u8; 32];
			data[0..4].copy_from_slice(b"evm:");
			data[4..24].copy_from_slice(&address[..]);
			AccountId32::from(data).into()
		}
	}

	// Returns the EthKYLAddress associated with a given AccountId or the
	// underlying EthKYLAddress of the AccountId.
	// Returns None if there is no EthKYLAddress associated with the AccountId
	// and there is no underlying EthKYLAddress in the AccountId.
	fn get_evm_address(account_id: &T::AccountId) -> Option<EthKYLAddress> {
		// Return the EthKYLAddress if a mapping to account_id exists
		EthKYLAddresses::<T>::get(account_id).or_else(|| {
			let data: &[u8] = account_id.into_ref().as_ref();
			// Return the underlying EVM address if it exists otherwise return None
			if data.starts_with(b"evm:") {
				Some(EthKYLAddress::from_slice(&data[4..24]))
			} else {
				None
			}
		})
	}

	// Returns the ETH address associated with an account ID and generates an
	// account mapping if no association exists.
	fn get_or_create_evm_address(account_id: &T::AccountId) -> EthKYLAddress {
		Self::get_evm_address(account_id).unwrap_or_else(|| {
			let addr = account_to_default_evm_address(account_id);

			// create reverse mapping
			Accounts::<T>::insert(&addr, &account_id);
			EthKYLAddresses::<T>::insert(&account_id, &addr);

			Pallet::<T>::deposit_event(Event::ClaimAccount {
				account_id: account_id.clone(),
				evm_address: addr,
			});

			addr
		})
	}

	// Returns the default EVM address associated with an account ID.
	fn get_default_evm_address(account_id: &T::AccountId) -> EthKYLAddress {
		account_to_default_evm_address(account_id)
	}

	// Returns true if a given AccountId is associated with a given EthKYLAddress
	// and false if is not.
	fn is_linked(account_id: &T::AccountId, evm: &EthKYLAddress) -> bool {
		Self::get_evm_address(account_id).as_ref() == Some(evm)
			|| &account_to_default_evm_address(account_id.into_ref()) == evm
	}
}

pub struct CallKillAccount<T>(PhantomData<T>);
impl<T: Config> OnKilledAccount<T::AccountId> for CallKillAccount<T> {
	fn on_killed_account(who: &T::AccountId) {
		// remove mapping created by `claim_account` or `get_or_create_evm_address`
		if let Some(evm_addr) = Pallet::<T>::evm_addresses(who) {
			Accounts::<T>::remove(evm_addr);
			EthKYLAddresses::<T>::remove(who);
		}
	}
}

impl<T: Config> StaticLookup for Pallet<T> {
	type Source = MultiAddress<T::AccountId, AccountIndex>;
	type Target = T::AccountId;

	fn lookup(a: Self::Source) -> Result<Self::Target, LookupError> {
		match a {
			MultiAddress::Address20(i) => Ok(T::AddressMapping::get_account_id(&EthKYLAddress::from_slice(&i))),
			_ => Err(LookupError),
		}
	}

	fn unlookup(a: Self::Target) -> Self::Source {
		MultiAddress::Id(a)
	}
}

impl<T: Config> EVMAccountsManager<T::AccountId> for Pallet<T> {
	/// Returns the AccountId used to generate the given EthKYLAddress.
	fn get_account_id(address: &EthKYLAddress) -> T::AccountId {
		T::AddressMapping::get_account_id(address)
	}

	/// Returns the EthKYLAddress associated with a given AccountId or the underlying EthKYLAddress of the
	/// AccountId.
	fn get_evm_address(account_id: &T::AccountId) -> Option<EthKYLAddress> {
		T::AddressMapping::get_evm_address(account_id)
	}

	/// Claim account mapping between AccountId and a generated EthKYLAddress based off of the
	/// AccountId.
	fn claim_default_evm_address(account_id: &T::AccountId) -> Result<EthKYLAddress, DispatchError> {
		Self::do_claim_default_evm_address(account_id.clone())
	}
}
