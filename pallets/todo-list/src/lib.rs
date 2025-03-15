#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_std::vec::Vec;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type TodoCount: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Task not found
        TaskNotFound,
        /// Not the task owner
        NotTaskOwner,
        /// Task already completed
        TaskAlreadyCompleted,
        /// Description too long
        DescriptionTooLong,
        /// Invalid task ID
        InvalidTaskId,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        TaskCreated { id: u32, owner: T::AccountId, description: BoundedVec<u8, T::TodoCount> },
        TaskCompleted { id: u32, owner: T::AccountId },
    }

    #[pallet::storage]
    pub type Todos<T: Config> = StorageMap<_, Blake2_128Concat, u32, (BoundedVec<u8, T::TodoCount>, T::AccountId, bool), OptionQuery>;

    #[pallet::storage]
    pub type TodoCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_task(origin: OriginFor<T>, description: Vec<u8>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let description = BoundedVec::try_from(description)
                .map_err(|_| Error::<T>::DescriptionTooLong)?;
            
            let count = TodoCount::<T>::get();
            let id = count + 1;
            
            Todos::<T>::insert(id, (description.clone(), sender.clone(), false));
            TodoCount::<T>::put(id);
            
            Self::deposit_event(Event::TaskCreated { id, owner: sender, description });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn complete_task(origin: OriginFor<T>, task_id: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Validate task ID
            if task_id == 0 {
                return Err(Error::<T>::InvalidTaskId.into());
            }

            // Get task details
            let task = Todos::<T>::get(task_id)
                .ok_or(Error::<T>::TaskNotFound)?;
            
            let (description, owner, completed) = task;
            
            // Check ownership
            if owner != sender {
                return Err(Error::<T>::NotTaskOwner.into());
            }
            
            // Check completion status
            if completed {
                return Err(Error::<T>::TaskAlreadyCompleted.into());
            }
            
            // Update task status
            Todos::<T>::insert(task_id, (description, owner.clone(), true));
            Self::deposit_event(Event::TaskCompleted { id: task_id, owner });
            Ok(())
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking {
    use super::*;
    use frame_benchmarking::{benchmarks, whitelisted_caller};
    use frame_system::RawOrigin;

    benchmarks! {
        create_task {
            let caller = whitelisted_caller();
            let description = vec![0u8; 100];
        }: create_task(RawOrigin::Signed(caller), description)
        verify {
            assert_eq!(TodoCount::<T>::get(), 1);
        }

        complete_task {
            let caller = whitelisted_caller();
            let description = vec![0u8; 100];
            let _ = Pallet::<T>::create_task(RawOrigin::Signed(caller.clone()).into(), description);
        }: complete_task(RawOrigin::Signed(caller), 1)
        verify {
            let (_, _, completed) = Todos::<T>::get(1).unwrap();
            assert!(completed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Origin, Test, System, TodoList};
    use frame_support::{assert_ok, assert_noop};

    #[test]
    fn test_create_task() {
        new_test_ext().execute_with(|| {
            let description = vec![0u8; 100];
            assert_ok!(TodoList::create_task(Origin::signed(1), description.clone()));
            assert_eq!(TodoCount::<Test>::get(), 1);
            let (stored_description, owner, completed) = Todos::<Test>::get(1).unwrap();
            assert_eq!(stored_description, BoundedVec::try_from(description).unwrap());
            assert_eq!(owner, 1);
            assert!(!completed);
        });
    }

    #[test]
    fn test_complete_task() {
        new_test_ext().execute_with(|| {
            let description = vec![0u8; 100];
            assert_ok!(TodoList::create_task(Origin::signed(1), description));
            assert_ok!(TodoList::complete_task(Origin::signed(1), 1));
            let (_, _, completed) = Todos::<Test>::get(1).unwrap();
            assert!(completed);
        });
    }

    #[test]
    fn test_complete_task_not_owner() {
        new_test_ext().execute_with(|| {
            let description = vec![0u8; 100];
            assert_ok!(TodoList::create_task(Origin::signed(1), description));
            assert_noop!(
                TodoList::complete_task(Origin::signed(2), 1),
                Error::<Test>::NotTaskOwner
            );
        });
    }

    #[test]
    fn test_complete_task_not_found() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                TodoList::complete_task(Origin::signed(1), 1),
                Error::<Test>::TaskNotFound
            );
        });
    }

    #[test]
    fn test_complete_task_invalid_id() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                TodoList::complete_task(Origin::signed(1), 0),
                Error::<Test>::InvalidTaskId
            );
        });
    }
}

#[cfg(test)]
mod mock {
    use super::*;
    use frame_support::construct_runtime;

    construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system,
            TodoList: pallet,
        }
    );

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type TodoCount = frame_support::traits::ConstU32<100>;
    }

    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = sp_core::H256;
        type Hashing = sp_runtime::traits::BlakeTwo256;
        type AccountId = u64;
        type Lookup = sp_runtime::identity::IdentityLookup<Self::AccountId>;
        type Header = sp_runtime::generic::Header<BlockNumber, Hashing>;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = ();
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        sp_io::TestExternalities::new(t)
    }
}

