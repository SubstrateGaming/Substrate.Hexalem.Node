use crate::{mock::*, types::*, Event, *};
use frame_support::{assert_noop, assert_ok};
use pallet_elo::Event as EloEvent;

#[test]
fn game_loop() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let players = vec![1, 2, 3];

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), players.clone(), 25));
		// Read pallet storage and assert an expected result.
		let hex_board_option: Option<crate::HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(
			hex_board.resources,
			<mock::TestRuntime as pallet::Config>::DefaultPlayerResources::get()
		);

		let default_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();
		assert_eq!(hex_board.hex_grid, default_hex_grid);

		let game_id: GameId = hex_board.get_game_id().unwrap();

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::GameCreated { game_id, grid_size: 25, players: players.clone() }.into(),
		);

		let game_option = GameStorage::<TestRuntime>::get(game_id);

		let game = game_option.unwrap();

		assert_eq!(game.players, players.clone());

		assert_eq!(game.get_player_turn(), 0);

		assert!(!game.get_played());

		assert_eq!(game.get_round(), 0);

		assert_eq!(game.get_selection_size(), 2);

		assert_eq!(game.get_state(), GameState::Playing);

		let current_selection_indexes = game.selection.clone();

		let selection_one_cost = <mock::TestRuntime as pallet::Config>::TileCosts::get()
			[current_selection_indexes[0] as usize];

		let move_played = Move { place_index: 11, buy_index: 0 };

		assert_eq!(selection_one_cost.cost.resource_type, ResourceType::Mana);
		assert_eq!(selection_one_cost.cost.amount, 1);

		assert_ok!(HexalemModule::play(RuntimeOrigin::signed(1), move_played.clone()));

		System::assert_last_event(Event::MovePlayed { game_id, player: 1, move_played }.into());

		let hex_board_option: Option<crate::HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();
		assert_eq!(hex_board.resources, [0, 1, 0, 0, 0, 0, 0]);

		let expected_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			selection_one_cost.tile_to_buy,
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();
		assert_eq!(hex_board.hex_grid, expected_hex_grid);

		let game_option = GameStorage::<TestRuntime>::get(game_id);

		let game = game_option.unwrap();

		assert_eq!(game.players, players.clone());

		assert_eq!(game.get_player_turn(), 0);

		assert!(game.get_played());

		assert_eq!(game.get_round(), 0);

		assert_eq!(game.get_selection_size(), 4);

		assert_eq!(game.get_state(), GameState::Playing);

		assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));

		System::assert_last_event(Event::NewTurn { game_id, next_player: 2 }.into());

		let hex_board_option: Option<crate::HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();
		assert_eq!(hex_board.resources, [1, 1, 2, 0, 0, 0, 0]);

		assert_eq!(hex_board.hex_grid, expected_hex_grid);

		let game_option = GameStorage::<TestRuntime>::get(game_id);

		let game = game_option.unwrap();

		assert_eq!(game.players, players.clone());

		assert_eq!(game.get_player_turn(), 1);

		assert!(!game.get_played());

		assert_eq!(game.get_round(), 0);

		assert_eq!(game.get_selection_size(), 4);

		assert_eq!(game.get_state(), GameState::Playing);
	});
}

