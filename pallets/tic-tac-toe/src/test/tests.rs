use crate::{test::mock::*, *};
use frame_support::{assert_noop, assert_ok};

mod create {
	use super::*;

	#[test]
	fn can_create_game() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			run_to_block(2);

			let game_id = 0;

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(game_id));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::GameCreated {
				game_id,
				by: ALICE,
			}));
		});
	}

	#[test]
	fn different_accounts_can_create_different_games() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(BOB));
			assert!(PendingGames::<Test>::contains_key(1));
		});
	}

	#[test]
	fn cannot_create_mutliple_games_with_signle_account() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_noop!(
				TicTacToe::create_game(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::CannotCreateNewGame
			);
		});
	}
}

mod join {
	use super::*;

	#[test]
	fn can_join_game() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			let game_id = 0;

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(game_id));

			run_to_block(2);

			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::GameJoined {
				game_id,
				by: BOB,
			}));

			assert!(ActivePlayers::<Test>::contains_key(BOB));

			assert!(!PendingGames::<Test>::contains_key(0));
			assert!(ActiveGames::<Test>::contains_key(0));
		});
	}

	#[test]
	fn cannot_join_game_if_host() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(BOB));
			assert!(PendingGames::<Test>::contains_key(1));

			assert_noop!(
				TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)),
				Error::<Test>::CannotJoinAnotherGame
			);
		});
	}

	#[test]
	fn cannot_join_own_game() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(PendingGames::<Test>::contains_key(0));

			run_to_block(2);

			assert_noop!(
				TicTacToe::join_pending_game(RuntimeOrigin::signed(ALICE)),
				Error::<Test>::CannotJoinAnotherGame
			);
		});
	}

	#[test]
	fn cannot_join_started_game() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			run_to_block(2);

			assert_noop!(
				TicTacToe::join_pending_game(RuntimeOrigin::signed(CHARLIE)),
				Error::<Test>::NoPendingGamesFound
			);
		});
	}
}

mod play {
	use super::*;

	#[test]
	fn can_play_game() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(ActivePlayers::<Test>::contains_key(BOB));

			assert!(ActiveGames::<Test>::contains_key(0));

			run_to_block(2);

			let coords = Coordinates::from((0, 0));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(ALICE), coords));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::MovePlayed {
				at: coords,
				by: ALICE,
			}));

			let game = ActiveGames::<Test>::get(0).unwrap();
			assert_eq!(
				game.cells,
				[
					[CellState::Circle, CellState::Empty, CellState::Empty],
					[CellState::Empty; 3],
					[CellState::Empty; 3]
				]
			);
			assert_eq!(game.state, BoardState::Playing(ALICE, BOB, CurrentTurn::PlayerTwo));
			assert_eq!(game.moves_played, 1);

			run_to_block(5);

			let coords = Coordinates::from((1, 2));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), coords));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::MovePlayed {
				at: coords,
				by: BOB,
			}));

			let game = ActiveGames::<Test>::get(0).unwrap();
			assert_eq!(
				game.cells,
				[
					[CellState::Circle, CellState::Empty, CellState::Empty],
					[CellState::Empty; 3],
					[CellState::Empty, CellState::Cross, CellState::Empty]
				]
			);
			assert_eq!(game.state, BoardState::Playing(ALICE, BOB, CurrentTurn::PlayerOne));
			assert_eq!(game.moves_played, 2);
		});
	}

	#[test]
	fn can_win_game_horizontal() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(ActivePlayers::<Test>::contains_key(BOB));

			let game_id = 0;

			assert!(ActiveGames::<Test>::contains_key(game_id));

			run_to_block(2);

			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((0, 0))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((0, 2))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((1, 0))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((0, 1))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((2, 0))
			));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::GameFinished {
				game_id,
				winner: Some(ALICE),
			}));

			assert!(!ActivePlayers::<Test>::contains_key(ALICE));
			assert!(!ActivePlayers::<Test>::contains_key(BOB));
			assert!(!PendingGames::<Test>::contains_key(game_id));
			assert!(!ActiveGames::<Test>::contains_key(game_id));
		});
	}

	#[test]
	fn can_win_game_diagonal() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(CHARLIE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(CHARLIE));
			assert!(ActivePlayers::<Test>::contains_key(BOB));

			let game_id = 0;

			assert!(ActiveGames::<Test>::contains_key(game_id));

			run_to_block(2);

			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(CHARLIE),
				Coordinates::from((2, 0))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((0, 0))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(CHARLIE),
				Coordinates::from((2, 1))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((1, 1))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(CHARLIE),
				Coordinates::from((0, 2))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((2, 2))));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::GameFinished {
				game_id,
				winner: Some(BOB),
			}));

			assert!(!ActivePlayers::<Test>::contains_key(CHARLIE));
			assert!(!ActivePlayers::<Test>::contains_key(BOB));
			assert!(!PendingGames::<Test>::contains_key(game_id));
			assert!(!ActiveGames::<Test>::contains_key(game_id));
		});
	}

	#[test]
	fn can_draw_game() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(ALICE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(ALICE));
			assert!(ActivePlayers::<Test>::contains_key(BOB));

			let game_id = 0;

			assert!(ActiveGames::<Test>::contains_key(game_id));

			run_to_block(2);

			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((0, 0))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((0, 1))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((0, 2))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((1, 0))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((1, 2))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((1, 1))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((2, 0))
			));
			assert_ok!(TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((2, 2))));
			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(ALICE),
				Coordinates::from((2, 1))
			));

			System::assert_last_event(RuntimeEvent::TicTacToe(crate::Event::GameFinished {
				game_id,
				winner: None,
			}));

			assert!(!ActivePlayers::<Test>::contains_key(ALICE));
			assert!(!ActivePlayers::<Test>::contains_key(BOB));
			assert!(!PendingGames::<Test>::contains_key(game_id));
			assert!(!ActiveGames::<Test>::contains_key(game_id));
		});
	}

	#[test]
	fn cannot_play_out_of_turn() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(CHARLIE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(CHARLIE));
			assert!(ActivePlayers::<Test>::contains_key(BOB));

			let game_id = 0;

			assert!(ActiveGames::<Test>::contains_key(game_id));

			run_to_block(2);

			assert_noop!(
				TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((2, 0))),
				Error::<Test>::InvalidTurn
			);
		});
	}

	#[test]
	fn cannot_play_on_already_marked_cell() {
		ExtBuilder.build().execute_with(|| {
			assert_ok!(TicTacToe::create_game(RuntimeOrigin::signed(CHARLIE)));
			assert_ok!(TicTacToe::join_pending_game(RuntimeOrigin::signed(BOB)));

			assert!(ActivePlayers::<Test>::contains_key(CHARLIE));
			assert!(ActivePlayers::<Test>::contains_key(BOB));

			let game_id = 0;

			assert!(ActiveGames::<Test>::contains_key(game_id));

			run_to_block(2);

			assert_ok!(TicTacToe::play_move(
				RuntimeOrigin::signed(CHARLIE),
				Coordinates::from((2, 0))
			));

			assert_noop!(
				TicTacToe::play_move(RuntimeOrigin::signed(BOB), Coordinates::from((2, 0))),
				Error::<Test>::InvalidCell
			);
		});
	}
}
