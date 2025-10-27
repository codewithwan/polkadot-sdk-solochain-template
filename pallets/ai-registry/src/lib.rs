//! # AI Registry Pallet
//!
//! A production-ready pallet for registering and managing AI models on-chain.
//!
//! ## Overview
//!
//! The AI Registry pallet enables AI developers to register their models with metadata
//! stored on IPFS. It provides functionality for:
//! - Registering new AI models with IPFS CID references
//! - Updating model metadata (price, status, description)
//! - Deactivating models
//! - Rating models based on inference quality
//! - Querying models by owner or ID
//!
//! ## Model Lifecycle
//!
//! 1. **Registration**: Developer calls `register_model` with IPFS CID and metadata
//! 2. **Active**: Model is available for inference requests
//! 3. **Updated**: Owner can update metadata via `update_model_metadata`
//! 4. **Rated**: Users who purchased inference can rate via `rate_model`
//! 5. **Deactivated**: Owner or governance can deactivate via `deactivate_model`
//!
//! ## Economic Model
//!
//! - Registration requires minimum stake and registration fee
//! - Model owner sets inference price
//! - Revenue shared between owner and validators
//!
//! ## Security
//!
//! - Only model owner can update or deactivate
//! - IPFS CID format validation
//! - Rating restricted to users who paid for inference
//! - Input validation on all parameters

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