#[test]
fn create_game() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(1), vec![], 25),
			Error::<TestRuntime>::NumberOfPlayersIsTooSmall
		);

		assert_noop!(
			HexalemModule::create_game(
				RuntimeOrigin::signed(1),
				(1..=101).collect::<Vec<u64>>(),
				25
			),
			Error::<TestRuntime>::NumberOfPlayersIsTooLarge
		);

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 1),
			Error::<TestRuntime>::BadGridSize
		);

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 2),
			Error::<TestRuntime>::BadGridSize
		);

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 20),
			Error::<TestRuntime>::BadGridSize
		);

		assert_ok!(HexalemModule::create_game(
			RuntimeOrigin::signed(1),
			(1..=100).collect::<Vec<u64>>(),
			25
		));

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 25),
			Error::<TestRuntime>::GameAlreadyCreated
		);

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(2), vec![2, 1], 25),
			Error::<TestRuntime>::AlreadyPlaying
		);

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(101), vec![101], 9));

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(102), vec![102], 49));

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(103), vec![104], 9),
			Error::<TestRuntime>::CreatorNotInPlayersAtIndexZero,
		);

		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(103), vec![104, 103], 9),
			Error::<TestRuntime>::CreatorNotInPlayersAtIndexZero,
		);

		// Error when the same player is included twice
		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(105), vec![105, 105], 25),
			Error::<TestRuntime>::AlreadyPlaying
		);

		// Error when the same player is included twice
		assert_noop!(
			HexalemModule::create_game(RuntimeOrigin::signed(105), vec![105, 106, 106, 107], 25),
			Error::<TestRuntime>::AlreadyPlaying
		);
	});
}

#[test]
fn test_resource_generation() {
	new_test_ext().execute_with(|| {
		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 25));

		let new_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(56),
			HexalemTile(48),
			HexalemTile(40),
			HexalemTile(32),
			HexalemTile(24),
			HexalemTile(16),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		HexalemModule::set_hex_board(
			1,
			HexBoard {
				matchmaking_state: MatchmakingState::Joined(game_id),
				hex_grid: new_hex_grid,
				resources: [0, 1, 0, 0, 0, 0, 0],
			},
		);

		assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.resources, [1, 1, 2, 3, 1, 1, 0]);
	});
}

#[test]
fn test_saturate_99() {
	new_test_ext().execute_with(|| {
		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 25));

		let new_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(56),
			HexalemTile(48),
			HexalemTile(40),
			HexalemTile(32),
			HexalemTile(24),
			HexalemTile(16),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		// Set player resources to 99 and set a new hex_grid
		HexalemModule::set_hex_board(
			1,
			HexBoard {
				matchmaking_state: MatchmakingState::Joined(game_id),
				hex_grid: new_hex_grid,
				resources: [99; NUMBER_OF_RESOURCE_TYPES],
			},
		);

		assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.resources, [99, 3, 99, 99, 99, 99, 99]);
	});
}

#[test]
fn test_game_finishes_on_25th_round() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1], 25));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		for _ in 0..<mock::TestRuntime as pallet::Config>::MaxRounds::get() {
			assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));
		}

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.matchmaking_state, MatchmakingState::Finished(Rewards::Draw));

		System::assert_has_event(Event::GameFinished { game_id }.into());

		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(1)),
			Error::<TestRuntime>::HexBoardNotInPlayingState,
		);
	});
}

#[test]
fn test_game_finishes_on_25th_round_3p() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1, 2, 3], 25));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(2);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		for _ in 0..<mock::TestRuntime as pallet::Config>::MaxRounds::get() {
			assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));
			assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(2)));
			assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(3)));
		}

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(2);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.matchmaking_state, MatchmakingState::Finished(Rewards::Draw));

		System::assert_has_event(Event::GameFinished { game_id }.into());

		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(1)),
			Error::<TestRuntime>::HexBoardNotInPlayingState
		);
		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(2)),
			Error::<TestRuntime>::HexBoardNotInPlayingState
		);
		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(3)),
			Error::<TestRuntime>::HexBoardNotInPlayingState
		);
	});
}

