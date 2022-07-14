#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::{*, ValueQuery, OptionQuery};
	use frame_system::pallet_prelude::*;
	use frame_support::inherent::Vec;
	use scale_info::prelude::vec;

	#[derive(TypeInfo, Default, Encode, Decode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T:Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
	}

	pub type Id = u32;

	#[derive(TypeInfo, Encode, Decode, Debug)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}

	impl Gender {
		fn get_gender(dna: &Vec<u8>) -> Self {

			let gender = if dna.len() % 2 == 0 {
				Self::Male
			} else {
				Self::Female
			};
			gender
		}
	}

	fn eq(a: u8, b: u8) -> bool {
		a == b
	}

	pub fn vec_compare(va: &[u8], vb: &[u8]) -> bool {
		(va.len() == vb.len()) &&  // zip stops at the shortest
		 va.iter()
		   .zip(vb)
		   .all(|(a,b)| eq(*a,*b))
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn kitty_number)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type TotalKitties<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_counter)]
	pub type KittiesCounter<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		KittyStored(Vec<u8>, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		DuplicatedDna,
		InvalidKitty,
		NoKittyWithDna,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer(origin: OriginFor<T>, receiver: T::AccountId, dna: Vec<u8>) -> DispatchResult {

			let _who = ensure_signed(origin)?;

			let mut kitty = match <Kitties<T>>::get(&dna) {
				None => {
					return Err(Error::<T>::NoKittyWithDna.into())
				},
				Some(kitty) => {
					kitty
				}
			};

			kitty.owner = receiver.clone();
			// <Kitties<T>>::insert(dna.clone(), kitty);

			// Remove kitty from my counter
			let mut my_counter: Vec<Vec<u8>> = match <KittiesCounter<T>>::get(_who.clone()) {
				None => {
					return Err(Error::<T>::InvalidKitty.into())
				},
				Some(counter) => {
					counter
				}
			};
			let remove_idx =  my_counter.iter().position(|x| vec_compare(&x, &dna) == true).unwrap();
			my_counter.remove(remove_idx);

			// Add kitty to receiver counter
			let mut receiver_counter: Vec<Vec<u8>> = match <KittiesCounter<T>>::get(receiver.clone()) {
				None => {
					vec![]
				},
				Some(counter) => {
					counter
				}
			};
			receiver_counter.push(dna);

			// Swap kitty
			<KittiesCounter<T>>::insert(receiver.clone(), receiver_counter);
			<KittiesCounter<T>>::insert(_who.clone(), my_counter);

			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {

			let _who = ensure_signed(origin)?;

			match <Kitties<T>>::get(&dna) {
				Some(kitty) => {
					return Err(Error::<T>::DuplicatedDna.into())
				},
				None => {
					let kitty: Kitty<T> = Kitty {
						dna: dna.clone(),
						owner: _who.clone(),
						price: price,
						gender: Gender::get_gender(&dna)
					};

					<Kitties<T>>::insert(dna.clone(), kitty);

					let new_qty = match <TotalKitties<T>>::get() {
						None => 1,
						Some(qty) => qty + 1
					};
					<TotalKitties<T>>::put(new_qty);

					let mut my_counter: Vec<Vec<u8>> = match <KittiesCounter<T>>::get(_who.clone()) {
						None => {
							vec![]
						},
						Some(counter) => {
							counter
						}
					};
					my_counter.push(dna.clone());
					<KittiesCounter<T>>::insert(_who.clone(), my_counter);

					// Emit an event.
					Self::deposit_event(Event::KittyStored(dna.clone(), _who));
					Ok(())
				}
			}

		}
	}
}
