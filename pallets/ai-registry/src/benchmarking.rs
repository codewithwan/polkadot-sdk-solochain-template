//! Benchmarking setup for pallet-ai-registry

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::v2::*;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_std::vec;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn register_model() {
		let caller: T::AccountId = whitelisted_caller();
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		let name = b"Benchmark Model".to_vec();
		let description = b"A model used for benchmarking".to_vec();
		
		// Fund the caller
		let min_balance = T::MinimumModelStake::get() + T::RegistrationFee::get();
		T::Currency::make_free_balance_be(&caller, min_balance);

		#[extrinsic_call]
		register_model(
			RawOrigin::Signed(caller.clone()),
			ipfs_cid,
			name,
			description,
			ModelType::Classification,
			1000u128,
		);

		assert!(Models::<T>::contains_key(0));
	}

	#[benchmark]
	fn update_model_metadata() {
		let caller: T::AccountId = whitelisted_caller();
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		
		// Setup: register a model first
		let min_balance = T::MinimumModelStake::get() + T::RegistrationFee::get();
		T::Currency::make_free_balance_be(&caller, min_balance);
		
		let _ = Pallet::<T>::register_model(
			RawOrigin::Signed(caller.clone()).into(),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			1000u128,
		);

		#[extrinsic_call]
		update_model_metadata(
			RawOrigin::Signed(caller),
			0,
			Some(2000u128),
			None,
			None,
		);

		let model = Models::<T>::get(0).unwrap();
		assert_eq!(model.price, 2000u128);
	}

	#[benchmark]
	fn deactivate_model() {
		let caller: T::AccountId = whitelisted_caller();
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		
		// Setup: register a model first
		let min_balance = T::MinimumModelStake::get() + T::RegistrationFee::get();
		T::Currency::make_free_balance_be(&caller, min_balance);
		
		let _ = Pallet::<T>::register_model(
			RawOrigin::Signed(caller.clone()).into(),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			1000u128,
		);

		#[extrinsic_call]
		deactivate_model(RawOrigin::Signed(caller), 0);

		let model = Models::<T>::get(0).unwrap();
		assert_eq!(model.status, ModelStatus::Deactivated);
	}

	#[benchmark]
	fn rate_model() {
		let owner: T::AccountId = whitelisted_caller();
		let rater: T::AccountId = account("rater", 0, 0);
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		
		// Setup: register a model first
		let min_balance = T::MinimumModelStake::get() + T::RegistrationFee::get();
		T::Currency::make_free_balance_be(&owner, min_balance);
		
		let _ = Pallet::<T>::register_model(
			RawOrigin::Signed(owner).into(),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			1000u128,
		);

		#[extrinsic_call]
		rate_model(RawOrigin::Signed(rater), 0, 5);

		let model = Models::<T>::get(0).unwrap();
		assert_eq!(model.rating_count, 1);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}
