#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
use sp_runtime::BoundedVec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub type Rating = u16;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// Maximum number of players that can join a single game
		#[pallet::constant]
		type MaxPlayers: Get<u32>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn DefaultRating() -> Rating {
		1000u16
	}

	#[pallet::storage]
	#[pallet::getter(fn get_rating)]
	pub type RatingStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, Rating, ValueQuery, DefaultRating>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Player has won and gained rating
		RatingGained { player: AccountIdOf<T>, new_rating: Rating, rating_gained: Rating },

		// Player has lost and lost rating
		RatingLost { player: AccountIdOf<T>, new_rating: Rating, rating_lost: Rating },
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
	fn do_update_rating(winner: &AccountIdOf<T>, loser: &AccountIdOf<T>) {
		let a: Rating = RatingStorage::<T>::get(winner);
		let b: Rating = RatingStorage::<T>::get(loser);

		let rating_change = Self::get_rating_change(&a, &b);

		let winner_new_rating = a.saturating_add(rating_change);
		let loser_new_rating = b.saturating_sub(rating_change);

		RatingStorage::<T>::set(winner, winner_new_rating);
		RatingStorage::<T>::set(loser, loser_new_rating);

		Self::deposit_event(Event::RatingGained {
			player: winner.clone(),
			new_rating: winner_new_rating,
			rating_gained: rating_change,
		});
		Self::deposit_event(Event::RatingLost {
			player: loser.clone(),
			new_rating: loser_new_rating,
			rating_lost: rating_change,
		});
	}

	fn do_lose_rating(
		player: &AccountIdOf<T>,
		amount: Rating,
	) {
		let rating: Rating = RatingStorage::<T>::get(player);

		let new_rating = rating.saturating_sub(amount);

		RatingStorage::<T>::set(player, new_rating);

		Self::deposit_event(Event::RatingLost {
			player: player.clone(),
			new_rating,
			rating_lost: amount,
		});
	}

	fn do_update_ratings(
		winner: &AccountIdOf<T>,
		losers: &BoundedVec<AccountIdOf<T>, <T as Config>::MaxPlayers>,
	) {
		let a: Rating = RatingStorage::<T>::get(winner);

		let mut winner_rating_change: Rating = 0;

		for loser in losers.iter() {
			if loser == winner {
				continue;
			}

			let b: Rating = RatingStorage::<T>::get(loser);

			let rating_change = Self::get_rating_change(&a, &b);

			winner_rating_change = winner_rating_change.saturating_add(rating_change);
			let loser_new_rating = b.saturating_sub(rating_change);

			RatingStorage::<T>::set(loser, loser_new_rating);

			Self::deposit_event(Event::RatingLost {
				player: loser.clone(),
				new_rating: loser_new_rating,
				rating_lost: rating_change,
			});
		}

		let winner_new_rating = a.saturating_add(winner_rating_change);
		RatingStorage::<T>::set(winner, winner_new_rating);

		Self::deposit_event(Event::RatingGained {
			player: winner.clone(),
			new_rating: winner_new_rating,
			rating_gained: winner_rating_change,
		});
	}

	fn get_rating_change(a: &Rating, b: &Rating) -> Rating {
		if b > a {
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
		}
	}
}

impl<T: Config> EloFunc<AccountIdOf<T>, <T as Config>::MaxPlayers> for Pallet<T> {
	fn update_rating(winner: &AccountIdOf<T>, loser: &AccountIdOf<T>) {
		Self::do_update_rating(winner, loser)
	}

	fn lose_rating(player: &AccountIdOf<T>, amount: Rating) {
		Self::do_lose_rating(player, amount);
	}

	fn update_ratings(
		winner: &AccountIdOf<T>,
		losers: &BoundedVec<AccountIdOf<T>, <T as Config>::MaxPlayers>,
	) {
		Self::do_update_ratings(winner, losers)
	}

	fn get_rating(player: &AccountIdOf<T>) -> Rating {
		RatingStorage::<T>::get(player)
	}
}

pub trait EloFunc<AccountId, MaxAccounts> {
	fn update_rating(winner: &AccountId, loser: &AccountId);

	fn lose_rating(player: &AccountId, amount: Rating);

	fn update_ratings(winner: &AccountId, losers: &BoundedVec<AccountId, MaxAccounts>);

	fn get_rating(player: &AccountId) -> Rating;
}
