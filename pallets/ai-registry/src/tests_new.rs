//! Basic integration tests for AI Registry pallet

use crate::{pallet as pallet_ai_registry, ModelStatus, ModelType};
use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types, traits::ConstU128};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system::Pallet<Test>;

	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances::Pallet<Test>;

	#[runtime::pallet_index(2)]
	pub type AIRegistry = pallet_ai_registry::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u128>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type Balance = u128;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
}

parameter_types! {
	pub const MinimumModelStake: u128 = 1000;
	pub const RegistrationFee: u128 = 100;
}

impl pallet_ai_registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinimumModelStake = MinimumModelStake;
	type RegistrationFee = RegistrationFee;
}

fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10000), (2, 10000), (3, 10000), (4, 500)],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

#[test]
fn register_model_works() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		assert_ok!(AIRegistry::register_model(
			RuntimeOrigin::signed(1),
			ipfs_cid,
			0, // Classification
			500
		));

		assert_eq!(pallet_ai_registry::NextModelId::<Test>::get(), 1);
		assert!(pallet_ai_registry::ModelOwner::<Test>::get(0).is_some());
		assert_eq!(pallet_ai_registry::ModelPrice::<Test>::get(0), Some(500));
	});
}

#[test]
fn register_with_invalid_cid_fails() {
	new_test_ext().execute_with(|| {
		let invalid_cid = b"invalid".to_vec();

		assert_noop!(
			AIRegistry::register_model(RuntimeOrigin::signed(1), invalid_cid, 0, 500),
			pallet_ai_registry::Error::<Test>::InvalidIPFSCID
		);
	});
}

#[test]
fn update_price_works() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		assert_ok!(AIRegistry::register_model(RuntimeOrigin::signed(1), ipfs_cid, 0, 500));

		assert_ok!(AIRegistry::update_model_price(RuntimeOrigin::signed(1), 0, 1000));

		assert_eq!(pallet_ai_registry::ModelPrice::<Test>::get(0), Some(1000));
	});
}

#[test]
fn update_price_unauthorized_fails() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		assert_ok!(AIRegistry::register_model(RuntimeOrigin::signed(1), ipfs_cid, 0, 500));

		assert_noop!(
			AIRegistry::update_model_price(RuntimeOrigin::signed(2), 0, 1000),
			pallet_ai_registry::Error::<Test>::UnauthorizedAccess
		);
	});
}

#[test]
fn deactivate_model_works() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		assert_ok!(AIRegistry::register_model(RuntimeOrigin::signed(1), ipfs_cid, 0, 500));

		assert_ok!(AIRegistry::deactivate_model(RuntimeOrigin::signed(1), 0));

		assert_eq!(
			pallet_ai_registry::ModelStatusStorage::<Test>::get(0),
			Some(ModelStatus::Deactivated)
		);
	});
}

#[test]
fn rate_model_works() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		assert_ok!(AIRegistry::register_model(RuntimeOrigin::signed(1), ipfs_cid, 0, 500));

		assert_ok!(AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 5));

		assert_eq!(pallet_ai_registry::ModelRatingTotal::<Test>::get(0), 5);
		assert_eq!(pallet_ai_registry::ModelRatingCount::<Test>::get(0), 1);

		// Add another rating
		assert_ok!(AIRegistry::rate_model(RuntimeOrigin::signed(3), 0, 3));

		assert_eq!(pallet_ai_registry::ModelRatingTotal::<Test>::get(0), 8);
		assert_eq!(pallet_ai_registry::ModelRatingCount::<Test>::get(0), 2);

		// Check average
		assert_eq!(AIRegistry::get_average_rating(0), Some(4)); // 8/2 = 4
	});
}

#[test]
fn rate_invalid_rating_fails() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		assert_ok!(AIRegistry::register_model(RuntimeOrigin::signed(1), ipfs_cid, 0, 500));

		assert_noop!(
			AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 0),
			pallet_ai_registry::Error::<Test>::InvalidRating
		);

		assert_noop!(
			AIRegistry::rate_model(RuntimeOrigin::signed(2), 0, 6),
			pallet_ai_registry::Error::<Test>::InvalidRating
		);
	});
}

#[test]
fn insufficient_balance_fails() {
	new_test_ext().execute_with(|| {
		let ipfs_cid = b"QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_vec();

		// Account 4 has only 500, needs 1000 stake
		assert_noop!(
			AIRegistry::register_model(RuntimeOrigin::signed(4), ipfs_cid, 0, 500),
			pallet_ai_registry::Error::<Test>::InsufficientStake
		);
	});
}
