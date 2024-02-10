#![cfg_attr(not(feature = "std"), no_std)]

use core::cmp;

use crate::vec::Vec;
use frame_system::pallet_prelude::BlockNumberFor;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//#[cfg(any(test, feature = "runtime-benchmarks"))]
//mod benchmarking;

mod types;
pub mod weights;

pub use crate::{types::*, weights::*};

use frame_support::{
	ensure, sp_runtime, sp_runtime::SaturatedConversion, traits::Get, StorageHasher,
};
use scale_info::prelude::vec;

use pallet_elo::EloFunc;
use pallet_matchmaker::MatchFunc;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	pub type TileSelectionOf<T> = TileSelection<<T as Config>::MaxTileSelection>;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub type HexGridOf<T> = HexGrid<<T as Config>::Tile, <T as Config>::MaxHexGridSize>;
	pub type HexBoardOf<T> = HexBoard<<T as Config>::Tile, <T as Config>::MaxHexGridSize>;

	pub type GameOf<T> = Game<
		AccountIdOf<T>,
		BlockNumberFor<T>,
		<T as Config>::MaxPlayers,
		<T as Config>::MaxTileSelection,
	>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		// Maximum number of players that can join a single game
		#[pallet::constant]
		type MaxPlayers: Get<u32> + Parameter;

		// Minimum number of players that can join a single game
		#[pallet::constant]
		type MinPlayers: Get<u8>;

		#[pallet::constant]
		type MaxRounds: Get<u8>;

		#[pallet::constant]
		type BlocksToPlayLimit: Get<u8>;

		#[pallet::constant]
		type MaxHexGridSize: Get<u32> + Parameter;

		#[pallet::constant]
		type MaxTileSelection: Get<u32> + Parameter;

		type Tile: Encode
			+ Decode
			+ TypeInfo
			+ Clone
			+ Copy
			+ PartialEq
			+ MaxEncodedLen
			+ Parameter
			+ Default
			+ GetTileInfo;

		#[pallet::constant]
		type TileCosts: Get<[TileCost<Self::Tile>; 15]>;

		#[pallet::constant]
		type TileResourceProductions: Get<[ResourceProductions; NUMBER_OF_TILE_TYPES]>;

		#[pallet::constant]
		type WaterPerHuman: Get<u8>;

		#[pallet::constant]
		type FoodPerHuman: Get<u8>;

		#[pallet::constant]
		type HomePerHumans: Get<u8>;

		#[pallet::constant]
		type FoodPerTree: Get<u8>;

		#[pallet::constant]
		type DefaultPlayerResources: Get<[ResourceUnit; 7]>;

		#[pallet::constant]
		type TargetGoalGold: Get<ResourceUnit>;

		#[pallet::constant]
		type TargetGoalHuman: Get<ResourceUnit>;

		type Matchmaker: MatchFunc<Self::AccountId>;

		type Elo: EloFunc<Self::AccountId, Self::MaxPlayers>;
	}

	#[pallet::storage]
	// Stores the Game data assigned to a creator address key
	pub type StoreTest<T: Config> = StorageMap<_, Blake2_128Concat, GameId, TileSelectionOf<T>>;

	#[pallet::storage]
	// Stores the Game data assigned to a creator address key
	pub type GameStorage<T: Config> = StorageMap<_, Blake2_128Concat, GameId, GameOf<T>>;

	#[pallet::storage]
	// Stores the HexBoard data assigned to a player key.
	pub type HexBoardStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, HexBoardOf<T>>;

	#[pallet::storage]
	// Stores the TargetGoalHash assigned to a player key.
	pub type TargetGoalStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, AccountIdOf<T>, TargetGoalHash>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Game started
		GameCreated { game_id: GameId, grid_size: u8, players: Vec<AccountIdOf<T>> },

		// Player played a move
		MovePlayed { game_id: GameId, player: AccountIdOf<T>, move_played: Move },

		TileUpgraded { game_id: GameId, player: AccountIdOf<T>, place_index: u8 },

		// New selection has been drawn
		NewTileSelection { game_id: GameId, selection: TileSelectionOf<T> },

		// Selection has been refilled
		SelectionRefilled { game_id: GameId, selection: TileSelectionOf<T> },

		TurnForceFinished { game_id: GameId, player: AccountIdOf<T> },

		// New turn
		NewTurn { game_id: GameId, next_player: AccountIdOf<T> },

		// Game has finished
		GameFinished { game_id: GameId /* , winner: AccountIdOf<T> */ },

		// Event that is never used. It serves the purpose to expose hidden rust enums
		ExposeEnums { tile_type: TileType, tile_pattern: TilePattern },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// Player has already initialised a game. They need to finish it.
		AlreadyPlaying,

		// Game has not been initialized yet. Unable to join it.
		GameNotInitialized,

		// HexBoard has not been initialized yet. Unable to play.
		HexBoardNotInitialized,

		// HexBoard state is set to Matchmaking
		HexBoardInMatchmakingState,

		// Creator needs to be included among players at index 0
		CreatorNotInPlayersAtIndexZero,

		// The game has already started. Can not create it twice.
		GameAlreadyCreated,

		// Other errors, that should never happen.
		InternalError,

		// Please set the number_of_players parameter to a bigger number.
		NumberOfPlayersIsTooSmall,

		// Please set the number_of_players parameter to a smaller number.
		NumberOfPlayersIsTooLarge,

		// Math overflow.
		MathOverflow,

		// Not enough resources to pay for the tile offer.
		NotEnoughResources,

		// Not enough population to play all moves.
		NotEnoughPopulation,

		// Entered index for buying is out of bounds.
		BuyIndexOutOfBounds,

		// Entered index for placing the tile is out of bounds.
		PlaceIndexOutOfBounds,

		// Player is not on the turn.
		PlayerNotOnTurn,

		// Player is not playing this game
		PlayerNotInGame,

		// Current player cannot force finish his own turn
		CurrentPlayerCannotForceFinishTurn,

		// Game has not started yet, or has been finished already.
		GameNotPlaying,

		// The grid size is not 9, 25, 49.
		BadGridSize,

		// You can not place a tile on another tile, unless it is empty.
		TileIsNotEmpty,

		// Tile is already on the max level.
		TileOnMaxLevel,

		// Can not level up empty tile.
		CannotLevelUpEmptyTile,

		// This tile can not be leveled up.
		CannotLevelUp,

		// The tile is surrounded by empty tiles.
		TileSurroundedByEmptyTiles,

		// Not enough blocks have passed to force finish turn
		BlocksToPlayLimitNotPassed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn create_game(
			origin: OriginFor<T>,
			players: Vec<AccountIdOf<T>>,
			grid_size: u8,
		) -> DispatchResult {
			let who: AccountIdOf<T> = ensure_signed(origin)?;

			// If you want to play, you need to specify yourself in the Vec as well
			let number_of_players = players.len();

			ensure!(
				number_of_players >= T::MinPlayers::get() as usize,
				Error::<T>::NumberOfPlayersIsTooSmall
			);

			ensure!(
				number_of_players <= T::MaxPlayers::get() as usize,
				Error::<T>::NumberOfPlayersIsTooLarge
			);

			ensure!(Self::is_valid_grid_size(grid_size), Error::<T>::BadGridSize);

			// Random GameId
			// I used `who` to ensure that even if 2 independent players wanted to create game in
			// the same block, they would be able to.
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			let game_id: GameId = Blake2_256::hash(&(&who, &current_block_number).encode());

			ensure!(players[0] == who, Error::<T>::CreatorNotInPlayersAtIndexZero);

			// Ensure that the game has not already been created
			ensure!(!GameStorage::<T>::contains_key(game_id), Error::<T>::GameAlreadyCreated);

			// Initialise HexBoards for all players
			for player in &players {
				ensure!(!HexBoardStorage::<T>::contains_key(player), Error::<T>::AlreadyPlaying);

				HexBoardStorage::<T>::set(
					player,
					Some(
						HexBoardOf::<T>::try_new::<T::DefaultPlayerResources>(
							grid_size as usize,
							MatchmakingState::Joined(game_id),
						)
						.ok_or(Error::<T>::InternalError)?,
					),
				);
			}

			// Default Game Config
			Self::do_create_new_game(game_id, current_block_number, players, grid_size)
		}

		#[pallet::call_index(100)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn queue(origin: OriginFor<T>) -> DispatchResult {
			let who: AccountIdOf<T> = ensure_signed(origin)?;

			// Make sure player has no board open.
			ensure!(!HexBoardStorage::<T>::contains_key(&who), Error::<T>::AlreadyPlaying);

			// Perhaps in the future, we might want to allow players to play on other grid sizes
			let grid_size: u8 = 25;

			HexBoardStorage::<T>::set(
				&who,
				Some(
					HexBoardOf::<T>::try_new::<T::DefaultPlayerResources>(
						grid_size as usize,
						MatchmakingState::Matchmaking,
					)
					.ok_or(Error::<T>::InternalError)?,
				),
			);

			// This might change with the introduction of ELO
			let bracket: u8 = 0;

			// Add player to queue, duplicate check is done in matchmaker.
			T::Matchmaker::add_queue(who, bracket)?;

			let potential_players = T::Matchmaker::try_match();

			// if result is not empty we have a valid match
			if !potential_players.is_empty() {
				// Random GameId
				// I used `potential_players` to ensure that even if 2 independent players wanted to
				// create game in the same block, they would be able to.
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let game_id: GameId =
					Blake2_256::hash(&(&potential_players[0], &current_block_number).encode());

				for player in &potential_players {
					// Ensures that the HexBoard exists
					let mut hex_board = match HexBoardStorage::<T>::get(player) {
						Some(value) => value,
						None => return Err(Error::<T>::HexBoardNotInitialized.into()),
					};

					hex_board.matchmaking_state = MatchmakingState::Joined(game_id);

					HexBoardStorage::<T>::set(player, Some(hex_board));
				}

				// Create new game
				Self::do_create_new_game(
					game_id,
					current_block_number,
					potential_players,
					grid_size,
				)?;

				// Maybe adjust the weight
			}

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn play(origin: OriginFor<T>, move_played: Move) -> DispatchResult {
			let who: AccountIdOf<T> = ensure_signed(origin)?;

			// Ensures that the HexBoard exists
			let mut hex_board = match HexBoardStorage::<T>::get(&who) {
				Some(value) => value,
				None => return Err(Error::<T>::HexBoardNotInitialized.into()),
			};

			let game_id: GameId =
				hex_board.get_game_id().ok_or(Error::<T>::HexBoardInMatchmakingState)?;

			// Ensures that the Game exists
			let mut game = match GameStorage::<T>::get(game_id) {
				Some(value) => value,
				None => return Err(Error::<T>::GameNotInitialized.into()),
			};

			ensure!(game.state == GameState::Playing, Error::<T>::GameNotPlaying);

			ensure!(
				game.borrow_players()[game.get_player_turn() as usize] == who,
				Error::<T>::PlayerNotOnTurn
			);

			ensure!(
				hex_board.hex_grid.len() > move_played.place_index as usize,
				Error::<T>::PlaceIndexOutOfBounds
			);

			ensure!(
				hex_board.hex_grid[move_played.place_index as usize].get_type() == TileType::Empty,
				Error::<T>::TileIsNotEmpty
			);

			// buy and place the move
			hex_board.hex_grid[move_played.place_index as usize] = Self::buy_from_selection(
				&mut game.selection,
				&mut hex_board,
				move_played.buy_index as usize,
			)?;

			game.set_played(true);

			Self::refill_selection(&mut game, game_id)?;

			// Check formations
			let grid_length: usize = hex_board.hex_grid.len();

			let side_length: i8 = Self::side_length(&grid_length);
			let max_distance: i8 = Self::max_distance_from_center(&grid_length);
			let (tile_q, tile_r) =
				Self::index_to_coords(move_played.place_index, &side_length, &max_distance)?;

			let mut neighbours = Self::get_neighbouring_tiles(&max_distance, &tile_q, &tile_r)?;
			ensure!(
				Self::not_surrounded_by_empty_tiles(
					&neighbours,
					&hex_board.hex_grid,
					&max_distance,
					&side_length
				),
				Error::<T>::TileSurroundedByEmptyTiles
			);

			neighbours.push(Some((tile_q, tile_r)));

			for tile in neighbours.into_iter().flatten() {
				Self::set_patterns(&mut hex_board, tile)?;
			}

			GameStorage::<T>::set(game_id, Some(game));
			HexBoardStorage::<T>::set(&who, Some(hex_board));

			Self::deposit_event(Event::MovePlayed { game_id, player: who, move_played });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn upgrade(origin: OriginFor<T>, place_index: u8) -> DispatchResult {
			let who: AccountIdOf<T> = ensure_signed(origin)?;

			// Ensures that the HexBoard exists
			let mut hex_board = match HexBoardStorage::<T>::get(&who) {
				Some(value) => value,
				None => return Err(Error::<T>::HexBoardNotInitialized.into()),
			};

			let game_id: GameId =
				hex_board.get_game_id().ok_or(Error::<T>::HexBoardInMatchmakingState)?;

			// Ensures that the Game exists
			let game = match GameStorage::<T>::get(game_id) {
				Some(value) => value,
				None => return Err(Error::<T>::GameNotInitialized.into()),
			};

			ensure!(game.state == GameState::Playing, Error::<T>::GameNotPlaying);

			ensure!(
				game.borrow_players()[game.get_player_turn() as usize] == who,
				Error::<T>::PlayerNotOnTurn
			);

			ensure!(
				hex_board.hex_grid.len() > place_index as usize,
				Error::<T>::PlaceIndexOutOfBounds
			);

			let tile_to_upgrade: T::Tile = hex_board.hex_grid[place_index as usize];

			let tile_level = tile_to_upgrade.get_level();

			ensure!(tile_level != 3, Error::<T>::TileOnMaxLevel);

			Self::spend_for_tile_upgrade(&mut hex_board, &tile_to_upgrade)?;

			hex_board.hex_grid[place_index as usize].set_level(tile_level.saturating_add(1));

			HexBoardStorage::<T>::set(&who, Some(hex_board));

			Self::deposit_event(Event::TileUpgraded { game_id, player: who, place_index });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn finish_turn(origin: OriginFor<T>) -> DispatchResult {
			let who: AccountIdOf<T> = ensure_signed(origin)?;

			// Ensures that the HexBoard exists
			let mut hex_board = match HexBoardStorage::<T>::get(&who) {
				Some(value) => value,
				None => return Err(Error::<T>::HexBoardNotInitialized.into()),
			};

			let game_id: GameId =
				hex_board.get_game_id().ok_or(Error::<T>::HexBoardInMatchmakingState)?;

			// Ensures that the Game exists
			let mut game = match GameStorage::<T>::get(game_id) {
				Some(value) => value,
				None => return Err(Error::<T>::GameNotInitialized.into()),
			};

			ensure!(game.state == GameState::Playing, Error::<T>::GameNotPlaying);

			ensure!(
				game.borrow_players()[game.get_player_turn() as usize] == who,
				Error::<T>::PlayerNotOnTurn
			);

			let current_block_number = <frame_system::Pallet<T>>::block_number();
			game.last_played_block = current_block_number;

			// If the player has not played, generate a new selection
			if game.get_played() {
				game.set_played(false);
			} else {
				Self::new_selection(&mut game, game_id)?;
			}

			// Update the resources
			Self::evaluate_board(&mut hex_board);

			if Self::is_game_won(&hex_board) {
				match game.borrow_players().len() {
					1 | 0 => (),
					2 => match game.get_player_turn() as usize {
						0 => T::Elo::update_rating(
							&game.borrow_players()[0],
							&game.borrow_players()[1],
						),
						1 => T::Elo::update_rating(
							&game.borrow_players()[1],
							&game.borrow_players()[0],
						),
						_ => return Err(Error::<T>::InternalError.into()), // Should never happen
					},
					_ => {
						T::Elo::update_ratings(
							&game.borrow_players()[game.get_player_turn() as usize],
							game.borrow_players(),
						);
					},
				};

				game.state = GameState::Finished { winner: Some(game.get_player_turn()) };

				Self::deposit_event(Event::GameFinished { game_id });
			} else {
				// Handle next turn counting
				let player_turn = game.get_player_turn();

				let next_player_turn =
					(player_turn + 1) % game.borrow_players().len().saturated_into::<u8>();

				game.set_player_turn(next_player_turn);

				if next_player_turn == 0 {
					let round = game.get_round() + 1;
					game.set_round(round);

					if round > game.max_rounds {
						game.set_state(GameState::Finished { winner: None });

						Self::deposit_event(Event::GameFinished { game_id });

						return Ok(());
					}
				}

				let next_player = game.borrow_players()[next_player_turn as usize].clone();

				Self::deposit_event(Event::NewTurn { game_id, next_player });
			}

			GameStorage::<T>::set(game_id, Some(game));

			HexBoardStorage::<T>::set(&who, Some(hex_board));

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn force_finish_turn(origin: OriginFor<T>, game_id: GameId) -> DispatchResult {
			let who: AccountIdOf<T> = ensure_signed(origin)?;

			let mut game = match GameStorage::<T>::get(game_id) {
				Some(value) => value,
				None => return Err(Error::<T>::GameNotInitialized.into()),
			};

			ensure!(game.borrow_players().contains(&who), Error::<T>::PlayerNotInGame);

			let current_player = game.borrow_players()[game.get_player_turn() as usize].clone();
			ensure!(current_player != who, Error::<T>::CurrentPlayerCannotForceFinishTurn);

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			ensure!(
				game.last_played_block
					.saturated_into::<u128>()
					.saturating_add(T::BlocksToPlayLimit::get() as u128) <
					current_block_number.saturated_into::<u128>(),
				Error::<T>::BlocksToPlayLimitNotPassed
			);

			game.last_played_block = current_block_number;

			// Handle next turn counting
			let player_turn = game.get_player_turn();

			let next_player_turn =
				(player_turn + 1) % game.borrow_players().len().saturated_into::<u8>();

			game.set_player_turn(next_player_turn);

			if next_player_turn == 0 {
				let round = game.get_round() + 1;
				game.set_round(round);

				if round > game.max_rounds {
					game.set_state(GameState::Finished { winner: None });

					Self::deposit_event(Event::GameFinished { game_id });

					return Ok(());
				}
			}

			let next_player = game.borrow_players()[game.get_player_turn() as usize].clone();

			GameStorage::<T>::set(game_id, Some(game));

			Self::deposit_event(Event::NewTurn { game_id, next_player });

			Self::deposit_event(Event::TurnForceFinished { game_id, player: current_player });

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn receive_reward(_origin: OriginFor<T>) -> DispatchResult {
			todo!()
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn root_delete_game(origin: OriginFor<T>, game_id: GameId) -> DispatchResult {
			ensure_root(origin)?;

			// Ensures that the Game exists
			let game = match GameStorage::<T>::get(game_id) {
				Some(value) => value,
				None => return Err(Error::<T>::GameNotInitialized.into()),
			};

			for player in game.borrow_players() {
				HexBoardStorage::<T>::remove(player);
			}

			GameStorage::<T>::remove(game_id);

			Ok(())
		}

		/*#[pallet::call_index(7)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn root_set_game(origin: OriginFor<T>, game_id: GameId, game: GameOf<T>) -> DispatchResult  {
			ensure_root(origin)?;

			<GameStorage<T>>::set(&game_id, Some(game));

			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn root_set_hex_board(origin: OriginFor<T>, player: AccountIdOf<T>, hex_board: HexBoardOf<T>) -> DispatchResult  {
			ensure_root(origin)?;

			<HexBoardStorage<T>>::set(&player, Some(hex_board));

			Ok(())
		}*/
	}
}

// Other helper methods
impl<T: Config> Pallet<T> {
	/// Instancializes a new Game
	fn do_create_new_game(
		game_id: GameId,
		current_block_number: BlockNumberFor<T>,
		players: Vec<AccountIdOf<T>>,
		grid_size: u8,
	) -> Result<(), sp_runtime::DispatchError> {
		// Default Game Config
		let mut game = Game {
			state: GameState::Playing,
			selection_size: 2,
			round: 0,
			max_rounds: T::MaxRounds::get(),
			player_turn_and_played: 0,
			last_played_block: current_block_number,
			players: players.clone().try_into().map_err(|_| Error::<T>::InternalError)?,
			selection: Default::default(),
		};

		Self::new_selection(&mut game, game_id)?;

		GameStorage::<T>::set(game_id, Some(game));

		Self::deposit_event(Event::GameCreated { game_id, grid_size, players });

		Ok(())
	}

	/// Helper method that generates a completely new selection from the selection_base
	fn new_selection(
		game: &mut GameOf<T>,
		selection_base: GameId,
	) -> Result<(), sp_runtime::DispatchError> {
		// Current random source
		let current_block_number = <frame_system::Pallet<T>>::block_number();

		let mut new_selection: Vec<TileCostIndex> = Default::default();

		let offset = (current_block_number.saturated_into::<u128>() % 32).saturated_into::<u8>();

		for i in 0..game.get_selection_size() {
			new_selection.push(
				selection_base[((i + offset) % 32) as usize] %
					T::TileCosts::get().len().saturated_into::<u8>(),
			);
		}

		// Casting
		game.selection = new_selection.try_into().map_err(|_| Error::<T>::InternalError)?;

		Self::deposit_event(Event::NewTileSelection {
			game_id: selection_base,
			selection: game.selection.clone(),
		});

		Ok(())
	}

	/// Helper method that refills the selection
	fn refill_selection(
		game: &mut GameOf<T>,
		selection_base: GameId,
	) -> Result<(), sp_runtime::DispatchError> {
		let selection_len = game.selection.len();

		if selection_len <= (game.get_selection_size() / 2) as usize {
			if game.get_selection_size() as u32 != T::MaxTileSelection::get() {
				game.set_selection_size(game.get_selection_size().saturating_add(2));
			}

			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let offset =
				(current_block_number.saturated_into::<u128>() % 32).saturated_into::<usize>();

			let mut new_selection = game.selection.to_vec();

			for i in selection_len..game.get_selection_size() as usize {
				new_selection.push(
					selection_base[(i + offset) % 32] %
						T::TileCosts::get().len().saturated_into::<u8>(),
				);
			}

			game.selection = new_selection.try_into().map_err(|_| Error::<T>::InternalError)?;

			Self::deposit_event(Event::SelectionRefilled {
				game_id: selection_base,
				selection: game.selection.clone(),
			});
		}

		Ok(())
	}

	/// Helper method that determines if the user can buy a piece from the active selection
	fn buy_from_selection(
		selection: &mut TileSelectionOf<T>,
		hex_board: &mut HexBoardOf<T>,
		index_to_buy: usize,
	) -> Result<T::Tile, sp_runtime::DispatchError> {
		// Select the offer
		ensure!(selection.len() > index_to_buy, Error::<T>::BuyIndexOutOfBounds);
		let selected_offer_index: TileCostIndex = selection.remove(index_to_buy);

		let all_offers = T::TileCosts::get();

		ensure!(all_offers.len() > selected_offer_index as usize, Error::<T>::BuyIndexOutOfBounds);
		let selected_offer = all_offers[selected_offer_index as usize];

		Self::spend_resource(&selected_offer.cost, hex_board)?;

		Ok(selected_offer.tile_to_buy)
	}

	/// Helper method that determines, how expensive the upgrade for a tile is.
	fn spend_for_tile_upgrade(
		hex_board: &mut HexBoardOf<T>,
		tile_to_upgrade: &T::Tile,
	) -> Result<(), sp_runtime::DispatchError> {
		match (tile_to_upgrade.get_type(), tile_to_upgrade.get_level()) {
			(TileType::Home, tile_level) => {
				Self::spend_resource(
					&ResourceAmount {
						resource_type: ResourceType::Wood,
						amount: (tile_level + 1).saturating_mul(2),
					},
					hex_board,
				)?;
				Self::spend_resource(
					&ResourceAmount {
						resource_type: ResourceType::Stone,
						amount: (tile_level + 1).saturating_mul(2),
					},
					hex_board,
				)?;
				Self::spend_resource(
					&ResourceAmount {
						resource_type: ResourceType::Gold,
						amount: tile_level.saturating_mul(2),
					},
					hex_board,
				)?;
			},
			(TileType::Empty, _) => return Err(Error::<T>::CannotLevelUpEmptyTile.into()),
			_ => return Err(Error::<T>::CannotLevelUp.into()),
		};

		Ok(())
	}

	/// Helper method that spends the resources according to ResourceAmount
	fn spend_resource(
		resource_cost: &ResourceAmount,
		hex_board: &mut HexBoardOf<T>,
	) -> Result<(), sp_runtime::DispatchError> {
		hex_board.resources[resource_cost.resource_type as usize] = hex_board.resources
			[resource_cost.resource_type as usize]
			.checked_sub(resource_cost.amount)
			.ok_or(Error::<T>::NotEnoughResources)?;
		Ok(())
	}

	fn set_patterns(
		hex_board: &mut HexBoardOf<T>,
		tile_coords: (i8, i8),
	) -> Result<(), sp_runtime::DispatchError> {
		let grid_length: usize = hex_board.hex_grid.len();
		let max_distance: i8 = Self::max_distance_from_center(&grid_length);
		let side_length: i8 = Self::side_length(&grid_length);

		let mut impact_tiles: Vec<i8> = vec![Self::coords_to_index(
			&max_distance,
			&side_length,
			&tile_coords.0,
			&tile_coords.1,
		)];

		for neighbour in
			Self::get_neighbouring_tiles(&max_distance, &tile_coords.0, &tile_coords.1)?
				.into_iter()
				.flatten()
		{
			impact_tiles.push(Self::coords_to_index(
				&max_distance,
				&side_length,
				&neighbour.0,
				&neighbour.1,
			));
		}

		for index in impact_tiles {
			Self::set_pattern_around_tile(
				hex_board,
				index.saturated_into(),
				&max_distance,
				&side_length,
			)?;
		}

		Ok(())
	}

	fn set_pattern_around_tile(
		hex_board: &mut HexBoardOf<T>,
		index: u8,
		max_distance: &i8,
		side_length: &i8,
	) -> Result<(), sp_runtime::DispatchError> {
		let tile = hex_board.hex_grid[index as usize];

		if tile.get_pattern() != TilePattern::Normal {
			return Ok(())
		}

		let (q, r) = Self::index_to_coords(index, side_length, max_distance)?;
		let neighbours = Self::get_neighbouring_tiles(max_distance, &q, &r)?;

		let mut n: Vec<Option<(u8, T::Tile)>> = vec![Some((index, tile))];

		for neighbour in neighbours {
			match neighbour {
				Some(value) => {
					let neighbour_index: u8 =
						Self::coords_to_index(max_distance, side_length, &value.0, &value.1)
							.saturated_into();
					n.push(Some((neighbour_index, hex_board.hex_grid[neighbour_index as usize])));
				},
				None => n.push(None),
			}
		}

		if let Some((tile_pattern, indexes)) = Self::get_pattern(n) {
			for i in indexes {
				hex_board.hex_grid[i as usize].set_pattern(tile_pattern);
			}
		}

		Ok(())
	}

	fn get_pattern(n: Vec<Option<(u8, T::Tile)>>) -> Option<(TilePattern, Vec<u8>)> {
		if let Some((_i, tile)) = n[0] {
			if tile.get_type() == TileType::Empty {
				return None
			}
		}

		// Delta
		if let Some(v) = Self::match_same_tile(n[0], n[1], n[2]) {
			return Some((TilePattern::Delta, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[2], n[3]) {
			return Some((TilePattern::Delta, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[3], n[4]) {
			return Some((TilePattern::Delta, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[4], n[5]) {
			return Some((TilePattern::Delta, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[5], n[6]) {
			return Some((TilePattern::Delta, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[6], n[1]) {
			return Some((TilePattern::Delta, v))
		}

		// Line
		if let Some(v) = Self::match_same_tile(n[0], n[1], n[4]) {
			return Some((TilePattern::Line, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[2], n[5]) {
			return Some((TilePattern::Line, v))
		}
		if let Some(v) = Self::match_same_tile(n[0], n[3], n[6]) {
			return Some((TilePattern::Line, v))
		}

		// ypsilon
		if let Some(v) = Self::match_same_tile_4(n[0], n[1], n[3], n[5]) {
			return Some((TilePattern::Line, v))
		}
		if let Some(v) = Self::match_same_tile_4(n[0], n[2], n[4], n[6]) {
			return Some((TilePattern::Line, v))
		}

		None
	}

	fn match_same_tile(
		n1: Option<(u8, T::Tile)>,
		n2: Option<(u8, T::Tile)>,
		n3: Option<(u8, T::Tile)>,
	) -> Option<Vec<u8>> {
		match (n1, n2, n3) {
			(Some((index1, tile1)), Some((index2, tile2)), Some((index3, tile3))) =>
				if tile1.same(&tile2) && tile1.same(&tile3) {
					Some(vec![index1, index2, index3])
				} else {
					None
				},
			_ => None,
		}
	}

	fn match_same_tile_4(
		n1: Option<(u8, T::Tile)>,
		n2: Option<(u8, T::Tile)>,
		n3: Option<(u8, T::Tile)>,
		n4: Option<(u8, T::Tile)>,
	) -> Option<Vec<u8>> {
		match (n1, n2, n3, n4) {
			(
				Some((index1, tile1)),
				Some((index2, tile2)),
				Some((index3, tile3)),
				Some((index4, tile4)),
			) =>
				if tile1.same(&tile2) && tile1.same(&tile3) && tile1.same(&tile4) {
					Some(vec![index1, index2, index3, index4])
				} else {
					None
				},
			_ => None,
		}
	}

	fn is_game_won(hex_board: &HexBoardOf<T>) -> bool {
		hex_board.resources[ResourceType::Human as usize] >= T::TargetGoalHuman::get()
	}

	fn produce(
		hex_board: &mut HexBoardOf<T>,
		resource_productions: &ResourceProductions,
		multiplier: u8,
	) {
		for resource_type_index in 0..NUMBER_OF_RESOURCE_TYPES {
			match (
				resource_productions.produces[resource_type_index],
				resource_productions.human_requirements[resource_type_index],
			) {
				(0, _) => (),
				(produces, 0) =>
					hex_board.resources[resource_type_index] = Self::saturate_at_99(
						hex_board.resources[resource_type_index]
							.saturating_add(produces.saturating_mul(multiplier)),
					),
				(produces, human_requirement) =>
					hex_board.resources[resource_type_index] = Self::saturate_at_99(
						hex_board.resources[resource_type_index].saturating_add(cmp::min(
							produces.saturating_mul(multiplier),
							hex_board.resources[ResourceType::Human as usize] / human_requirement,
						)),
					),
			}
		}
	}

	fn evaluate_board(hex_board: &mut HexBoardOf<T>) {
		let board_stats: BoardStats = hex_board.get_stats();

		hex_board.resources[ResourceType::Mana as usize] = Self::saturate_at_99(
			hex_board.resources[ResourceType::Mana as usize]
				.saturating_add(hex_board.resources[ResourceType::Human as usize] / 3)
				.saturating_add(board_stats.get_tiles(TileType::Home)),
		);

		let food_and_water_eaten = cmp::min(
			hex_board.resources[ResourceType::Food as usize].saturating_mul(T::FoodPerHuman::get()),
			hex_board.resources[ResourceType::Water as usize]
				.saturating_mul(T::WaterPerHuman::get()),
		);

		let mut home_weighted: u8 = 0;

		for level in 0..NUMBER_OF_LEVELS {
			home_weighted = home_weighted.saturating_add(
				(level as u8 + 1u8).saturating_mul(board_stats.get_levels(TileType::Home, level)),
			);
		}

		let new_humans = Self::saturate_at_99(cmp::max(
			cmp::min(
				board_stats.get_tiles(TileType::Home).saturating_add(food_and_water_eaten),
				home_weighted.saturating_mul(T::HomePerHumans::get()),
			),
			1,
		));

		for (tile_type_index, resource_productions) in
			T::TileResourceProductions::get().iter().enumerate().take(NUMBER_OF_TILE_TYPES)
		{
			Self::produce(
				hex_board,
				resource_productions,
				board_stats.get_tiles_by_tile_index(tile_type_index),
			);
		}

		hex_board.resources[ResourceType::Human as usize] = new_humans;
	}

	/// Check if the hexagon at (q, r) is within the valid bounds of the grid
	fn is_valid_hex(max_distance: &i8, q: &i8, r: &i8) -> bool {
		&q.abs() <= max_distance && &r.abs() <= max_distance
	}

	/// Check if at least one of the neighbouring tiles is not Empty.
	fn not_surrounded_by_empty_tiles(
		neighbours: &Vec<Option<(i8, i8)>>,
		hex_grid: &HexGridOf<T>,
		max_distance: &i8,
		side_length: &i8,
	) -> bool {
		for neighbour in neighbours {
			match neighbour {
				Some((q, r)) =>
					if hex_grid[Self::coords_to_index(max_distance, side_length, q, r) as usize]
						.get_type() != TileType::Empty
					{
						return true
					},
				None => (),
			};
		}
		false
	}

	/// Get the neighbors of a hex tile in the grid
	fn get_neighbouring_tiles(
		max_distance: &i8,
		q: &i8,
		r: &i8,
	) -> Result<Vec<Option<(i8, i8)>>, sp_runtime::DispatchError> {
		let mut neigbouring_tiles: Vec<Option<(i8, i8)>> = Default::default();

		let directions = [(0, -1), (1, -1), (1, 0), (0, 1), (-1, 1), (-1, 0)];

		for (q_direction, r_direction) in directions {
			let neighbour_q = q.checked_add(q_direction).ok_or(Error::<T>::MathOverflow)?;
			let neighbout_r = r.checked_add(r_direction).ok_or(Error::<T>::MathOverflow)?;

			if Self::is_valid_hex(max_distance, &neighbour_q, &neighbout_r) {
				neigbouring_tiles.push(Some((neighbour_q, neighbout_r)));
			} else {
				neigbouring_tiles.push(None)
			}
		}

		Ok(neigbouring_tiles)
	}

	fn coords_to_index(max_distance: &i8, side_length: &i8, q: &i8, r: &i8) -> i8 {
		q + max_distance + (r + max_distance) * side_length
	}

	fn index_to_coords(
		index: u8,
		side_length: &i8,
		max_distance: &i8,
	) -> Result<(i8, i8), sp_runtime::DispatchError> {
		let index_i8: i8 = index.try_into().map_err(|_| Error::<T>::InternalError)?;
		let q: i8 = (index_i8 % side_length) - max_distance;
		let r: i8 = index_i8 / side_length - (side_length - 1) / 2;
		Ok((q, r))
	}

	/// Fast helper method that quickly computes the max_distance for the size of the board
	fn max_distance_from_center(hex_grid_len: &usize) -> i8 {
		// (sqrt(hex_grid_len) - 1) / 2
		match hex_grid_len {
			9 => 1,
			25 => 2,
			49 => 3,
			_ => 0,
		}
	}

	/// Fast helper method that quickly computes the side_length for the size of the board
	fn side_length(hex_grid_len: &usize) -> i8 {
		// (sqrt(hex_grid_len)
		match hex_grid_len {
			9 => 3,
			25 => 5,
			49 => 7,
			_ => 0,
		}
	}

	/// Helper method that tells you if the board size is valid
	fn is_valid_grid_size(size: u8) -> bool {
		matches!(size, 9 | 25 | 49)
	}

	fn saturate_at_99(x: u8) -> u8 {
		cmp::min(x, 99)
	}

	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	pub fn set_hex_board(player: AccountIdOf<T>, hex_board: HexBoardOf<T>) {
		<HexBoardStorage<T>>::set(&player, Some(hex_board));
	}

	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	pub fn set_game(game_id: GameId, game: GameOf<T>) {
		<GameStorage<T>>::set(game_id, Some(game));
	}
}