#[test]
fn test_game_force_finishes_on_25th_round_3p() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1, 2, 3], 25));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(2);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		for _ in 0..<mock::TestRuntime as pallet::Config>::MaxRounds::get() {
			System::set_block_number(
				<mock::TestRuntime as pallet::Config>::BlocksToPlayLimit::get() as u64 + System::block_number() + 1,
			);
			assert_ok!(HexalemModule::force_finish_turn(RuntimeOrigin::signed(2), game_id));

			System::set_block_number(
				<mock::TestRuntime as pallet::Config>::BlocksToPlayLimit::get() as u64 + System::block_number() + 1,
			);
			assert_ok!(HexalemModule::force_finish_turn(RuntimeOrigin::signed(3), game_id));

			System::set_block_number(
				<mock::TestRuntime as pallet::Config>::BlocksToPlayLimit::get() as u64 + System::block_number() + 1,
			);
			assert_ok!(HexalemModule::force_finish_turn(RuntimeOrigin::signed(1), game_id));
		}

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(2);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.matchmaking_state, MatchmakingState::Finished(Rewards::Draw));

		System::assert_has_event(Event::GameFinished { game_id }.into());

		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(1)),
			Error::<TestRuntime>::HexBoardNotInPlayingState
		);
		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(2)),
			Error::<TestRuntime>::HexBoardNotInPlayingState
		);
		assert_noop!(
			HexalemModule::finish_turn(RuntimeOrigin::signed(3)),
			Error::<TestRuntime>::HexBoardNotInPlayingState
		);
	});
}

#[test]
fn test_force_finish_turn() {
	new_test_ext().execute_with(|| {
		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1, 2], 25));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		// force_finish_turn can not be called before the BlocksToPlayLimit has been passed
		assert_noop!(
			HexalemModule::force_finish_turn(RuntimeOrigin::signed(2), game_id),
			Error::<TestRuntime>::BlocksToPlayLimitNotPassed
		);

		System::set_block_number(
			<mock::TestRuntime as pallet::Config>::BlocksToPlayLimit::get() as u64 + 1,
		);

		// force_finish_turn can not be called by the player that is currently on turn
		assert_noop!(
			HexalemModule::force_finish_turn(RuntimeOrigin::signed(1), game_id),
			Error::<TestRuntime>::CurrentPlayerCannotForceFinishTurn
		);

		// force_finish_turn can not be called by the player that is not in the game
		assert_noop!(
			HexalemModule::force_finish_turn(RuntimeOrigin::signed(3), game_id),
			Error::<TestRuntime>::PlayerNotInGame
		);

		// Now that enough blocks have passed, force_finish_turn can be called
		assert_ok!(HexalemModule::force_finish_turn(RuntimeOrigin::signed(2), game_id));
	})
}

#[test]
fn play() {
	new_test_ext().execute_with(|| {
		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1, 2], 25));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(1), Move { place_index: 12, buy_index: 0 }),
			Error::<TestRuntime>::TileIsNotEmpty
		);

		// newly placed tile needs to connect to already placed tiles
		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(1), Move { place_index: 0, buy_index: 0 }),
			Error::<TestRuntime>::TileSurroundedByEmptyTiles
		);

		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(1), Move { place_index: 26, buy_index: 0 }),
			Error::<TestRuntime>::PlaceIndexOutOfBounds
		);

		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(1), Move { place_index: 11, buy_index: 2 }),
			Error::<TestRuntime>::BuyIndexOutOfBounds
		);

		// Set player resources to 0
		HexalemModule::set_hex_board(
			1,
			HexBoard {
				matchmaking_state: MatchmakingState::Joined(game_id),
				hex_grid: hex_board.hex_grid,
				resources: [0; NUMBER_OF_RESOURCE_TYPES],
			},
		);

		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(1), Move { place_index: 11, buy_index: 0 }),
			Error::<TestRuntime>::NotEnoughResources
		);

		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(2), Move { place_index: 11, buy_index: 0 }),
			Error::<TestRuntime>::PlayerNotOnTurn
		);

		assert_noop!(
			HexalemModule::play(RuntimeOrigin::signed(3), Move { place_index: 11, buy_index: 0 }),
			Error::<TestRuntime>::HexBoardNotInitialized
		);
	})
}

