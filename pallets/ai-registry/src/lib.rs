//! # AI Registry Pallet - Working Version
//!
//! A simplified but fully functional version of the AI registry pallet
//! that resolves Substrate trait bound compatibility issues by inlining types.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod tests_new;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode, MaxEncodedLen};
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_runtime::RuntimeDebug;

	/// Unique identifier for models
	pub type ModelId = u64;

	/// Type of AI model
	#[derive(
		Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen,
	)]
	pub enum ModelType {
		Classification,
		Regression,
		Generative,
	}

	impl Default for ModelType {
		fn default() -> Self {
			ModelType::Classification
		}
	}

	/// Status of a model
	#[derive(
		Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen,
	)]
	pub enum ModelStatus {
		Active,
		Paused,
		Deactivated,
	}

	impl Default for ModelStatus {
		fn default() -> Self {
			ModelStatus::Active
		}
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type MinimumModelStake: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type RegistrationFee: Get<BalanceOf<Self>>;
	}

	/// Simplified model metadata (non-generic to avoid MaxEncodedLen issues)
	#[pallet::storage]
	pub type ModelOwner<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, T::AccountId>;

	#[pallet::storage]
	pub type ModelCID<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, BoundedVec<u8, ConstU32<128>>>;

	#[pallet::storage]
	pub type ModelPrice<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, u128>;

	#[pallet::storage]
	pub type ModelTypeStorage<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, ModelType>;

	#[pallet::storage]
	pub type ModelStatusStorage<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, ModelStatus>;

	#[pallet::storage]
	pub type ModelRatingTotal<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, u64, ValueQuery>;

	#[pallet::storage]
	pub type ModelRatingCount<T: Config> = StorageMap<_, Blake2_128Concat, ModelId, u32, ValueQuery>;

	#[pallet::storage]
	pub type ModelsByOwner<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, ModelId, (), OptionQuery>;

	#[pallet::storage]
	pub type NextModelId<T: Config> = StorageValue<_, ModelId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ModelRegistered { model_id: ModelId, owner: T::AccountId },
		ModelUpdated { model_id: ModelId },
		ModelDeactivated { model_id: ModelId },
		ModelRated { model_id: ModelId, rating: u8 },
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidIPFSCID,
		InsufficientStake,
		ModelNotFound,
		UnauthorizedAccess,
		InvalidRating,
		InsufficientBalance,
		ArithmeticOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new AI model (using u8 for model_type: 0=Classification, 1=Regression, 2=Generative)
		#[pallet::call_index(0)]
		#[pallet::weight(50_000_000)]
		pub fn register_model(
			origin: OriginFor<T>,
			ipfs_cid: Vec<u8>,
			model_type_u8: u8,
			price: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate model type
			ensure!(model_type_u8 <= 2, Error::<T>::InvalidIPFSCID); // Reuse error for simplicity
			let model_type = match model_type_u8 {
				0 => ModelType::Classification,
				1 => ModelType::Regression,
				_ => ModelType::Generative,
			};

			// Validate CID
			let bounded_cid: BoundedVec<u8, ConstU32<128>> =
				ipfs_cid.try_into().map_err(|_| Error::<T>::InvalidIPFSCID)?;
			ensure!(Self::validate_ipfs_cid(&bounded_cid), Error::<T>::InvalidIPFSCID);

			// Check stake
			let free_balance = T::Currency::free_balance(&who);
			ensure!(free_balance >= T::MinimumModelStake::get(), Error::<T>::InsufficientStake);

			// Charge fee
			let fee = T::RegistrationFee::get();
			ensure!(free_balance >= fee, Error::<T>::InsufficientBalance);

			let _imbalance = T::Currency::withdraw(
				&who,
				fee,
				frame_support::traits::WithdrawReasons::FEE,
				ExistenceRequirement::KeepAlive,
			)?;

			// Get next ID
			let model_id = NextModelId::<T>::get();
			let next_id = model_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;

			// Store model data
			ModelOwner::<T>::insert(model_id, &who);
			ModelCID::<T>::insert(model_id, bounded_cid);
			ModelPrice::<T>::insert(model_id, price);
			ModelTypeStorage::<T>::insert(model_id, model_type);
			ModelStatusStorage::<T>::insert(model_id, ModelStatus::Active);
			ModelsByOwner::<T>::insert(&who, model_id, ());
			NextModelId::<T>::put(next_id);

			Self::deposit_event(Event::ModelRegistered { model_id, owner: who });

			Ok(())
		}

		/// Update model price
		#[pallet::call_index(1)]
		#[pallet::weight(30_000_000)]
		pub fn update_model_price(
			origin: OriginFor<T>,
			model_id: ModelId,
			new_price: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let owner = ModelOwner::<T>::get(model_id).ok_or(Error::<T>::ModelNotFound)?;
			ensure!(owner == who, Error::<T>::UnauthorizedAccess);

			ModelPrice::<T>::insert(model_id, new_price);

			Self::deposit_event(Event::ModelUpdated { model_id });

			Ok(())
		}

		/// Deactivate model
		#[pallet::call_index(2)]
		#[pallet::weight(25_000_000)]
		pub fn deactivate_model(origin: OriginFor<T>, model_id: ModelId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let owner = ModelOwner::<T>::get(model_id).ok_or(Error::<T>::ModelNotFound)?;
			ensure!(owner == who, Error::<T>::UnauthorizedAccess);

			ModelStatusStorage::<T>::insert(model_id, ModelStatus::Deactivated);

			Self::deposit_event(Event::ModelDeactivated { model_id });

			Ok(())
		}

		/// Rate a model (1-5 stars)
		#[pallet::call_index(3)]
		#[pallet::weight(28_000_000)]
		pub fn rate_model(origin: OriginFor<T>, model_id: ModelId, rating: u8) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			ensure!(rating >= 1 && rating <= 5, Error::<T>::InvalidRating);
			ensure!(ModelOwner::<T>::contains_key(model_id), Error::<T>::ModelNotFound);

			ModelRatingTotal::<T>::mutate(model_id, |total| {
				*total = total.saturating_add(rating as u64);
			});

			ModelRatingCount::<T>::mutate(model_id, |count| {
				*count = count.saturating_add(1);
			});

			Self::deposit_event(Event::ModelRated { model_id, rating });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Validate IPFS CID format
		fn validate_ipfs_cid(cid: &BoundedVec<u8, ConstU32<128>>) -> bool {
			if cid.len() < 46 {
				return false;
			}
			// CIDv0: starts with "Qm" and is 46 characters
			if cid.len() == 46 && cid.starts_with(b"Qm") {
				return true;
			}
			// CIDv1: starts with "b"
			if cid.starts_with(b"b") || cid.starts_with(b"B") {
				return true;
			}
			false
		}

		/// Get average rating
		pub fn get_average_rating(model_id: ModelId) -> Option<u8> {
			let count = ModelRatingCount::<T>::get(model_id);
			if count > 0 {
				let total = ModelRatingTotal::<T>::get(model_id);
				Some((total / count as u64) as u8)
			} else {
				None
			}
		}
	}
}