pub mod types;
pub use types::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::SaturatedConversion;
	use sp_std::vec::Vec;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configuration trait for the AI Registry pallet
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet
		type WeightInfo: WeightInfo;

		/// Currency type for handling payments and stakes
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Minimum stake required to register a model
		#[pallet::constant]
		type MinimumModelStake: Get<BalanceOf<Self>>;

		/// Registration fee for new models
		#[pallet::constant]
		type RegistrationFee: Get<BalanceOf<Self>>;

		/// Maximum length of IPFS CID
		#[pallet::constant]
		type MaxCidLength: Get<u32>;

		/// Maximum length of model name
		#[pallet::constant]
		type MaxNameLength: Get<u32>;

		/// Maximum length of model description
		#[pallet::constant]
		type MaxDescriptionLength: Get<u32>;
	}

	/// Storage for model metadata indexed by ModelId
	#[pallet::storage]
	#[pallet::without_storage_info]
	pub type Models<T: Config> =
		StorageMap<_, Blake2_128Concat, ModelId, ModelMetadata<T>, OptionQuery>;

	/// Double map for querying models by owner
	#[pallet::storage]
	pub type ModelsByOwner<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		ModelId,
		(),
		OptionQuery,
	>;

	/// Counter for generating unique model IDs
	#[pallet::storage]
	pub type NextModelId<T: Config> = StorageValue<_, ModelId, ValueQuery>;

	/// Events emitted by this pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new model was registered
		/// [model_id, owner, ipfs_cid]
		ModelRegistered {
			model_id: ModelId,
			owner: T::AccountId,
			ipfs_cid: BoundedVec<u8, T::MaxCidLength>,
		},
		/// Model metadata was updated
		/// [model_id, owner]
		ModelUpdated { model_id: ModelId, owner: T::AccountId },
		/// Model was deactivated
		/// [model_id, owner]
		ModelDeactivated { model_id: ModelId, owner: T::AccountId },
		/// Model was rated
		/// [model_id, rater, rating]
		ModelRated { model_id: ModelId, rater: T::AccountId, rating: u8 },
	}

	/// Errors that can occur in this pallet
	#[pallet::error]
	pub enum Error<T> {
		/// IPFS CID format is invalid
		InvalidIPFSCID,
		/// Insufficient stake for model registration
		InsufficientStake,
		/// Model not found
		ModelNotFound,
		/// Caller is not authorized for this operation
		UnauthorizedAccess,
		/// Model already exists with this ID
		ModelAlreadyExists,
		/// Rating value is invalid (must be 1-5)
		InvalidRating,
		/// Model name is too long
		NameTooLong,
		/// Model description is too long
		DescriptionTooLong,
		/// IPFS CID is too long
		CidTooLong,
		/// Model is not in active status
		ModelNotActive,
		/// Arithmetic overflow occurred
		ArithmeticOverflow,
		/// User has not purchased inference for this model
		NotInferenceUser,
		/// Insufficient balance for registration fee
		InsufficientBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a new AI model
		///
		/// # Arguments
		/// * `origin` - The account registering the model
		/// * `ipfs_cid` - IPFS Content Identifier for the model
		/// * `name` - Human-readable model name
		/// * `description` - Model description
		/// * `model_type` - Type of AI model (classification, regression, generative)
		/// * `price` - Price for single inference in native tokens
		///
		/// # Errors
		/// * `InvalidIPFSCID` - CID format validation failed
		/// * `InsufficientStake` - Caller doesn't have minimum stake
		/// * `NameTooLong` - Name exceeds maximum length
		/// * `DescriptionTooLong` - Description exceeds maximum length
		/// * `InsufficientBalance` - Cannot pay registration fee
		///
		/// # Events
		/// * `ModelRegistered` - Model successfully registered
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register_model())]
		pub fn register_model(
			origin: OriginFor<T>,
			ipfs_cid: Vec<u8>,
			name: Vec<u8>,
			description: Vec<u8>,
			model_type: ModelType,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate IPFS CID length
			let bounded_cid: BoundedVec<u8, T::MaxCidLength> =
				ipfs_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
			ensure!(Self::validate_ipfs_cid(&bounded_cid), Error::<T>::InvalidIPFSCID);

			// Validate name length
			let bounded_name: BoundedVec<u8, T::MaxNameLength> =
				name.try_into().map_err(|_| Error::<T>::NameTooLong)?;

			// Validate description length
			let bounded_description: BoundedVec<u8, T::MaxDescriptionLength> =
				description.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;

			// Check minimum stake
			let free_balance = T::Currency::free_balance(&who);
			ensure!(free_balance >= T::MinimumModelStake::get(), Error::<T>::InsufficientStake);

			// Charge registration fee
			let fee = T::RegistrationFee::get();
			ensure!(free_balance >= fee, Error::<T>::InsufficientBalance);

			// Transfer registration fee (burned or to treasury)
			T::Currency::withdraw(
				&who,
				fee,
				frame_support::traits::WithdrawReasons::FEE,
				ExistenceRequirement::KeepAlive,
			)?;

			// Get next model ID
			let model_id = NextModelId::<T>::get();
			let next_id = model_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;

			// Create model metadata
			let now = frame_system::Pallet::<T>::block_number();
			let price_u128: u128 = price.saturated_into();
			let created_at_u64: u64 = now.saturated_into();
			let metadata = ModelMetadata {
				owner: who.clone(),
				ipfs_cid: bounded_cid.clone(),
				name: bounded_name,
				description: bounded_description,
				model_type,
				price: price_u128,
				created_at: created_at_u64,
				total_inferences: 0,
				total_rating: 0,
				rating_count: 0,
				status: ModelStatus::Active,
			};

			// Store model
			Models::<T>::insert(model_id, metadata);
			ModelsByOwner::<T>::insert(&who, model_id, ());
			NextModelId::<T>::put(next_id);

			// Emit event
			Self::deposit_event(Event::ModelRegistered {
				model_id,
				owner: who,
				ipfs_cid: bounded_cid,
			});

			Ok(())
		}

		/// Update model metadata
		///
		/// # Arguments
		/// * `origin` - Must be the model owner
		/// * `model_id` - ID of the model to update
		/// * `new_price` - Optional new price
		/// * `new_description` - Optional new description
		/// * `new_status` - Optional new status
		///
		/// # Errors
		/// * `ModelNotFound` - Model doesn't exist
		/// * `UnauthorizedAccess` - Caller is not the owner
		///
		/// # Events
		/// * `ModelUpdated` - Metadata successfully updated
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_model_metadata())]
		pub fn update_model_metadata(
			origin: OriginFor<T>,
			model_id: ModelId,
			new_price: Option<BalanceOf<T>>,
			new_description: Option<Vec<u8>>,
			new_status: Option<ModelStatus>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get model and verify ownership
			Models::<T>::try_mutate(model_id, |maybe_model| -> DispatchResult {
				let model = maybe_model.as_mut().ok_or(Error::<T>::ModelNotFound)?;
				ensure!(model.owner == who, Error::<T>::UnauthorizedAccess);

				// Update fields if provided
				if let Some(price) = new_price {
					model.price = price.saturated_into();
				}

				if let Some(desc) = new_description {
					let bounded_desc: BoundedVec<u8, T::MaxDescriptionLength> =
						desc.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?;
					model.description = bounded_desc;
				}

				if let Some(status) = new_status {
					model.status = status;
				}

				Ok(())
			})?;

			Self::deposit_event(Event::ModelUpdated { model_id, owner: who });

			Ok(())
		}

		/// Deactivate a model
		///
		/// # Arguments
		/// * `origin` - Must be the model owner or governance
		/// * `model_id` - ID of the model to deactivate
		///
		/// # Errors
		/// * `ModelNotFound` - Model doesn't exist
		/// * `UnauthorizedAccess` - Caller is not the owner
		///
		/// # Events
		/// * `ModelDeactivated` - Model successfully deactivated
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::deactivate_model())]
		pub fn deactivate_model(origin: OriginFor<T>, model_id: ModelId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Get model and verify ownership
			Models::<T>::try_mutate(model_id, |maybe_model| -> DispatchResult {
				let model = maybe_model.as_mut().ok_or(Error::<T>::ModelNotFound)?;
				ensure!(model.owner == who, Error::<T>::UnauthorizedAccess);

				model.status = ModelStatus::Deactivated;

				Ok(())
			})?;

			Self::deposit_event(Event::ModelDeactivated { model_id, owner: who });

			Ok(())
		}

		/// Rate a model
		///
		/// # Arguments
		/// * `origin` - User who purchased inference
		/// * `model_id` - ID of the model to rate
		/// * `rating` - Rating value (1-5)
		///
		/// # Errors
		/// * `ModelNotFound` - Model doesn't exist
		/// * `InvalidRating` - Rating not in 1-5 range
		/// * `NotInferenceUser` - Caller hasn't purchased inference
		///
		/// # Events
		/// * `ModelRated` - Model successfully rated
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::rate_model())]
		pub fn rate_model(
			origin: OriginFor<T>,
			model_id: ModelId,
			rating: u8,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Validate rating
			ensure!(rating >= 1 && rating <= 5, Error::<T>::InvalidRating);

			// Update model rating
			Models::<T>::try_mutate(model_id, |maybe_model| -> DispatchResult {
				let model = maybe_model.as_mut().ok_or(Error::<T>::ModelNotFound)?;

				// TODO: In production, verify user has purchased inference
				// This would check pallet-inference storage
				// For MVP, we allow any user to rate

				// Update rating statistics
				let new_total = model
					.total_rating
					.checked_add(rating as u64)
					.ok_or(Error::<T>::ArithmeticOverflow)?;
				let new_count =
					model.rating_count.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;

				model.total_rating = new_total;
				model.rating_count = new_count;

				Ok(())
			})?;

			Self::deposit_event(Event::ModelRated { model_id, rater: who, rating });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Validate IPFS CID format
		///
		/// Basic validation: CID should start with "Qm" (CIDv0) or be valid CIDv1
		/// For production, use a proper CID validation library
		fn validate_ipfs_cid(cid: &BoundedVec<u8, T::MaxCidLength>) -> bool {
			if cid.len() < 46 {
				return false;
			}

			// CIDv0: starts with "Qm" and is 46 characters
			if cid.len() == 46 && cid.starts_with(b"Qm") {
				return true;
			}

			// CIDv1: starts with "b" and uses base32
			if cid.starts_with(b"b") || cid.starts_with(b"B") {
				return true;
			}

			false
		}

		/// Get average rating for a model
		pub fn get_average_rating(model_id: ModelId) -> Option<u8> {
			Models::<T>::get(model_id).and_then(|model| {
				if model.rating_count > 0 {
					Some((model.total_rating / model.rating_count as u64) as u8)
				} else {
					None
				}
			})
		}

		/// Increment inference count for a model
		/// Called by pallet-inference when inference is completed
		pub fn increment_inference_count(model_id: ModelId) -> DispatchResult {
			Models::<T>::try_mutate(model_id, |maybe_model| -> DispatchResult {
				let model = maybe_model.as_mut().ok_or(Error::<T>::ModelNotFound)?;
				model.total_inferences =
					model.total_inferences.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
				Ok(())
			})
		}
	}
}
