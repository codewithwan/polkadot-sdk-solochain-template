//! Type definitions for AI Registry pallet

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Unique identifier for models
pub type ModelId = u64;

/// Type of AI model
#[derive(
	Clone, Copy, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub enum ModelType {
	/// Classification model (e.g., image classification, sentiment analysis)
	Classification,
	/// Regression model (e.g., price prediction, value estimation)
	Regression,
	/// Generative model (e.g., text generation, image generation)
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
	/// Model is active and available for inference
	Active,
	/// Model is temporarily paused by owner
	Paused,
	/// Model is permanently deactivated
	Deactivated,
	/// Model is deprecated (superseded by newer version)
	Deprecated,
}

impl Default for ModelStatus {
	fn default() -> Self {
		ModelStatus::Active
	}
}

/// Comprehensive metadata for an AI model
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct ModelMetadata<T: frame_system::Config> {
	/// Owner/creator of the model
	pub owner: T::AccountId,
	/// IPFS Content Identifier for model data
	pub ipfs_cid: BoundedVec<u8, ConstU32<128>>,
	/// Human-readable model name
	pub name: BoundedVec<u8, ConstU32<256>>,
	/// Model description
	pub description: BoundedVec<u8, ConstU32<1024>>,
	/// Type of AI model
	pub model_type: ModelType,
	/// Price per inference (will be converted from BalanceOf<T>)
	pub price: u128,
	/// Block number when model was created
	pub created_at: u64,
	/// Total number of inferences performed
	pub total_inferences: u64,
	/// Sum of all ratings (for average calculation)
	pub total_rating: u64,
	/// Number of ratings received
	pub rating_count: u32,
	/// Current status of the model
	pub status: ModelStatus,
}
