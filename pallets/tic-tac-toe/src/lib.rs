// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod test;

mod game;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;

pub(crate) use game::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	pub type BoardOf<T> = Board<<T as frame_system::Config>::AccountId>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	pub type ActivePlayers<T: Config> = StorageMap<_, Identity, T::AccountId, GameId>;

	#[pallet::storage]
	pub type PendingGames<T: Config> = StorageMap<_, Identity, GameId, BoardOf<T>>;

	#[pallet::storage]
	pub type ActiveGames<T: Config> = StorageMap<_, Identity, GameId, BoardOf<T>>;

	#[pallet::storage]
	pub type NextGameId<T: Config> = StorageValue<_, GameId, ValueQuery>;

	#[pallet::storage]
	pub type LastActiveGameId<T: Config> = StorageValue<_, GameId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new game has been created.
		GameCreated { game_id: GameId, by: T::AccountId },
		/// A player has joined a pending game.
		GameJoined { game_id: GameId, by: T::AccountId },
		/// A player has executed a move.
		MovePlayed { at: Coordinates, by: T::AccountId },
		/// A game has finished, if draw then no winner will be shown.
		GameFinished { game_id: GameId, winner: Option<T::AccountId> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A player that is involved in an active game cannot create new ones.
		CannotCreateNewGame,
		/// A player that is involved in an active game cannot join another one.
		CannotJoinAnotherGame,
		/// The player is nt currently involved in any game.
		PlayerNotActive,
		/// There are no pending games to join, please create one.
		NoPendingGamesFound,
		/// Tried to play while not in their turn
		InvalidTurn,
		/// Tried to play on an already filled cell
		InvalidCell,
		/// The game is in an incorrect state
		InvalidGameState,
		/// The game needs and additional player to start
		GamePending,
		/// The game is already over
		GameAlreadyFinished,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({10_000})]
		pub fn create_game(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;

			ensure!(!ActivePlayers::<T>::contains_key(&account), Error::<T>::CannotCreateNewGame);

			let game_id = Self::next_game_id();
			let board = BoardOf::<T>::new(account.clone());

			ActivePlayers::<T>::insert(account.clone(), game_id);
			PendingGames::<T>::insert(game_id, board);

			Self::deposit_event(Event::GameCreated { game_id, by: account });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		pub fn join_pending_game(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			ensure!(!ActivePlayers::<T>::contains_key(&account), Error::<T>::CannotJoinAnotherGame);

			let game_id = LastActiveGameId::<T>::get();
			ensure!(PendingGames::<T>::contains_key(game_id), Error::<T>::NoPendingGamesFound);

			let game = {
				let mut game = PendingGames::<T>::take(game_id).unwrap();
				game.start_game(account.clone());

				game
			};

			LastActiveGameId::<T>::set(game_id.saturating_add(1) % GameId::MAX);
			ActiveGames::<T>::insert(game_id, game);
			ActivePlayers::<T>::insert(account.clone(), game_id);

			Self::deposit_event(Event::GameJoined { game_id, by: account });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({10_000})]
		pub fn play_move(origin: OriginFor<T>, at: Coordinates) -> DispatchResult {
			let account = ensure_signed(origin)?;

			let maybe_game_id = ActivePlayers::<T>::get(&account);
			ensure!(maybe_game_id.is_some(), Error::<T>::PlayerNotActive);

			let game_id = maybe_game_id.unwrap();

			ActiveGames::<T>::mutate(game_id, |maybe_game| {
				if let Some(ref mut game) = maybe_game {
					let results = game.play_turn(&account, at);

					match results {
						PlayResult::Winner(acc) =>
							if let BoardState::Finished(p1, p2) = game.get_state() {
								Self::remove_players(p1, p2);
								Self::deposit_event(Event::<T>::GameFinished {
									game_id,
									winner: Some(acc),
								});
								*maybe_game = None;
								Ok(())
							} else {
								Self::clear_corrupt_game(game_id);
								Err(Error::<T>::InvalidGameState.into())
							},
						PlayResult::Draw =>
							if let BoardState::Finished(p1, p2) = game.get_state() {
								Self::remove_players(p1, p2);
								Self::deposit_event(Event::<T>::GameFinished {
									game_id,
									winner: None,
								});
								*maybe_game = None;
								Ok(())
							} else {
								Self::clear_corrupt_game(game_id);
								Err(Error::<T>::InvalidGameState.into())
							},
						PlayResult::InvalidTurn => Err(Error::<T>::InvalidTurn.into()),
						PlayResult::InvalidCell => Err(Error::<T>::InvalidCell.into()),
						PlayResult::GamePending => Err(Error::<T>::GamePending.into()),
						PlayResult::GameAlreadyFinished =>
							Err(Error::<T>::GameAlreadyFinished.into()),
						PlayResult::Continue => {
							Self::deposit_event(Event::<T>::MovePlayed { at, by: account });
							Ok(())
						},
					}
				} else {
					Self::clear_corrupt_game(game_id);
					Err(Error::<T>::InvalidGameState.into())
				}
			})
		}
	}

	impl<T: Config> Pallet<T> {
		fn next_game_id() -> GameId {
			let next_game_id = NextGameId::<T>::get();

			NextGameId::<T>::mutate(|value| *value = value.saturating_add(1) % GameId::MAX);

			next_game_id
		}

		fn remove_players(player_1: T::AccountId, player_2: T::AccountId) {
			ActivePlayers::<T>::remove(player_1);
			ActivePlayers::<T>::remove(player_2);
		}

		fn clear_corrupt_game(game_id: GameId) {
			ActiveGames::<T>::remove(game_id);
			PendingGames::<T>::remove(game_id);

			let accounts = ActivePlayers::<T>::iter()
				.filter(|(_, id)| *id == game_id)
				.map(|(acc, _)| acc)
				.collect::<sp_std::vec::Vec<_>>();

			for acc in accounts {
				ActivePlayers::<T>::remove(acc);
			}
		}
	}
}

sp_core::generate_feature_enabled_macro!(runtime_benchmarks_enabled, feature = "runtime-benchmarks", $);