#[test]
fn play_pattern() {
	new_test_ext().execute_with(|| {
		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1, 2], 25));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let new_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(24),
			HexalemTile(24),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(56),
			HexalemTile(0),
			HexalemTile(24),
			HexalemTile(0),
			HexalemTile(56),
			HexalemTile(0),
			HexalemTile(24),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		// Set player resources to 0
		HexalemModule::set_hex_board(
			1,
			HexBoard {
				matchmaking_state: MatchmakingState::Joined(game_id),
				hex_grid: new_hex_grid.clone(),
				resources: [5; NUMBER_OF_RESOURCE_TYPES],
			},
		);

		let game_option = GameStorage::<TestRuntime>::get(game_id);

		let game = game_option.unwrap();

		assert_eq!(
			<mock::TestRuntime as pallet::Config>::TileCosts::get()[game.selection[1] as usize]
				.tile_to_buy,
			HexalemTile(56)
		);

		assert_ok!(HexalemModule::play(
			RuntimeOrigin::signed(1),
			Move { place_index: 21, buy_index: 0 }
		));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		//assert_eq!(hex_board.hex_grid, new_hex_grid);

		assert_eq!(hex_board.hex_grid[16].get_type(), TileType::Cave);
		assert_eq!(hex_board.hex_grid[20].get_type(), TileType::Cave);
		assert_eq!(hex_board.hex_grid[21].get_type(), TileType::Cave);
		assert_eq!(hex_board.hex_grid[16].get_pattern(), TilePattern::Delta);
		assert_eq!(hex_board.hex_grid[20].get_pattern(), TilePattern::Delta);
		assert_eq!(hex_board.hex_grid[21].get_pattern(), TilePattern::Delta);

		let game_option = GameStorage::<TestRuntime>::get(game_id);

		let game = game_option.unwrap();

		assert_eq!(
			<mock::TestRuntime as pallet::Config>::TileCosts::get()[game.selection[2] as usize]
				.tile_to_buy,
			HexalemTile(24)
		);

		assert_eq!(
			<mock::TestRuntime as pallet::Config>::TileCosts::get()[game.selection[3] as usize]
				.tile_to_buy,
			HexalemTile(24)
		);

		assert_ok!(HexalemModule::play(
			RuntimeOrigin::signed(1),
			Move { place_index: 8, buy_index: 2 }
		));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.hex_grid[6].get_type(), TileType::Water);
		assert_eq!(hex_board.hex_grid[7].get_type(), TileType::Water);
		assert_eq!(hex_board.hex_grid[8].get_type(), TileType::Water);
		assert_eq!(hex_board.hex_grid[6].get_pattern(), TilePattern::Line);
		assert_eq!(hex_board.hex_grid[7].get_pattern(), TilePattern::Line);
		assert_eq!(hex_board.hex_grid[8].get_pattern(), TilePattern::Line);

		assert_ok!(HexalemModule::play(
			RuntimeOrigin::signed(1),
			Move { place_index: 17, buy_index: 2 }
		));

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(hex_board.hex_grid[18].get_type(), TileType::Water);
		assert_eq!(hex_board.hex_grid[17].get_type(), TileType::Water);
		assert_eq!(hex_board.hex_grid[22].get_type(), TileType::Water);
		assert_eq!(hex_board.hex_grid[22].get_pattern(), TilePattern::Delta);
		assert_eq!(hex_board.hex_grid[18].get_pattern(), TilePattern::Delta);
		assert_eq!(hex_board.hex_grid[17].get_pattern(), TilePattern::Delta);
	});
}

