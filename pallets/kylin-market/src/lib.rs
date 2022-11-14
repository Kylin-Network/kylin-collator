#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
	BoundedVec,
};
use frame_system::{ensure_signed, RawOrigin};

use sp_std::prelude::*;
use kylin_primitives::nft::{NftInfo, AccountIdOrCollectionNftTuple};
use kylin_primitives::types::{CollectionId, NftId};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;
use crate::types::*;
pub use pallet::*;

pub type InstanceInfoOf<T> = NftInfo<
	<T as frame_system::Config>::AccountId,
	BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>,
>;

pub type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub type ListInfoOf<T> = ListInfo<
	<T as frame_system::Config>::AccountId,
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
>;

pub type OfferOf<T> = Offer<
	<T as frame_system::Config>::AccountId,
	BalanceOf<T>,
	<T as frame_system::Config>::BlockNumber,
>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use kylin_primitives::types::{CollectionId, NftId};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn listed_nfts)]
	pub type ListedNfts<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, CollectionId,
		Blake2_128Concat, NftId,
		ListInfoOf<T>, OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat, (CollectionId, NftId),
		Blake2_128Concat, T::AccountId,
		OfferOf<T>, OptionQuery,
	>;

	#[pallet::config]
	pub trait Config: frame_system::Config + kylin_feed::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type ProtocolOrigin: EnsureOrigin<Self::Origin>;

		type Currency: ReservableCurrency<Self::AccountId>;

		#[pallet::constant]
		type MinimumOfferAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenPriceUpdated {
			owner: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
			price: Option<BalanceOf<T>>,
		},
		TokenSold {
			owner: T::AccountId,
			buyer: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
			price: BalanceOf<T>,
		},
		TokenListed {
			owner: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
			price: BalanceOf<T>,
		},
		TokenUnlisted { owner: T::AccountId, collection_id: CollectionId, nft_id: NftId },
		OfferPlaced {
			offerer: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
			price: BalanceOf<T>,
		},
		OfferWithdrawn { sender: T::AccountId, collection_id: CollectionId, nft_id: NftId },
		OfferAccepted {
			owner: T::AccountId,
			buyer: T::AccountId,
			collection_id: CollectionId,
			nft_id: NftId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		NoPermission,
		TokenNotForSale,
		CannotWithdrawOffer,
		CannotUnlistToken,
		CannotOfferOnOwnToken,
		CannotBuyOwnToken,
		UnknownOffer,
		CannotListNftOwnedByNft,
		TokenDoesNotExist,
		OfferTooLow,
		AlreadyOffered,
		OfferHasExpired,
		ListingHasExpired,
		PriceDiffersFromExpected,
		NonTransferable,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T>
		where T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
	{
		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn buy(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			amount: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_buy(sender, collection_id, nft_id, amount, false)
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn list(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			amount: BalanceOf<T>,
			expires: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id)
				.ok_or(Error::<T>::TokenDoesNotExist)?;

			ensure!(
				!Self::is_nft_owned_by_nft(collection_id, nft_id),
				Error::<T>::CannotListNftOwnedByNft
			);
			ensure!(sender == owner, Error::<T>::NoPermission);

			let nft = kylin_feed::Pallet::<T>::nfts(collection_id, nft_id)
				.ok_or(Error::<T>::TokenDoesNotExist)?;

			kylin_feed::Pallet::<T>::check_is_transferable(&nft)?;
			kylin_feed::Pallet::<T>::set_lock((collection_id, nft_id), true);
			if Self::is_nft_listed(collection_id, nft_id) {
				ListedNfts::<T>::remove(collection_id, nft_id);
			}

			// Add new ListInfo with listed_by, amount, Option<BlockNumber>
			ListedNfts::<T>::insert(
				collection_id,
				nft_id,
				ListInfo { listed_by: sender, amount, expires },
			);

			Self::deposit_event(Event::TokenListed { owner, collection_id, nft_id, price: amount });

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn unlist(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Check if NFT is still in ListedNfts storage
			ensure!(Self::is_nft_listed(collection_id, nft_id), Error::<T>::CannotUnlistToken);
			let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id)
				.ok_or(Error::<T>::TokenDoesNotExist)?;
			// Ensure owner of NFT is performing call to unlist
			ensure!(sender == owner, Error::<T>::NoPermission);
			// Set the NFT lock to false to allow interactions with the NFT
			kylin_feed::Pallet::<T>::set_lock((collection_id, nft_id), false);
			// Remove from storage
			ListedNfts::<T>::remove(collection_id, nft_id);
			// Emit TokenUnlisted Event
			Self::deposit_event(Event::TokenUnlisted { owner, collection_id, nft_id });

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn make_offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			amount: BalanceOf<T>,
			expires: Option<T::BlockNumber>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Ensure amount is above the minimum threshold
			ensure!(amount >= T::MinimumOfferAmount::get(), Error::<T>::OfferTooLow);
			// Ensure NFT exists & sender is not owner
			let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id)
				.ok_or(Error::<T>::TokenDoesNotExist)?;

			ensure!(sender != owner, Error::<T>::CannotOfferOnOwnToken);
			// If offer has already been made, must withdraw_offer first before making a new offer
			ensure!(
				!Self::has_active_offer(collection_id, nft_id, sender.clone()),
				Error::<T>::AlreadyOffered
			);

			// Reserve currency from offerer account
			<T as pallet::Config>::Currency::reserve(&sender, amount)?;

			let token_id = (collection_id, nft_id);
			Offers::<T>::insert(
				token_id,
				sender.clone(),
				Offer { maker: sender.clone(), amount, expires },
			);

			Self::deposit_event(Event::OfferPlaced {
				offerer: sender,
				collection_id,
				nft_id,
				price: amount,
			});

			Ok(())
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn withdraw_offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let token_id = (collection_id, nft_id);
			// Ensure that offer exists from sender that is withdrawing their offer
			Offers::<T>::try_mutate_exists(
				token_id,
				sender.clone(),
				|maybe_offer| -> DispatchResult {
					let offer = maybe_offer.take().ok_or(Error::<T>::UnknownOffer)?;
					// Ensure NFT exists & sender is not owner
					let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id)
						.ok_or(Error::<T>::TokenDoesNotExist)?;
					// Cannot withdraw offer on own token
					ensure!(
						sender == owner || sender == offer.maker,
						Error::<T>::CannotWithdrawOffer
					);

					// Unreserve currency from offerer account
					<T as pallet::Config>::Currency::unreserve(&offer.maker, offer.amount);
					// Emit OfferWithdrawn Event
					Self::deposit_event(Event::OfferWithdrawn { sender, collection_id, nft_id });

					Ok(())
				},
			)
		}

		#[pallet::weight(T::DbWeight::get().reads_writes(1,1).ref_time().saturating_add(10_000))]
		pub fn accept_offer(
			origin: OriginFor<T>,
			collection_id: CollectionId,
			nft_id: NftId,
			offerer: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id)
				.ok_or(Error::<T>::TokenDoesNotExist)?;
			ensure!(sender == owner, Error::<T>::NoPermission);

			let token_id = (collection_id, nft_id);
			Offers::<T>::try_mutate_exists(
				token_id,
				offerer.clone(),
				|maybe_offer| -> DispatchResult {
					let offer = maybe_offer.take().ok_or(Error::<T>::UnknownOffer)?;

					if let Some(expires) = offer.expires {
						if expires <= <frame_system::Pallet<T>>::block_number() {
							return Err(Error::<T>::OfferHasExpired.into())
						}
					}

					<T as pallet::Config>::Currency::unreserve(&offer.maker, offer.amount);
					Self::do_buy(offer.maker, collection_id, nft_id, None, true)?;
					// Emit OfferAccepted event
					Self::deposit_event(Event::OfferAccepted {
						owner,
						buyer: offerer,
						collection_id,
						nft_id,
					});

					Ok(())
				},
			)
		}
	}
}

