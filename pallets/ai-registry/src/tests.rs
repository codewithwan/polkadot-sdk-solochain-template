//! Unit tests for AI Registry pallet

use crate::{
	mock::*,
	pallet::{Error, Event, Models, ModelsByOwner, NextModelId},
	ModelStatus, ModelType,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_model_works() {
	new_test_ext().execute_with(|| {
		// Valid IPFS CIDv0
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		let name = b"Test Model".to_vec();
		let description = b"A test classification model".to_vec();

		// Register model
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid.clone(),
			name.clone(),
			description,
			ModelType::Classification,
			500
		));

		// Check storage
		let model = Models::<Test>::get(0).expect("Model should exist");
		assert_eq!(model.owner, 1);
		assert_eq!(model.model_type, ModelType::Classification);
		assert_eq!(model.price, 500);
		assert_eq!(model.status, ModelStatus::Active);
		assert_eq!(model.total_inferences, 0);
		assert_eq!(model.rating_count, 0);

		// Check double map
		assert!(ModelsByOwner::<Test>::get(1, 0).is_some());

		// Check next ID incremented
		assert_eq!(NextModelId::<Test>::get(), 1);

		// Check event
		System::assert_has_event(
			Event::ModelRegistered {
				model_id: 0,
				owner: 1,
				ipfs_cid: ipfs_cid.try_into().unwrap(),
			}
			.into(),
		);
	});
}

#[test]
fn register_model_with_cidv1_works() {
	new_test_ext().execute_with(|| {
		// Valid IPFS CIDv1
		let ipfs_cid = b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_vec();
		let name = b"Test Model v1".to_vec();
		let description = b"A test model with CIDv1".to_vec();

		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			name,
			description,
			ModelType::Generative,
			1000
		));

		assert_eq!(NextModelId::<Test>::get(), 1);
	});
}

#[test]
fn register_model_fails_with_invalid_cid() {
	new_test_ext().execute_with(|| {
		// Invalid CID (too short)
		let invalid_cid = b"invalid".to_vec();
		let name = b"Test Model".to_vec();
		let description = b"Description".to_vec();

		assert_noop!(
			AIRegistry::register_model(
				RuntimeOrigin::signed(1),
				invalid_cid,
				name,
				description,
				ModelType::Classification,
				500
			),
			Error::<Test>::InvalidIPFSCID
		);
	});
}

#[test]
fn register_model_fails_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		let name = b"Test Model".to_vec();
		let description = b"Description".to_vec();

		// Account 4 has only 500, needs 100 fee + 1000 stake
		assert_noop!(
			AIRegistry::register_model(
				RuntimeOrigin::signed(4),
				ipfs_cid,
				name,
				description,
				ModelType::Classification,
				500
			),
			Error::<Test>::InsufficientStake
		);
	});
}

#[test]
fn update_model_metadata_works() {
	new_test_ext().execute_with(|| {
		// Register a model first
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Update price
		assert_ok!(AIRegistry::update_model_metadata(
			RuntimeOrigin::signed(1),
			0,
			Some(1000),
			None,
			None
		));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.price, 1000);

		// Update description
		let new_desc = b"Updated description".to_vec();
		assert_ok!(AIRegistry::update_model_metadata(
			RuntimeOrigin::signed(1),
			0,
			None,
			Some(new_desc.clone()),
			None
		));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.description.to_vec(), new_desc);

		// Update status
		assert_ok!(AIRegistry::update_model_metadata(
			RuntimeOrigin::signed(1),
			0,
			None,
			None,
			Some(ModelStatus::Paused)
		));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.status, ModelStatus::Paused);
	});
}

#[test]
fn update_model_fails_with_unauthorized_access() {
	new_test_ext().execute_with(|| {
		// Register model with account 1
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Try to update with account 2
		assert_noop!(
			AIRegistry::update_model_metadata(RuntimeOrigin::signed(2), 0, Some(1000), None, None),
			Error::<Test>::UnauthorizedAccess
		);
	});
}

#[test]
fn update_nonexistent_model_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AIRegistry::update_model_metadata(RuntimeOrigin::signed(1), 999, Some(1000), None, None),
			Error::<Test>::ModelNotFound
		);
	});
}