#[test]
fn upgrade() {
	new_test_ext().execute_with(|| {
		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), vec![1, 2], 25));

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 12),
			Error::<TestRuntime>::NotEnoughResources
		);

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		let new_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(56),
			HexalemTile(48),
			HexalemTile(40),
			HexalemTile(32),
			HexalemTile(24),
			HexalemTile(16),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();

		HexalemModule::set_hex_board(
			1,
			HexBoard {
				matchmaking_state: MatchmakingState::Joined(game_id),
				hex_grid: new_hex_grid,
				resources: [10; NUMBER_OF_RESOURCE_TYPES],
			},
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 0),
			Error::<TestRuntime>::CannotLevelUpEmptyTile
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 11),
			Error::<TestRuntime>::CannotLevelUp
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 10),
			Error::<TestRuntime>::CannotLevelUp
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 9),
			Error::<TestRuntime>::CannotLevelUp
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 100),
			Error::<TestRuntime>::PlaceIndexOutOfBounds
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(2), 12),
			Error::<TestRuntime>::PlayerNotOnTurn
		);

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(3), 12),
			Error::<TestRuntime>::HexBoardNotInitialized
		);

		let upgrade_costs: [[ResourceUnit; NUMBER_OF_RESOURCE_TYPES]; NUMBER_OF_LEVELS - 1] =
			[[0, 0, 0, 0, 2, 2, 0], [0, 0, 0, 0, 4, 4, 2], [0, 0, 0, 0, 6, 6, 4]];

		for (level, upgrade_costs_for_level) in
			upgrade_costs.iter().enumerate().take(NUMBER_OF_LEVELS - 1)
		{
			assert_ok!(HexalemModule::upgrade(RuntimeOrigin::signed(1), 12));

			let hex_board_option: Option<HexBoardOf<TestRuntime>> =
				HexBoardStorage::<TestRuntime>::get(1);

			let hex_board = hex_board_option.unwrap();

			let mut resources_expected = [10u8; NUMBER_OF_RESOURCE_TYPES];
			for (resource_type, upgrade_cost) in
				upgrade_costs_for_level.iter().enumerate().take(NUMBER_OF_RESOURCE_TYPES)
			{
				resources_expected[resource_type] -= upgrade_cost;
			}

			assert_eq!(
				hex_board.resources,
				resources_expected, // Once the constant is set, change the values
			);

			assert_eq!(hex_board.hex_grid[12].get_level(), (level as u8) + 1);

			// Refresh resources, so that the player has got enough of them to pay for the upgrade
			HexalemModule::set_hex_board(
				1,
				HexBoard {
					matchmaking_state: MatchmakingState::Joined(game_id),
					hex_grid: hex_board.hex_grid,
					resources: [10; NUMBER_OF_RESOURCE_TYPES],
				},
			);
		}

		assert_noop!(
			HexalemModule::upgrade(RuntimeOrigin::signed(1), 12),
			Error::<TestRuntime>::TileOnMaxLevel
		);
	})
}

#[test]
fn coords_to_sindex() {
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &0, &0), 12);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &-2, &-2), 0);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &2, &2), 24);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &2, &-2), 4);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &-2, &2), 20);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &1, &0), 13);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &2, &0), 14);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &2, &1), 19);
	assert_eq!(crate::Pallet::<TestRuntime>::coords_to_index(&2, &5, &-1, &1), 16);
}

#[test]
fn get_neighbouring_tiles() {
	assert_eq!(
		crate::Pallet::<TestRuntime>::get_neighbouring_tiles(&2, &0, &0),
		Ok(vec![
			Some((0, -1)),
			Some((1, -1)),
			Some((1, 0)),
			Some((0, 1)),
			Some((-1, 1)),
			Some((-1, 0))
		])
	);

	assert_eq!(
		crate::Pallet::<TestRuntime>::get_neighbouring_tiles(&2, &-2, &-2),
		Ok(vec![None, None, Some((-1, -2)), Some((-2, -1)), None, None])
	);

	assert_eq!(
		crate::Pallet::<TestRuntime>::get_neighbouring_tiles(&2, &-2, &2),
		Ok(vec![Some((-2, 1)), Some((-1, 1)), Some((-1, 2)), None, None, None])
	);
}

#[test]
fn is_valid_hex() {
	assert!(crate::Pallet::<TestRuntime>::is_valid_hex(&2, &0, &0));

	assert!(crate::Pallet::<TestRuntime>::is_valid_hex(&2, &1, &-1));

	assert!(crate::Pallet::<TestRuntime>::is_valid_hex(&2, &-2, &-2));

	assert!(crate::Pallet::<TestRuntime>::is_valid_hex(&2, &2, &-2));

	assert!(!crate::Pallet::<TestRuntime>::is_valid_hex(&2, &-3, &2));

	assert!(!crate::Pallet::<TestRuntime>::is_valid_hex(&2, &-3, &-2));

	assert!(!crate::Pallet::<TestRuntime>::is_valid_hex(&2, &-1, &-3));

	assert!(!crate::Pallet::<TestRuntime>::is_valid_hex(&2, &-1, &10));
}

