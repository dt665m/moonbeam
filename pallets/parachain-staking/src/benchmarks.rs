// Copyright 2019-2020 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{BalanceOf, Call, Config, Pallet, Range};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelist_account};
use frame_support::traits::{Currency, Get, ReservableCurrency};
use frame_system::RawOrigin;
use sp_runtime::Perbill;

/// Default balance amount is minimum collator stake
fn default_balance<T: Config>() -> BalanceOf<T> {
	<<T as Config>::MinCollatorStk as Get<BalanceOf<T>>>::get()
}

/// Create a funded user.
fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	T::Currency::make_free_balance_be(&user, total);
	T::Currency::issue(total);
	user
}

/// Create a funded collator. Base amount is MinCollatorStk == default_balance but the
/// last parameter `extra` represents how much additional balance is minted to the collator.
fn create_funded_collator<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> Result<T::AccountId, &'static str> {
	let user = create_funded_user::<T>(string, n, extra);
	Pallet::<T>::join_candidates(
		RawOrigin::Signed(user.clone()).into(),
		default_balance::<T>(),
	)?;
	Ok(user)
}

const USER_SEED: u32 = 999666;

benchmarks! {
	join_candidates {
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let min_collator_stk = default_balance::<T>();
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), min_collator_stk)
	verify {
		assert!(Pallet::<T>::is_candidate(&caller));
	}

	leave_candidates {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 0u32.into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		// TODO: roll_2_rounds and ensure is_candidate == false
		assert!(Pallet::<T>::collator_state(&caller).unwrap().is_leaving());
	}

	go_offline {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 0u32.into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::collator_state(&caller).unwrap().is_active());
	}

	go_online {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 0u32.into())?;
		Pallet::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::collator_state(&caller).unwrap().is_active());
	}

	candidate_bond_more {
		let balance = default_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, balance)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), balance)
	verify {
		let expected_bond = balance * 2u32.into();
		assert_eq!(T::Currency::reserved_balance(&caller), expected_bond);
	}

	candidate_bond_less {
		let balance = default_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, balance)?;
		Pallet::<T>::candidate_bond_more(RawOrigin::Signed(caller.clone()).into(), balance)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), balance)
	verify {
		assert_eq!(T::Currency::reserved_balance(&caller), balance);
	}

	nominate {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator, bond)
	verify {
		assert!(Pallet::<T>::is_nominator(&caller));
	}

	leave_nominators {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator, bond)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::is_nominator(&caller));
	}

	revoke_nomination {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), bond)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator)
	verify {
		assert!(!Pallet::<T>::is_nominator(&caller));
	}

	nominator_bond_more {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), bond)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator, bond)
	verify {
		let expected_bond = bond * 2u32.into();
		assert_eq!(T::Currency::reserved_balance(&caller), expected_bond);
	}

	nominator_bond_less {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let total = default_balance::<T>();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), total)?;
		let bond_less = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator, bond_less)
	verify {
		let expected = total - bond_less;
		assert_eq!(T::Currency::reserved_balance(&caller), expected);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use frame_support::assert_ok;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}

	#[test]
	fn bench_join_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_join_candidates::<Test>());
		});
	}

	#[test]
	fn bench_leave_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_leave_candidates::<Test>());
		});
	}

	#[test]
	fn bench_go_offline() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_go_offline::<Test>());
		});
	}

	#[test]
	fn bench_go_online() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_go_online::<Test>());
		});
	}

	#[test]
	fn bench_candidate_bond_more() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_candidate_bond_more::<Test>());
		});
	}

	#[test]
	fn bench_candidate_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_candidate_bond_less::<Test>());
		});
	}

	#[test]
	fn bench_nominate() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_nominate::<Test>());
		});
	}

	#[test]
	fn bench_leave_nominators() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_leave_nominators::<Test>());
		});
	}

	#[test]
	fn bench_revoke_nomination() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_revoke_nomination::<Test>());
		});
	}

	#[test]
	fn bench_nominator_bond_more() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_nominator_bond_more::<Test>());
		});
	}

	#[test]
	fn bench_nominator_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_nominator_bond_less::<Test>());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