impl<T: Config> Pallet<T>
	where T: pallet_uniques::Config<CollectionId = CollectionId, ItemId = NftId>,
{
	fn do_buy(
		buyer: T::AccountId,
		collection_id: CollectionId,
		nft_id: NftId,
		amount: Option<BalanceOf<T>>,
		is_offer: bool,
	) -> DispatchResult {
		// Ensure buyer is not the root owner
		let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id)
			.ok_or(Error::<T>::TokenDoesNotExist)?;
		ensure!(buyer != owner, Error::<T>::CannotBuyOwnToken);

		let owner_origin = T::Origin::from(RawOrigin::Signed(owner.clone()));
		let token_id = (collection_id, nft_id);

		let list_price = if is_offer {
			Offers::<T>::get(token_id, buyer.clone())
				.map(|o| o.amount)
				.ok_or(Error::<T>::UnknownOffer)?
		} else {
			let list_info =
				ListedNfts::<T>::take(collection_id, nft_id).ok_or(Error::<T>::TokenNotForSale)?;
			// Ensure that the current owner is the one that listed the NFT
			ensure!(list_info.listed_by == owner, Error::<T>::TokenNotForSale);
			// Ensure the listing has not expired if Some(expires)
			// if None then there is no expiration
			if let Some(expires) = list_info.expires {
				ensure!(
					expires > <frame_system::Pallet<T>>::block_number(),
					Error::<T>::ListingHasExpired
				);
			}
			list_info.amount
		};

		// Check if list_price is equal to amount to prevent front running a buy
		if let Some(amount) = amount {
			ensure!(list_price == amount, Error::<T>::PriceDiffersFromExpected);
		}

		// Set NFT Lock status to false to facilitate the purchase
		kylin_feed::Pallet::<T>::set_lock((collection_id, nft_id), false);

		// Transfer currency then transfer the NFT
		<T as pallet::Config>::Currency::transfer(
			&buyer,
			&owner,
			list_price,
			ExistenceRequirement::KeepAlive,
		)?;

		let new_owner = AccountIdOrCollectionNftTuple::AccountId(buyer.clone());
		kylin_feed::Pallet::<T>::send(owner_origin, collection_id, nft_id, new_owner)?;

		Self::deposit_event(Event::TokenSold {
			owner,
			buyer,
			collection_id,
			nft_id,
			price: list_price,
		});

		Ok(())
	}

	fn is_nft_listed(collection_id: CollectionId, nft_id: NftId) -> bool {
		ListedNfts::<T>::contains_key(collection_id, nft_id)
	}

	fn has_active_offer(collection_id: CollectionId, nft_id: NftId, sender: T::AccountId) -> bool {
		Offers::<T>::contains_key((collection_id, nft_id), sender)
	}

	fn is_nft_owned_by_nft(collection_id: CollectionId, nft_id: NftId) -> bool {
		let owner = pallet_uniques::Pallet::<T>::owner(collection_id, nft_id);
		if let Some(current_owner) = owner {
			let current_owner_cid_nid =
				kylin_feed::Pallet::<T>::decode_nft_account_id::<T::AccountId>(current_owner);
			if let Some(_current_owner_cid_nid) = current_owner_cid_nid {
				return true
			}
		}
		false
	}
}