#[test]
fn index_to_coords() {
	assert_eq!(crate::Pallet::<TestRuntime>::index_to_coords(0, &5, &2), Ok((-2, -2)));

	assert_eq!(crate::Pallet::<TestRuntime>::index_to_coords(12, &5, &2), Ok((0, 0)));
}

#[test]
fn match_tiles() {
	assert_eq!(crate::Pallet::<TestRuntime>::match_same_tile(None, None, None), None);

	assert_eq!(
		crate::Pallet::<TestRuntime>::match_same_tile(
			Some((16, HexalemTile(56))),
			Some((20, HexalemTile(56))),
			Some((21, HexalemTile(56)))
		),
		Some(vec![16, 20, 21])
	);

	assert_eq!(
		crate::Pallet::<TestRuntime>::match_same_tile(
			Some((11, HexalemTile(16))),
			Some((10, HexalemTile(16))),
			Some((15, HexalemTile(16)))
		),
		Some(vec![11, 10, 15])
	);
}

#[test]
fn tiles() {
	assert_eq!(HexalemTile(0).get_type(), TileType::Empty);

	assert_eq!(HexalemTile(24).get_type(), TileType::Water);

	assert_eq!(HexalemTile(56).get_type(), TileType::Cave);
}

#[test]
fn simple_2p_matchmaking() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(HexalemModule::queue(RuntimeOrigin::signed(1)));

		let hex_board_option: Option<crate::HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(
			hex_board.resources,
			<mock::TestRuntime as pallet::Config>::DefaultPlayerResources::get()
		);

		let default_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();
		assert_eq!(hex_board.hex_grid, default_hex_grid);

		assert_eq!(hex_board.matchmaking_state, MatchmakingState::Matchmaking);

		assert_eq!(MatchmakerModule::queue_size(0), 1);

		assert_ok!(HexalemModule::queue(RuntimeOrigin::signed(2)));

		let hex_board_option: Option<crate::HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(2);

		let hex_board = hex_board_option.unwrap();

		assert_eq!(
			hex_board.resources,
			<mock::TestRuntime as pallet::Config>::DefaultPlayerResources::get()
		);

		let default_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile::get_home(),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();
		assert_eq!(hex_board.hex_grid, default_hex_grid);

		assert_ne!(hex_board.matchmaking_state, MatchmakingState::Matchmaking);

		assert_eq!(MatchmakerModule::queue_size(0), 0);

		let game_id: GameId = hex_board.get_game_id().unwrap();

		let game_option = GameStorage::<TestRuntime>::get(game_id);

		let game = game_option.unwrap();

		assert_eq!(game.players, vec![1, 2]);

		assert_eq!(game.get_player_turn(), 0);

		assert!(!game.get_played());

		assert_eq!(game.get_round(), 0);

		assert_eq!(game.get_selection_size(), 2);

		assert_eq!(game.get_state(), GameState::Playing);

		assert_noop!(
			HexalemModule::queue(RuntimeOrigin::signed(1)),
			Error::<TestRuntime>::AlreadyPlaying
		);

		assert_noop!(
			HexalemModule::queue(RuntimeOrigin::signed(2)),
			Error::<TestRuntime>::AlreadyPlaying
		);
	});
}

#[test]
fn queue_edgecases() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		assert_ok!(HexalemModule::queue(RuntimeOrigin::signed(1)));

		assert_noop!(
			HexalemModule::queue(RuntimeOrigin::signed(1)),
			Error::<TestRuntime>::AlreadyPlaying
		);
	});
}