#[test]
fn deactivate_model_works() {
	new_test_ext().execute_with(|| {
		// Register model
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Deactivate
		assert_ok!(AIRegistry::deactivate_model(RuntimeOrigin::signed(1), 0));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.status, ModelStatus::Deactivated);

		// Check event
		System::assert_has_event(Event::ModelDeactivated { model_id: 0, owner: 1 }.into());
	});
}

#[test]
fn deactivate_model_fails_unauthorized() {
	new_test_ext().execute_with(|| {
		// Register model with account 1
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Try to deactivate with account 2
		assert_noop!(
			AIRegistry::deactivate_model(RuntimeOrigin::signed(2), 0),
			Error::<Test>::UnauthorizedAccess
		);
	});
}

#[test]
fn rate_model_works() {
	new_test_ext().execute_with(|| {
		// Register model
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Rate with 5 stars
		assert_ok!(AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 5));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.total_rating, 5);
		assert_eq!(model.rating_count, 1);

		// Rate again with 3 stars
		assert_ok!(AIRegistry::rate_model(RuntimeOrigin::signed(3), 0, 3));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.total_rating, 8);
		assert_eq!(model.rating_count, 2);

		// Check average rating
		let avg = AIRegistry::get_average_rating(0);
		assert_eq!(avg, Some(4)); // 8/2 = 4
	});
}

#[test]
fn rate_model_fails_with_invalid_rating() {
	new_test_ext().execute_with(|| {
		// Register model
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Try to rate with 0 (invalid)
		assert_noop!(
			AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 0),
			Error::<Test>::InvalidRating
		);

		// Try to rate with 6 (invalid)
		assert_noop!(
			AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 6),
			Error::<Test>::InvalidRating
		);
	});
}

#[test]
fn rate_nonexistent_model_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AIRegistry::rate_model(RuntimeOrigin::signed(1), 999, 5),
			Error::<Test>::ModelNotFound
		);
	});
}

#[test]
fn multiple_models_registration_works() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		// Register 3 models
		for i in 0..3 {
			assert_ok!(AIRegistry::register_model(
				RuntimeOrigin::signed(1),
				ipfs_cid.clone(),
				format!("Model {}", i).as_bytes().to_vec(),
				b"Description".to_vec(),
				ModelType::Classification,
				500 * (i as u128 + 1)
			));
		}

		// Check all exist
		assert!(Models::<Test>::get(0).is_some());
		assert!(Models::<Test>::get(1).is_some());
		assert!(Models::<Test>::get(2).is_some());

		// Check next ID
		assert_eq!(NextModelId::<Test>::get(), 3);

		// Check ownership mapping
		assert!(ModelsByOwner::<Test>::get(1, 0).is_some());
		assert!(ModelsByOwner::<Test>::get(1, 1).is_some());
		assert!(ModelsByOwner::<Test>::get(1, 2).is_some());
	});
}

#[test]
fn increment_inference_count_works() {
	new_test_ext().execute_with(|| {
		// Register model
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// Initial count should be 0
		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.total_inferences, 0);

		// Increment count (simulating inference completion)
		assert_ok!(AIRegistry::increment_inference_count(0));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.total_inferences, 1);

		// Increment again
		assert_ok!(AIRegistry::increment_inference_count(0));

		let model = Models::<Test>::get(0).unwrap();
		assert_eq!(model.total_inferences, 2);
	});
}

#[test]
fn get_average_rating_works() {
	new_test_ext().execute_with(|| {
		// No model registered yet
		assert_eq!(AIRegistry::get_average_rating(0), None);

		// Register model
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();
		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			b"Model".to_vec(),
			b"Description".to_vec(),
			ModelType::Classification,
			500
		));

		// No ratings yet
		assert_eq!(AIRegistry::get_average_rating(0), None);

		// Add some ratings
		assert_ok!(AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 5));
		assert_eq!(AIRegistry::get_average_rating(0), Some(5));

		assert_ok!(AIRegistry::rate_model(RuntimeOrigin::signed(3), 0, 3));
		assert_eq!(AIRegistry::get_average_rating(0), Some(4)); // (5+3)/2 = 4
	});
}
