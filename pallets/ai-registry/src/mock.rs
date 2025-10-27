//! Mock runtime for AI Registry pallet tests

use crate as pallet_ai_registry;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU128, ConstU32},
};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
	// The main runtime
	#[runtime::runtime]
	// Runtime Types to be generated
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
	pub const MaxCidLength: u32 = 128;
	pub const MaxNameLength: u32 = 256;
	pub const MaxDescriptionLength: u32 = 1024;
}

impl pallet_ai_registry::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = Balances;
	type MinimumModelStake = MinimumModelStake;
	type RegistrationFee = RegistrationFee;
	type MaxCidLength = MaxCidLength;
	type MaxNameLength = MaxNameLength;
	type MaxDescriptionLength = MaxDescriptionLength;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(1, 10000), // Account with sufficient balance
			(2, 10000),
			(3, 10000),
			(4, 500),   // Account with insufficient balance
		],
		dev_accounts: vec![],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