#[test]
fn elo_2p_match() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let players = vec![1, 2];

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), players.clone(), 25));

		let new_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile::new(TileType::Home, 3, TilePattern::Normal),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let mut hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		hex_board.resources = [99; NUMBER_OF_RESOURCE_TYPES];

		hex_board.hex_grid = new_hex_grid;

		// Set player resources to 99 and set a new hex_grid
		HexalemModule::set_hex_board(1, hex_board);

		assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));

		System::assert_has_event(Event::GameFinished { game_id }.into());

		System::assert_has_event(
			EloEvent::RatingGained { player: 1, new_rating: 1016, rating_gained: 16 }.into(),
		);

		System::assert_has_event(
			EloEvent::RatingLost { player: 2, new_rating: 984, rating_lost: 16 }.into(),
		);

		assert_eq!(EloModule::get_rating(1), 1016);
		assert_eq!(EloModule::get_rating(2), 984);
	});
}

#[test]
fn elo_4p_match() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let players = vec![1, 2, 3, 4];

		assert_ok!(HexalemModule::create_game(RuntimeOrigin::signed(1), players.clone(), 25));

		let new_hex_grid: HexGridOf<TestRuntime> = vec![
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile::new(TileType::Home, 3, TilePattern::Normal),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
			HexalemTile(0),
		]
		.try_into()
		.unwrap();

		let hex_board_option: Option<HexBoardOf<TestRuntime>> =
			HexBoardStorage::<TestRuntime>::get(1);

		let mut hex_board = hex_board_option.unwrap();

		let game_id: GameId = hex_board.get_game_id().unwrap();

		hex_board.resources = [99; NUMBER_OF_RESOURCE_TYPES];

		hex_board.hex_grid = new_hex_grid;

		// Set player resources to 99 and set a new hex_grid
		HexalemModule::set_hex_board(1, hex_board);

		assert_ok!(HexalemModule::finish_turn(RuntimeOrigin::signed(1)));

		System::assert_has_event(Event::GameFinished { game_id }.into());
		System::assert_has_event(
			EloEvent::RatingGained { player: 1, new_rating: 1048, rating_gained: 48 }.into(),
		);
		System::assert_has_event(
			EloEvent::RatingLost { player: 2, new_rating: 984, rating_lost: 16 }.into(),
		);
		System::assert_has_event(
			EloEvent::RatingLost { player: 3, new_rating: 984, rating_lost: 16 }.into(),
		);
		System::assert_has_event(
			EloEvent::RatingLost { player: 4, new_rating: 984, rating_lost: 16 }.into(),
		);

		assert_eq!(EloModule::get_rating(1), 1048);
		assert_eq!(EloModule::get_rating(2), 984);
		assert_eq!(EloModule::get_rating(3), 984);
		assert_eq!(EloModule::get_rating(4), 984);
	});
}

#[test]
fn clean_hex_board_storage() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		HexBoardStorage::<TestRuntime>::set(
			1,
			Some(
				HexBoardOf::<TestRuntime>::try_new::<<mock::TestRuntime as pallet::Config>::DefaultPlayerResources>(
					25,
					MatchmakingState::Matchmaking,
				)
				.unwrap(),
			),
		);

		assert_noop!(HexalemModule::receive_rewards(RuntimeOrigin::signed(1)), Error::<TestRuntime>::HexBoardNotInFinishedState);

		HexBoardStorage::<TestRuntime>::set(
			2,
			Some(
				HexBoardOf::<TestRuntime>::try_new::<<mock::TestRuntime as pallet::Config>::DefaultPlayerResources>(
					25,
					MatchmakingState::Joined(Default::default()),
				)
				.unwrap(),
			),
		);

		assert_noop!(HexalemModule::receive_rewards(RuntimeOrigin::signed(2)), Error::<TestRuntime>::HexBoardNotInFinishedState);

		HexBoardStorage::<TestRuntime>::set(
			3,
			Some(
				HexBoardOf::<TestRuntime>::try_new::<<mock::TestRuntime as pallet::Config>::DefaultPlayerResources>(
					25,
					MatchmakingState::Finished(Rewards::Winner),
				)
				.unwrap(),
			),
		);

		assert_ok!(HexalemModule::receive_rewards(RuntimeOrigin::signed(3)));
	});
}