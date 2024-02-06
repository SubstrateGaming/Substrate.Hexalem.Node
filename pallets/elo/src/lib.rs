#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub type Rating = u16;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn DefaultRating() -> u16 {
		1000u16
	}

	#[pallet::storage]
	#[pallet::getter(fn get_rating)]
	pub type RatingStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Rating, ValueQuery, DefaultRating>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn do_update_rating(winner: &AccountIdOf<T>, loser: &AccountIdOf<T>) -> () {
		let a: u16 = RatingStorage::<T>::get(winner);
		let b: u16 = RatingStorage::<T>::get(loser);

		let rating_change = if b > a {
			match b - a {
				0..=49 => 16,
				50..=99 => 17,
				100..=149 => 18,
				150..=199 => 19,
				200..=224 => 20,
				225..=249 => 21,
				250..=274 => 22,
				275..=299 => 23,
				300..=324 => 24,
				325..=349 => 25,
				350..=374 => 26,
				375..=399 => 27,
				400..=424 => 28,
				425..=449 => 29,
				450..=474 => 30,
				475..=499 => 31,
				_ => 32,
			}
		} else {
			match a - b {
				0..=49 => 16,
				50..=99 => 15,
				100..=149 => 14,
				150..=199 => 13,
				200..=224 => 12,
				225..=249 => 11,
				250..=274 => 10,
				275..=299 => 9,
				300..=324 => 8,
				325..=349 => 7,
				350..=374 => 6,
				375..=399 => 5,
				400..=424 => 4,
				425..=449 => 3,
				450..=474 => 2,
				_ => 1,
			}
		};

		RatingStorage::<T>::set(winner, a.saturating_add(rating_change));
		RatingStorage::<T>::set(loser, b.saturating_sub(rating_change));
	}
}

impl<T: Config> EloFunc<AccountIdOf<T>> for Pallet<T> {
	fn update_rating(winner: &AccountIdOf<T>, loser: &AccountIdOf<T>) -> () {
		Self::do_update_rating(winner, loser)
	}
}

pub trait EloFunc<AccountId> {
	/// empty specific bracket queue
	fn update_rating(winner: &AccountId, loser: &AccountId) -> ();
}
