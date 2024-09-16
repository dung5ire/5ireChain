use super::*;
use frame_support::traits::OnRuntimeUpgrade;

use log::{log, Level};

#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

pub mod v1 {
	use frame_support::{pallet_prelude::*, weights::Weight};

	use super::*;
	const TARGET: &'static str = "runtime::reward::migration::v1";

	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			log::info!(
				target: TARGET,
				"Running migration with onchain storage version {:?}",
				onchain_version
			);

			if onchain_version == 0 {
				let mut count = 0;
				for (nominator_id, balance) in NominatorRewardAccounts::<T>::iter() {
					if let Some(nominations) =
						pallet_staking::Nominators::<T>::get(nominator_id.clone())
					{
						if let Some(validator) = nominations.targets.first() {
							NominatorEarningsAccount::<T>::insert(
								validator.clone(),
								nominator_id.clone(),
								balance,
							);
						}
						NominatorRewardAccounts::<T>::remove(nominator_id.clone());
					}
					count += 1;
				}
				StorageVersion::new(1).put::<Pallet<T>>();
				log!(Level::Info, "reward v1 applied successfully");
				T::DbWeight::get().reads_writes((count as u64) + 1, (count as u64) + 1)
			} else {
				log!(Level::Warn, "Skipping reward v1, should be removed");
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			let prev_count = NominatorRewardAccounts::<T>::iter().count();
			Ok((prev_count as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(prev_count: Vec<u8>) -> Result<(), TryRuntimeError> {
			let prev_count_old_storage: u32 = Decode::decode(&mut prev_count.as_slice()).expect(
				"the state parameter should be something that was generated by pre_upgrade",
			);
			let post_count_old_storage = NominatorRewardAccounts::<T>::iter().count() as u32;
			ensure!(
				post_count_old_storage == 0 ,
				"count should be 0 in NominatorRewardAccounts"
			);

			// compare old storage key length (NominatorRewardAccounts) and new storage key length (NominatorEarningsAccount)
			let post_count_new_storage = NominatorEarningsAccount::<T>::iter().count() as u32;
			ensure!(
				prev_count_old_storage ==  post_count_new_storage,
				"length should be the same"
			);
			ensure!(Pallet::<T>::on_chain_storage_version() == 1, "wrong storage version");

			Ok(())
		}
	}
}